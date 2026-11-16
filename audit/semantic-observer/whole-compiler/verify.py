"""Verify one ordinary observer-runtime producer using existing Trident guards."""
import datetime
import hashlib
import importlib.util
import json
from pathlib import Path
import subprocess
import sys
import zipfile

sys.dont_write_bytecode = True

OUT = Path(__file__).resolve().parent
ROOT = OUT.parents[1]
TRIDENT = ROOT / "semantic-observer-integration/trident"
HELPER = TRIDENT / "audit/self-hosting/check-selfhost-fixed-point.py"
INVENTORY_CHECKER = ROOT / "target-observer-joy/release/examples/selfhost_inventory"
JOY = ROOT / "install-observer-integration/bin/joy"
BASELINE = ROOT / "measurements/lexer-v9-c2-to-c3.json"


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def require(condition, message):
    if not condition:
        raise ValueError(message)


def differences(a, b, prefix=""):
    if isinstance(a, dict) and isinstance(b, dict):
        out = []
        for key in sorted(set(a) | set(b)):
            path = prefix + "/" + key
            if key not in a or key not in b:
                out.append(dict(path=path, baseline=a.get(key), observed=b.get(key)))
            else:
                out.extend(differences(a[key], b[key], path))
        return out
    return [] if a == b else [dict(path=prefix, baseline=a, observed=b)]


report = dict(schema="nox/whole-compiler-compatibility-verification/v1", status="running",
              scope="Single ordinary NoObserver C2(S1) producer and non-time compatibility; no new two-stage fixed-point, semantic corpus, SH6 or proof claim",
              started_utc=datetime.datetime.now(datetime.timezone.utc).isoformat(),
              producer=str(OUT / "c2-to-c3.json"), baseline=str(BASELINE),
              inventory_checker=str(INVENTORY_CHECKER), inventory_checks=[],
              job_checker=str(JOY), job_checks=[], steps=[])
