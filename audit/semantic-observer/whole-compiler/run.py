"""One local ordinary NoObserver whole-compiler compatibility experiment."""
import datetime
import gzip
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import time

ROOT = Path(__file__).resolve().parents[2]
OUT = Path(__file__).resolve().parent
FAMILY = ROOT / "semantic-observer-integration"
TRIDENT = FAMILY / "trident"
JOY = ROOT / "install-observer-integration/bin/joy"
SOURCE_COMPILER = ROOT / "measurements/selfhost-cli-guide/compiler.dag"
COMPILER = OUT / "compiler.dag"
BASELINE = ROOT / "measurements/lexer-v9-c2-to-c3.json"
PRODUCER = ROOT / "measurements/ci-split-readonly/platforms/bootstrap-aarch64-apple-darwin-repeat-1-attempt-1/repeat-1/c2-step.json"
PINS = json.loads(gzip.decompress((ROOT / "nox-observer/audit/semantic-observer/joy-integration/pins.json.gz").read_bytes()))["pins"]
EXPECTED_JOY = "853806f2bb0aba35c2872820597192216a20b05ee674c4096a3a525648e518eb"
EXPECTED_C2 = "76a07c08265bd2ef525164472b6b53ac3f0e6cbbedce3250c4202f40ffba34c8"
INVENTORY = OUT / "inventory.json"
PROBE = TRIDENT / "audit/self-hosting/probe-native-closure.py"
CHECKER = TRIDENT / "audit/self-hosting/check-selfhost-fixed-point.py"
ENV = dict(os.environ, CARGO_TARGET_DIR=str(ROOT / "target-observer-joy"))


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def identity(path):
    return dict(path=str(path), bytes=path.stat().st_size, sha256=sha(path))


def now():
    return datetime.datetime.now(datetime.timezone.utc).isoformat()


def clean_inputs():
    rows = {}
    for name, revision in PINS.items():
        directory = FAMILY / name
        head = subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=directory, text=True)
        status = subprocess.check_output(["git", "status", "--porcelain=v1", "--untracked-files=all"], cwd=directory, text=True)
        if head.strip() != revision or status:
            raise RuntimeError(f"input changed or dirty: {name}")
        rows[name] = dict(path=str(directory), revision=head.strip(), status_porcelain=status)
    return rows


REPORT = dict(schema="nox/whole-compiler-compatibility-launch/v1", status="preflight",
              scope="Local actual C2(S1) execution using ordinary NoObserver wrappers; no SH6 replacement or proof claim",
              started_utc=now(), commands=[], cargo_target_dir=ENV["CARGO_TARGET_DIR"])


def flush():
    RECEIPT.seek(0)
    RECEIPT.truncate()
    json.dump(REPORT, RECEIPT, indent=2)
    RECEIPT.write("\n")
    RECEIPT.flush()


def run(name, command):
    stdout, stderr = OUT / (name + ".stdout"), OUT / (name + ".stderr")
    row = dict(command=list(map(str, command)), cwd=str(TRIDENT), started_utc=now(),
               stdout=str(stdout), stderr=str(stderr), status="running")
    REPORT["commands"].append(row)
    flush()
    print(name + " started " + row["started_utc"], flush=True)
    started = time.monotonic_ns()
    with stdout.open("xb") as out, stderr.open("xb") as err:
        result = subprocess.run(row["command"], cwd=TRIDENT, env=ENV, stdout=out, stderr=err)
    row.update(status="completed", exit_code=result.returncode, ended_utc=now(),
               elapsed_nanoseconds=time.monotonic_ns() - started,
               stdout_identity=identity(stdout), stderr_identity=identity(stderr))
    flush()
    print(name + " exit " + str(result.returncode) + " " + row["ended_utc"], flush=True)
    if result.returncode:
        raise RuntimeError(name + " failed; raw logs retained")


with (OUT / "launch.json").open("x", encoding="utf-8") as RECEIPT:
    try:
        REPORT["inputs_start"] = clean_inputs()
        REPORT["joy_start"] = identity(JOY)
        if sha(JOY) != EXPECTED_JOY or sha(SOURCE_COMPILER) != EXPECTED_C2:
            raise RuntimeError("pinned binary/compiler mismatch")
        producer = json.loads(PRODUCER.read_bytes())
        if producer["status"] != "compiler-returned" or producer["result_sha256"] != EXPECTED_C2:
            raise RuntimeError("actual CI C2 producer mismatch")
        ci_c2 = PRODUCER.parent / "c2.dag"
        if ci_c2.read_bytes() != SOURCE_COMPILER.read_bytes():
            raise RuntimeError("compiler copy differs from retained actual CI C2")
        with COMPILER.open("xb") as output:
            output.write(SOURCE_COMPILER.read_bytes())
        REPORT.update(compiler=identity(COMPILER), compiler_original=identity(SOURCE_COMPILER),
                      producer=identity(PRODUCER), producer_artifact_id=10948746458,
                      baseline=identity(BASELINE), probe=identity(PROBE), checker=identity(CHECKER))
        run("inventory", ["cargo", "run", "--release", "--locked", "--offline", "--example",
                          "selfhost_inventory", "--", "--root", ".", "--entry", "compiler/nox/main.tri",
                          "--output", INVENTORY])
        inventory = json.loads(INVENTORY.read_bytes())
        baseline = json.loads(BASELINE.read_bytes())
        if sha(INVENTORY) != baseline["inventory_sha256"]:
            raise RuntimeError("inventory differs from frozen S1")
        rows = {}
        for name, row in inventory["modules"].items():
            source = TRIDENT / row["path"]
            git_bytes = subprocess.check_output(["git", "show", PINS["trident"] + ":" + row["path"]], cwd=TRIDENT)
            if source.read_bytes() != git_bytes or sha(source) != baseline["sources"][name]["sha256"]:
                raise RuntimeError("source differs from clean Git or frozen S1: " + name)
            rows[name] = dict(path=row["path"], sha256=sha(source), source_bytes=source.stat().st_size)
        REPORT.update(inventory=identity(INVENTORY), sources=rows,
                      inventory_checker=identity(ROOT / "target-observer-joy/release/examples/selfhost_inventory"))
        REPORT["status"] = "preflight-passed"
        flush()
        run("compiler", ["python3", PROBE, "--joy", JOY, "--compiler", COMPILER,
                         "--inventory", INVENTORY, "--output", OUT / "c2-to-c3.json",
                         "--budget", "20000000000", "--arena-nodes", "1000000000",
                         "--resident-nodes", "3145728", "--collection-work", "10000000000",
                         "--time-ms", "7200000", "--validation-visits", "16777216", "--emit", "program"])
        REPORT["status"] = "execution-returned"
    except BaseException as error:
        REPORT.update(status="failed", error=dict(kind=type(error).__name__, message=str(error)))
        raise
    finally:
        REPORT["ended_utc"] = now()
        REPORT["joy_end"] = identity(JOY)
        REPORT["compiler_end"] = identity(COMPILER) if COMPILER.exists() else None
        try:
            REPORT["inputs_end"] = clean_inputs()
            if sha(JOY) != EXPECTED_JOY or (COMPILER.exists() and sha(COMPILER) != EXPECTED_C2):
                raise RuntimeError("binary or compiler changed")
        except BaseException as error:
            REPORT.update(status="inputs-changed", input_error=str(error))
        flush()
