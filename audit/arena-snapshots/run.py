"""Retain explicit Rust 1.89 snapshot gate commands, source maps and bounded logs."""
import hashlib
import json
import os
from pathlib import Path
import signal
import subprocess
import sys
import time

ROOT = Path(__file__).resolve().parents[2]
OUT = Path(__file__).resolve().parent
FAMILY = ROOT.parent
BIN = Path('/Users/master/.rustup/toolchains/1.89.0-aarch64-apple-darwin/bin')
OWNED = ['specs/sequential-compaction.md', 'rs/sequential/compacting.rs',
         'rs/sequential/observe/mod.rs', 'rs/sequential/observe/hook.rs',
         'rs/sequential/observe/capture.rs', 'rs/sequential/observe/stream.rs',
         'rs/sequential/observe/v2.rs', 'rs/sequential/observe/wire.rs',
         'rs/sequential/observe/tests.rs', 'rs/sequential/observe/tests/snapshots.rs',
         'rs/sequential/observe/tests/snapshots/failures.rs']


def identity(path):
    raw = path.read_bytes()
    return dict(bytes=len(raw), sha256=hashlib.sha256(raw).hexdigest())


def git(path, *args):
    return subprocess.check_output(['git', '-C', str(path), *args], text=True).strip()


def sources():
    return dict(revision=git(ROOT, 'rev-parse', 'HEAD'),
                files={name: identity(ROOT / name) for name in OWNED if (ROOT / name).exists()},
                siblings={name: dict(path=str((FAMILY / name).resolve()),
                                     revision=git(FAMILY / name, 'rev-parse', 'HEAD'),
                                     status=git(FAMILY / name, 'status', '--porcelain'))
                          for name in ['strata', 'hemera', 'lens', 'honeycrisp']})


def rss_tree(pid):
    rows = [tuple(map(int, row.split())) for row in
            subprocess.check_output(['/bin/ps', '-axo', 'pid=,ppid=,rss='], text=True).splitlines()]
    selected = {pid}
    while True:
        following = selected | {child for child, parent, _ in rows if parent in selected}
        if following == selected:
            return sum(rss for child, _, rss in rows if child in selected)
        selected = following


kind = sys.argv[1]
assert len(sys.argv) == 2 or sys.argv[2].isdigit()
label = kind + ('-' + sys.argv[2] if len(sys.argv) > 2 else '')
commands = {
    'check': ['check', '--workspace', '--all-targets', '--release', '--locked', '--offline'],
    'focused': ['test', '-p', 'cyber-nox', '--lib', '--release', '--locked', '--offline',
                'sequential::observe', '--', '--nocapture', '--test-threads=2'],
    'workspace': ['test', '--workspace', '--release', '--locked', '--offline', '--', '--test-threads=2'],
    'all-features': ['test', '--workspace', '--all-features', '--release', '--locked', '--offline', '--', '--test-threads=2'],
}
argv = ['/Users/master/.cargo/bin/rustup', 'run', '1.89.0', str(BIN / 'cargo'), *commands[kind]]
env = dict(os.environ, PATH=str(BIN) + ':' + os.environ['PATH'], RUSTUP_TOOLCHAIN='1.89.0',
           RUSTC=str(BIN / 'rustc'), RUSTDOC=str(BIN / 'rustdoc'),
           CARGO_BUILD_JOBS='2', CARGO_TARGET_DIR=str(FAMILY / 'target'))
path = OUT / (label + '-command.json')
assert not path.exists()
report = dict(schema=1, scope='bounded observer snapshots; no proof acceptance', label=label,
              argv=argv, cwd=str(ROOT), started_ns=time.time_ns(),
              environment={key: value for key, value in env.items() if key == 'PATH' or key.startswith(('RUST', 'CARGO'))},
              toolchain={name: dict(path=str(BIN / name), **identity(BIN / name),
                                    version=subprocess.check_output([str(BIN / name), '--version', '--verbose'], env=env, text=True))
                         for name in ['cargo', 'rustc']}, sources_before=sources(),
              runner=identity(Path(__file__)), rss_limit_kib=2 * 1024 * 1024, rss_sampling_ms=250,
              log_limit_bytes=128 << 20, sampled_peak_tree_rss_kib=0, status='running')
with (OUT / (label + '.stdout')).open('xb') as stdout, (OUT / (label + '.stderr')).open('xb') as stderr:
    process = subprocess.Popen(argv, cwd=ROOT, env=env, stdout=stdout, stderr=stderr, start_new_session=True)
    while process.poll() is None:
        report['sampled_peak_tree_rss_kib'] = max(report['sampled_peak_tree_rss_kib'], rss_tree(process.pid))
        if report['sampled_peak_tree_rss_kib'] > report['rss_limit_kib'] or stdout.tell() + stderr.tell() > report['log_limit_bytes']:
            report['guard_failure'] = 'RSS or logs'
            os.killpg(process.pid, signal.SIGKILL)
        path.write_text(json.dumps(report, indent=2) + '\n')
        time.sleep(0.25)
    report['exit_code'] = process.wait()
report.update(ended_ns=time.time_ns(), sources_after=sources(),
              stdout=identity(OUT / (label + '.stdout')), stderr=identity(OUT / (label + '.stderr')))
report['warning_lines'] = [line for suffix in ['stdout', 'stderr']
                           for line in (OUT / (label + '.' + suffix)).read_text().splitlines()
                           if line.startswith('warning:')]
report['status'] = ('passed' if report['exit_code'] == 0 and not report['warning_lines']
                    and 'guard_failure' not in report and report['sources_before'] == report['sources_after'] else 'failed')
path.write_text(json.dumps(report, indent=2) + '\n')
print(json.dumps({key: report[key] for key in ['label', 'status', 'exit_code', 'sampled_peak_tree_rss_kib', 'warning_lines']}))
raise SystemExit(0 if report['status'] == 'passed' else 1)