with (OUT / "verification.json").open("x", encoding="utf-8") as output:
    try:
        launch = json.loads((OUT / "launch.json").read_bytes())
        require(launch["status"] == "execution-returned", "launcher did not complete successfully")
        actual = json.loads((OUT / "c2-to-c3.json").read_bytes())
        baseline = json.loads(BASELINE.read_bytes())
        origin_path = ROOT / "measurements/selfhost-cli-guide/receipt.json"
        origin = json.loads(origin_path.read_bytes())["ci_compiler_origin"]
        require(origin["artifact_id"] == launch["producer_artifact_id"] == 10948746458,
                "CI origin artifact differs")
        ci_step = Path(launch["producer"]["path"])
        ci_root = ci_step.parent.parent
        ci_receipt = json.loads((ci_root / "receipt.json").read_bytes())
        ci_files = json.loads((ci_root / "files.json").read_bytes())
        require(origin["producer_receipt_sha256"] == sha(ci_root / "receipt.json"),
                "CI origin platform receipt differs")
        require(ci_receipt["status"] == "passed" and ci_receipt["ci_origin"]["run_id"] == str(origin["run_id"]),
                "CI platform origin/status differs")
        require(sha(ci_root / "files.json") == ci_receipt["files"]["sha256"], "CI file manifest differs")
        for relative in ("repeat-1/c2-step.json", "repeat-1/c2.dag"):
            file = ci_root / relative
            require(file.stat().st_size == ci_files[relative]["bytes"] and sha(file) == ci_files[relative]["sha256"],
                    "CI producer/artifact manifest binding differs")
        require(launch["producer"]["sha256"] == sha(ci_step), "CI nested producer changed")
        ci_metadata_path = ROOT / "measurements/ci-split-readonly/artifact-10948746458.json"
        ci_metadata = json.loads(ci_metadata_path.read_bytes())
        ci_zip = ci_metadata_path.with_suffix(".zip")
        require(ci_metadata["id"] == origin["artifact_id"] and ci_metadata["workflow_run"]["id"] == origin["run_id"], "CI API origin differs")
        require(ci_metadata["workflow_run"]["head_sha"] == launch["inputs_start"]["trident"]["revision"], "CI API source revision differs")
        require(ci_zip.stat().st_size == ci_metadata["size_in_bytes"] and "sha256:" + sha(ci_zip) == ci_metadata["digest"], "original CI ZIP identity differs")
        with zipfile.ZipFile(ci_zip) as original:
            for relative in ("receipt.json", "files.json", "repeat-1/c2-step.json", "repeat-1/c2.dag"):
                require(original.read(relative) == (ci_root / relative).read_bytes(), "CI ZIP member differs: " + relative)
        report["compiler_ci_origin"] = dict(origin, guide_receipt_sha256=sha(origin_path))
        report["compiler_ci_origin"].update(nested_producer_sha256=sha(ci_step),
                                            api_metadata_sha256=sha(ci_metadata_path),
                                            zip_sha256=sha(ci_zip), zip_bytes=ci_zip.stat().st_size,
                                            original_zip_members_equal=True)
        report.update(helper_sha256=sha(HELPER), baseline_sha256=sha(BASELINE),
                      launch_sha256=sha(OUT / "launch.json"), producer_sha256=sha(OUT / "c2-to-c3.json"),
                      inventory_checker_sha256_start=sha(INVENTORY_CHECKER),
                      job_checker_sha256_start=sha(JOY))
        require(report["helper_sha256"] == launch["checker"]["sha256"], "checker changed")
        require(report["baseline_sha256"] == launch["baseline"]["sha256"], "baseline changed")
        require(report["inventory_checker_sha256_start"] == launch["inventory_checker"]["sha256"], "inventory checker changed")
        spec = importlib.util.spec_from_file_location("existing_fixed_point", HELPER)
        helper = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(helper)
        # Invoke the unchanged per-producer guard, including snapshot inventory
        # and canonical repack. Two-runtime compatibility is not its two-step
        # fixed-point relation, which correctly requires one runtime identity.
        step = helper.step(OUT / "c2-to-c3.json", INVENTORY_CHECKER, report)
        actual_dir = Path(actual["artifact_directory"])
        baseline_dir = Path(baseline["artifact_directory"])
        compiler = Path(actual["compiler"]).read_bytes()
        result = (actual_dir / "result.dag").read_bytes()
        require(result == compiler, "actual C3 differs from input C2 bytes")
        require(result == (baseline_dir / "result.dag").read_bytes(), "actual C3 differs from frozen S1 result")
        require((actual_dir / "job.dag").read_bytes() == (baseline_dir / "job.dag").read_bytes(), "JOB1 differs from frozen S1")
        require(actual["manifest"] == baseline["manifest"], "manifest differs from frozen S1")
        require(actual["inventory_sha256"] == baseline["inventory_sha256"], "inventory differs from frozen S1")
        require(actual["admission"]["package"] == baseline["admission"]["package"], "admitted package differs")
        source_rows = {}
        revision = launch["inputs_start"]["trident"]["revision"]
        for name, row in actual["sources"].items():
            git_bytes = subprocess.check_output(["git", "show", revision + ":" + row["path"]], cwd=TRIDENT)
            snapshot = Path(row["copy"]).read_bytes()
            require(snapshot == git_bytes == (TRIDENT / row["path"]).read_bytes(), "source Git/snapshot/live mismatch: " + name)
            require(row["sha256"] == baseline["sources"][name]["sha256"], "source differs from S1: " + name)
            source_rows[name] = dict(path=row["path"], sha256=row["sha256"], source_bytes=len(snapshot))
        require(source_rows == launch["sources"], "source set differs from launch")
        require(len(source_rows) == 94 and sum(row["source_bytes"] for row in source_rows.values()) == 370544,
                "whole compiler source extent differs")
        before = dict(baseline["execution"]["execution"])
        after = dict(actual["execution"]["execution"])
        timings = dict(baseline_elapsed_micros=before.pop("elapsed_micros"), observed_elapsed_micros=after.pop("elapsed_micros"))
        diff = differences(before, after)
        report.update(non_time_execution_differences=diff, timing=timings,
                      compared_execution_top_level_fields=sorted(before),
                      baseline_host_flags=baseline["host_flags"], observed_host_flags=actual["host_flags"],
                      source_revision=revision, source_sha256_set=source_rows,
                      exact_C2_C3_bytes_equal=True, exact_frozen_S1_result_bytes_equal=True,
                      exact_frozen_S1_JOB1_bytes_equal=True, artifact_sha256=sha(actual_dir / "result.dag"),
                      artifact_bytes=len(result), published_particle=actual["execution"]["published_particle"])
        require(not diff, "non-time execution fields differ; complete differences retained")
        old_host = dict(zip(baseline["host_flags"][::2], baseline["host_flags"][1::2]))
        new_host = dict(zip(actual["host_flags"][::2], actual["host_flags"][1::2]))
        require(old_host.pop("--time-ms") == "3600000" and new_host.pop("--time-ms") == "7200000" and old_host == new_host,
                "unexpected host profile difference")
        require(actual["execution"]["published_particle"] == baseline["execution"]["published_particle"], "published particle differs")
        for name, initial in launch["inputs_start"].items():
            directory = Path(initial["path"])
            require(subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=directory, text=True).strip() == initial["revision"], "revision changed: " + name)
            require(not subprocess.check_output(["git", "status", "--porcelain=v1", "--untracked-files=all"], cwd=directory), "input dirty: " + name)
        report["inventory_checker_sha256_end"] = sha(INVENTORY_CHECKER)
        report["job_checker_sha256_end"] = sha(JOY)
        require(report["inventory_checker_sha256_start"] == report["inventory_checker_sha256_end"], "inventory checker changed")
        require(report["job_checker_sha256_start"] == report["job_checker_sha256_end"] == launch["joy_start"]["sha256"], "Joy changed")
        report["status"] = "passed"
    except BaseException as error:
        report.update(status="failed", error=dict(kind=type(error).__name__, message=str(error)))
        raise
    finally:
        report["ended_utc"] = datetime.datetime.now(datetime.timezone.utc).isoformat()
        json.dump(report, output, indent=2)
        output.write("\n")
print(json.dumps({key: report[key] for key in ("status", "artifact_sha256", "artifact_bytes", "timing", "non_time_execution_differences")}))
