use std::{env, fs, io, path::Path, process::{Command, Stdio}, sync::mpsc,
    thread, time::{Duration, Instant, SystemTime, UNIX_EPOCH}};

fn wall_ns() -> io::Result<u128> {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_nanos()).map_err(io::Error::other)
}
fn observe(root: &Path, phase: &str, index: usize, full: bool) -> io::Result<()> {
    let mut commands: Vec<(&str, &str, Vec<&str>)> = vec![
        ("ps", "/bin/ps", vec!["-axo", "pid,ppid,pcpu,comm"]),
        ("load", "/usr/sbin/sysctl", vec!["vm.loadavg"]),
    ];
    if full {
        commands.extend([
            ("power", "/usr/bin/pmset", vec!["-g", "batt"]),
            ("power-settings", "/usr/bin/pmset", vec!["-g", "custom"]),
            ("thermal", "/usr/bin/pmset", vec!["-g", "therm"]),
        ]);
    }
    for (label, binary, args) in commands {
        let stem = format!("{phase}-{index:03}-{label}");
        let started = wall_ns()?;
        let clock = Instant::now();
        match Command::new(binary).args(&args).output() {
            Ok(result) => {
                fs::write(root.join(format!("{stem}.stdout")), result.stdout)?;
                fs::write(root.join(format!("{stem}.stderr")), result.stderr)?;
                fs::write(root.join(format!("{stem}.receipt")), format!(
                    "command={binary} {}\nstarted_unix_ns={started}\nelapsed_ns={}\nexit_code={}\n",
                    args.join(" "), clock.elapsed().as_nanos(), result.status.code().unwrap_or(-1)))?;
            }
            Err(error) => {
                fs::write(root.join(format!("{stem}.unavailable")), format!(
                    "command={binary} {}\nstarted_unix_ns={started}\nerror={error}\n", args.join(" ")))?;
            }
        }
    }
    Ok(())
}
fn main() -> io::Result<()> {
    let args: Vec<_> = env::args().collect();
    if args.len() < 4 || args[2] != "--" {
        return Err(io::Error::other("usage: measure NEW_OUTPUT_DIRECTORY -- COMMAND ARGS..."));
    }
    let root = Path::new(&args[1]);
    fs::create_dir(root)?;
    let telemetry = root.join("telemetry");
    fs::create_dir(&telemetry)?;
    observe(&telemetry, "before", 0, true)?;
    let stdout = fs::File::create(root.join("stdout"))?;
    let stderr = fs::File::create(root.join("stderr"))?;
    let started = wall_ns()?;
    let clock = Instant::now();
    let mut child = Command::new(&args[3]).args(&args[4..])
        .stdout(Stdio::from(stdout)).stderr(Stdio::from(stderr)).spawn()?;
    let child_pid = child.id();
    let (send, receive) = mpsc::channel();
    let sample_root = telemetry.clone();
    let sampler = thread::spawn(move || -> io::Result<()> {
        let mut index = 0;
        loop {
            match receive.recv_timeout(Duration::from_secs(5)) {
                Ok(()) | Err(mpsc::RecvTimeoutError::Disconnected) => return Ok(()),
                Err(mpsc::RecvTimeoutError::Timeout) => {
                    index += 1;
                    observe(&sample_root, "during", index, false)?;
                }
            }
        }
    });
    let status = child.wait()?;
    let elapsed = clock.elapsed().as_nanos();
    let finished = wall_ns()?;
    let _ = send.send(());
    let sampler_result = sampler.join().map_err(|_| io::Error::other("sampler panicked"))?;
    // Preserve process outcome even if the sampler itself failed.
    fs::write(root.join("process.json"), format!(
        "{{\"child_pid\":{child_pid},\"started_unix_ns\":{started},\"finished_unix_ns\":{finished},\"elapsed_ns\":{elapsed},\"exit_code\":{},\"sample_interval_seconds\":5,\"sampler_ok\":{}}}\n",
        status.code().unwrap_or(-1), sampler_result.is_ok()))?;
    sampler_result?;
    observe(&telemetry, "after", 0, true)?;
    if !status.success() {
        std::process::exit(status.code().unwrap_or(1));
    }
    Ok(())
}
