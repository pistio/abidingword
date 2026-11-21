#!/usr/bin/env python3
"""Assemble evidence only after the frozen-source checks finish successfully.

Run explicitly from the repository root. This script starts no checks, CI jobs,
builds or network requests. All validation happens before tracked files are written.
"""

from __future__ import annotations

from collections import Counter
from datetime import datetime, timezone
import hashlib
import json
from pathlib import Path
import re
import shutil
import subprocess


SOURCE = "607e965d9c59a1dd93d23aaf95961944c11211a7"
BASELINE = "b8d9af869c95395a44cc6d9a3867626ab83d9804"
FIRST_SOURCE = "b5a0c8c800da83f45f475cbf3639897c4f4bf977"
DELETION_BASELINE = "3424f799baf6758afd2d918a6fd08e396de83ff9"
ROOT = Path(__file__).resolve().parents[2]
WORK = ROOT / "work/recovery-verification"
DEST = ROOT / "docs/implementation/evidence/recovery-20261120"
RECORD = ROOT / "docs/implementation/daily-recovery-20261120-verification.json"


def require(condition: bool, message: str) -> None:
    if not condition:
        raise RuntimeError(message)


def git(*args: str) -> str:
    return subprocess.check_output(["git", *args], cwd=ROOT, text=True).strip()


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def read_json(path: Path):
    require(path.is_file(), f"Missing required evidence: {path}")
    return json.loads(path.read_text())


def relative(path: Path) -> str:
    return path.relative_to(ROOT).as_posix()


def frontend_counts(report: dict) -> dict:
    files = report.get("testResults", [])
    require(bool(files), "Vitest report has no actual test files")
    names = [item["name"] for item in files]
    require(len(set(names)) == len(names), "Duplicate Vitest test-file entries")
    assertions = [test for item in files for test in item["assertionResults"]]
    statuses = Counter(test["status"] for test in assertions)
    return {
        "actualTestFiles": len(files),
        "fileStatuses": dict(Counter(item["status"] for item in files)),
        "assertions": len(assertions),
        "assertionStatuses": dict(statuses),
        "reportedTotalTests": report["numTotalTests"],
        "reportedPassedTests": report["numPassedTests"],
        "reportedFailedTests": report["numFailedTests"],
        "reportedPendingTests": report["numPendingTests"],
        "reportedTodoTests": report.get("numTodoTests", 0),
        "reportedDescribeSuites": report["numTotalTestSuites"],
        "suiteCountNote": "Vitest numTotalTestSuites includes describe groups; actualTestFiles counts testResults entries.",
    }


def native_counts(text: str) -> dict:
    pattern = re.compile(
        r"^test result: (ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored; "
        r"(\d+) measured; (\d+) filtered out;.*$", re.MULTILINE
    )
    summaries = []
    target = "unknown"
    for line in text.splitlines():
        if line.lstrip().startswith(("Running ", "Doc-tests ")):
            target = line.strip()
        match = pattern.fullmatch(line)
        if match:
            state, passed, failed, ignored, measured, filtered = match.groups()
            summaries.append({
                "target": target, "result": state, "passed": int(passed),
                "failed": int(failed), "ignored": int(ignored),
                "measured": int(measured), "filteredOut": int(filtered),
            })
    require(bool(summaries), "Cargo log contains no test summaries")
    ignored_names = re.findall(r"^test (.+?) \.\.\. ignored(?:.*)$", text, re.MULTILINE)
    return {
        "testTargets": len(summaries),
        **{key: sum(item[key] for item in summaries)
           for key in ("passed", "failed", "ignored", "measured", "filteredOut")},
        "ignoredTests": ignored_names,
        "targets": summaries,
    }


def main() -> None:
    require(git("rev-parse", "HEAD") == SOURCE, "HEAD is not the frozen implementation")
    expected = {
        "frontend": ["npm-ci", "typecheck", "eslint", "i18n", "i18n-corpus",
                     "resources", "audit", "frontend-tests", "frontend-build"],
        "native": ["rust-fmt", "rust-clippy", "rust-tests"],
    }
    records = {}
    copies: dict[Path, str] = {}

    def add(path: Path, name: str) -> None:
        require(path.is_file() and not path.is_symlink(), f"Missing/nonregular evidence: {path}")
        require(not Path(name).is_absolute() and ".." not in Path(name).parts,
                f"Invalid evidence destination: {name}")
        require(name not in copies.values(), f"Duplicate evidence destination: {name}")
        copies[path] = name

    for mode, names in expected.items():
        path = WORK / f"{mode}-checks.json"
        record = read_json(path)
        require(record.get("sourceSha") == SOURCE, f"{mode}: wrong source SHA")
        require(record.get("mode") == mode, f"{mode}: wrong check record")
        for key in ("cleanBefore", "cleanAfter", "sourceUnchanged", "headUnchanged"):
            require(record.get(key) is True, f"{mode}: {key} is not true")
        require(bool(record.get("completedUTC")), f"{mode}: checks have not completed")
        checks = record.get("checks", [])
        require([check["name"] for check in checks] == names,
                f"{mode}: incomplete or unexpected command list")
        inputs = record.get("inputHashes", {})
        require(bool(inputs), f"{mode}: missing input hashes")
        for name, expected_hash in inputs.items():
            path_input = ROOT / name
            require(path_input.is_file() and digest(path_input) == expected_hash,
                    f"{mode}: input changed since checks: {name}")
        for check in checks:
            require(check.get("exitCode") == 0, f"{mode}: failed command {check['name']}")
            log = ROOT / check["log"]
            require(log.is_file() and digest(log) == check.get("logSha256"),
                    f"{mode}: missing/changed log {log}")
            add(log, log.name)
        records[mode] = record
        add(path, path.name)
    require(records["frontend"]["inputHashes"] == records["native"]["inputHashes"],
            "Frontend and native checks used different inputs")
    tracked = subprocess.check_output(["git", "ls-files", "-z"], cwd=ROOT).split(b"\0")
    current_inputs = {name.decode(): digest(ROOT / name.decode()) for name in tracked
                      if name and not name.startswith(b"docs/") and (ROOT / name.decode()).is_file()}
    require(current_inputs == records["frontend"]["inputHashes"],
            "Tracked implementation inputs were added, removed or changed after verification")

    provenance_path = WORK / "source-provenance-final.json"
    provenance = read_json(provenance_path)
    require(provenance.get("commit") == SOURCE and provenance.get("dirty") is False,
            "Final build provenance is not the clean frozen source")
    require(provenance.get("schemaVersion") == 27 and len(provenance.get("migrations", [])) == 27,
            "Migration catalogue/schema version changed")
    for migration in provenance["migrations"]:
        path = ROOT / "src-tauri/migrations" / migration["file"]
        require(digest(path) == migration["sha256"], f"Migration hash changed: {path}")
    add(provenance_path, provenance_path.name)
    metadata_correction = read_json(WORK / "email-metadata-correction.json")
    require(metadata_correction["sourceTreesUnchanged"] is True and subprocess.run(["git", "merge-base", "--is-ancestor", metadata_correction["replacementImplementation"], SOURCE], cwd=ROOT).returncode == 0, "Invalid commit privacy correction")
    add(WORK / "email-metadata-correction.json", "email-metadata-correction.json")
    ci_failure = read_json(WORK / "ci-first-failure.json")
    require("44839514609" in json.dumps(ci_failure) and "failure" in json.dumps(ci_failure), "Missing first CI failure")
    add(WORK / "ci-first-failure.json", "ci-first-failure.json")
    ci_failure_log = WORK / "ci-first-46d27d25-rust.log"
    require("needless_bool" in ci_failure_log.read_text(), "Missing Rust1.100 lint failure")
    add(ci_failure_log, "ci-rust-first-failure.log")
    for ci_log_name in ["ci-first-46d27d25-frontend.log"]:
        add(WORK / ci_log_name, "ci-first/" + ci_log_name)

    report_path = WORK / "frontend-tests.json"
    report = read_json(report_path)
    frontend = frontend_counts(report)
    require(report.get("success") is True, "Final Vitest report did not succeed")
    require(frontend["fileStatuses"] == {"passed": frontend["actualTestFiles"]},
            "Not every frontend file passed")
    require(frontend["assertionStatuses"] == {"passed": frontend["assertions"]},
            "Frontend assertions failed or were skipped")
    require(frontend["assertions"] == frontend["reportedTotalTests"] == frontend["reportedPassedTests"],
            "Frontend actual/reported totals disagree")
    require(all(frontend[key] == 0 for key in
                ("reportedFailedTests", "reportedPendingTests", "reportedTodoTests")),
            "Frontend contains nonpassing tests")
    front_log = (WORK / "final-frontend-tests.log").read_text(errors="replace")
    require(not re.search(r"Unhandled (?:Errors|Rejection|Exception)|caught \d+ unhandled error", front_log),
            "Frontend log reports unhandled errors")
    frontend["unhandledErrors"] = 0
    frontend["unhandledEvidence"] = "Complete Vitest process exited zero; complete default-reporter log contains no unhandled-error report."
    frontend["nonFatalReactActWarnings"] = front_log.count("not wrapped in act")
    baseline_warnings = read_json(WORK / "starting-ci-warning-summary.json")
    frontend["startingCIReactActWarnings"] = baseline_warnings["nonFatalReactActWarnings"]
    frontend["warningNote"] = "Existing React test act warnings remain visible; they are not unhandled errors and were not suppressed. Full logs are retained."
    add(WORK / "starting-ci-warning-summary.json", "starting-ci-warning-summary.json")
    add(report_path, report_path.name)

    native_log = (WORK / "final-rust-tests.log").read_text(errors="replace")
    native = native_counts(native_log)
    require(native["failed"] == 0 and native["filteredOut"] == 0,
            "Native suite failed or was filtered")
    require(all(item["result"] == "ok" for item in native["targets"]), "Nonpassing Cargo summary")
    require(native["ignored"] == len(native["ignoredTests"]), "Ignored native test count is not accounted for")
    required_native = [
        "committed_wal_content_is_staged_and_restored_instead_of_stale_database_pages",
        "missing_writing_tables_and_forged_history_are_unavailable_before_replacement",
        "current_database_history_is_verified_before_any_safety_copy_or_replacement",
        "redirected_daily_cache_and_safety_directories_are_rejected_without_external_writes",
        "manual_database_names_are_neither_counted_as_today_nor_pruned",
        "a_file_restore_failure_rolls_back_database_and_already_replaced_files",
        "old_dangling_citation_rows_do_not_stop_the_upgrade",
        "a_backup_made_before_this_version_restores_and_keeps_its_citations",
        "historical_safety_copy_exception_does_not_accept_unrelated_broken_references",
    ]
    for name in required_native:
        require(f"test {name} ... ok" in native_log, f"Missing passing native regression: {name}")

    browser_path = WORK / "browser-check.json"
    browser_check = read_json(browser_path)
    require(browser_check.get("sourceSha") == SOURCE, "Browser source SHA is not final")
    for key in ("cleanBefore", "cleanAfter", "headUnchanged"):
        require(browser_check.get(key) is True, f"Browser: {key} is not true")
    require(browser_check.get("exitCode") == 0 and bool(browser_check.get("startedUTC")),
            "Browser procedure is unfinished or failed")
    if "scriptSha256" in browser_check:
        require(digest(ROOT / "tests/screenshots/recovery-interface.mjs") == browser_check["scriptSha256"],
                "Recovery browser script changed")
    for name, expected_hash in browser_check.get("artifactsSha256", {}).items():
        require(digest(ROOT / name) == expected_hash, f"Recovery browser artifact changed: {name}")
    browser_results_path = WORK / "final-browser/results.json"
    browser_results = read_json(browser_results_path)
    flows = browser_results.get("results", [])
    require(len(flows) == 36 and browser_results.get("failures") == [], "Incomplete/failed browser flows")
    require(len({flow["name"] for flow in flows}) == 36, "Duplicate browser flow")
    for flow in flows:
        require(flow.get("errors") == [] and flow.get("cancelledSafely") is True
                and flow.get("keyboardReload") is True, f"Browser flow failed: {flow['name']}")
    add(browser_path, browser_path.name)
    add(browser_results_path, "browser-results.json")
    add(WORK / "final-browser.log", "final-browser.log")
    for lang in ("en", "ko"):
        for kind in ("daily", "full"):
            name = f"{lang}-480x640-1x-{kind}.png"
            add(WORK / "final-browser" / name, "browser/" + name)
    add(WORK / "final-browser/en-640x450-2x-full.png", "browser/en-640x450-2x-full.png")

    # Exact-baseline deletion failures are separate from setup/selector mistakes.
    delete_before = read_json(WORK / "delete-before/frontend.json")
    delete_before_source = read_json(WORK / "delete-before/source.json")
    require(delete_before_source.get("sourceHead") == DELETION_BASELINE
            and delete_before_source.get("applicationDiff") == "",
            "Block deletion baseline production identity is not clean/exact")
    delete_before_counts = frontend_counts(delete_before)
    require(delete_before_counts["reportedFailedTests"] == 8
            and delete_before_counts["assertionStatuses"] == {"failed": 8},
            "Expected eight actual block-deletion baseline failures")
    parent_dir = WORK / "parent-deletion"
    parent_meta = read_json(parent_dir / "verification-metadata.json")
    parent_source = read_json(parent_dir / "before-source.json")
    require(parent_meta.get("baselineCommit") == DELETION_BASELINE
            and parent_meta.get("baselineProductionClean") is True
            and parent_source.get("baselineCommit") == DELETION_BASELINE
            and parent_source.get("productionClean") is True,
            "Parent deletion baseline production identity is not clean/exact")
    deletion_before_counts = {"blocks": 8}
    for label, filename, expected_failed in (
        ("parents", "before.log", 9),
        ("findingsAndLegacyPoints", "note-point-before.log", 6),
        ("section", "section-before.log", 1),
    ):
        content = re.sub(r"\x1b\[[0-?]*[ -/]*[@-~]", "", (parent_dir / filename).read_text())
        matches = re.findall(r"Tests\s+(\d+) failed \((\d+)\)", content)
        require(matches and matches[-1] == (str(expected_failed), str(expected_failed)),
                f"Wrong negative deletion regression count: {filename}")
        deletion_before_counts[label] = expected_failed
        add(parent_dir / filename, "before/deletion/" + filename)
    require(sum(deletion_before_counts.values()) == 24, "Deletion baseline count must be 24")
    for filename in ("frontend.json", "source.json", "stdout.log"):
        add(WORK / "delete-before" / filename, "before/deletion/blocks-" + filename)
        add(WORK / "delete-before/initial-harness-attempt" / filename,
            "before/deletion/excluded-setup/initial-block-harness-" + filename)
    add(parent_dir / "verification-metadata.json", "before/deletion/verification-metadata.json")
    add(parent_dir / "before-source.json", "before/deletion/parent-source.json")
    source_staging = WORK / "deletion-baseline-test-sources"
    baseline_test_manifest = read_json(source_staging / "manifest.json")
    require(baseline_test_manifest.get("baselineProductionCommit") == DELETION_BASELINE,
            "Wrong baseline for retained deletion test bytes")
    expected_test_sources = {"delete-save-barrier.test-source.txt": delete_before_source["testSha256"]}
    expected_test_sources.update({name.removesuffix(".tsx") + "-source.txt": expected_hash
                                  for name, expected_hash in parent_meta["baselineTestFiles"].items()})
    initial_harness_source = read_json(WORK / "delete-before/initial-harness-attempt/source.json")
    expected_test_sources["initial-block-harness.test-source.txt"] = initial_harness_source["testSha256"]
    require({Path(item["staged"]).name for item in baseline_test_manifest["files"]}
            == set(expected_test_sources), "Retained baseline test-source set is incomplete")
    baseline_test_source_paths = []
    for item in baseline_test_manifest["files"]:
        name = Path(item["staged"]).name
        path = source_staging / name
        require(item["sha256"] == expected_test_sources[name] == digest(path),
                f"Retained deletion baseline test bytes changed: {name}")
        destination = ("before/deletion/excluded-setup/" if name.startswith("initial-")
                       else "before/deletion/") + name
        add(path, destination)
        baseline_test_source_paths.append(destination)
    add(source_staging / "manifest.json", "before/deletion/test-source-manifest.json")
    note_point_source_hashes = source_staging / "note-point-before-source.sha256"
    for line in note_point_source_hashes.read_text().splitlines():
        expected_hash, name = line.split(maxsplit=1)
        if name.startswith("tests/"):
            actual_hash = expected_test_sources[Path(name).name.removesuffix(".tsx") + "-source.txt"]
        else:
            original = subprocess.check_output(["git", "show", f"{DELETION_BASELINE}:{name}"], cwd=ROOT)
            actual_hash = hashlib.sha256(original).hexdigest()
        require(actual_hash == expected_hash, f"Note/point baseline fingerprint mismatch: {name}")
    add(note_point_source_hashes, "before/deletion/note-point-before-source.sha256")
    add(parent_dir / "setup-missing-test.log", "before/deletion/excluded-setup/setup-missing-test.log")
    for attempt_name in ("browser-setup-attempt", "browser-selector-attempt",
                         "browser-error-fixture-attempt", "browser-localized-error-attempt"):
        add(parent_dir / (attempt_name + ".log"), "before/deletion/excluded-setup/" + attempt_name + ".log")
        add(parent_dir / attempt_name / "results.json", "before/deletion/excluded-setup/" + attempt_name + ".json")
    focus_before = read_json(parent_dir / "browser-before-focus-correction/results.json")
    require(focus_before.get("passed") is True
            and focus_before.get("released", {}).get("focused", "").startswith("<body>"),
            "Earlier Chromium focus observation is missing or misclassified")
    add(parent_dir / "browser-before-focus-correction/results.json", "before/deletion/body-focus-observation.json")
    add(parent_dir / "browser-before-focus-correction.log", "before/deletion/body-focus-observation.log")
    deletion_test_names = (
        "delete-save-barrier.test.tsx", "parent-delete-save-barrier.test.tsx",
        "note-point-delete-save-barrier.test.tsx", "section-delete-save-barrier.test.tsx",
    )
    final_deletion_counts = {}
    for name in deletion_test_names:
        matching = [item for item in report["testResults"] if Path(item["name"]).name == name]
        require(len(matching) == 1 and matching[0]["status"] == "passed",
                f"Missing passing final deletion file: {name}")
        assertions = matching[0]["assertionResults"]
        require(assertions and all(item["status"] == "passed" for item in assertions),
                f"Nonpassing final deletion assertions: {name}")
        final_deletion_counts[name] = len(assertions)
    # Counts are derived from the final report, not hardcoded total-suite figures.
    require(sum(final_deletion_counts.values()) >= sum(deletion_before_counts.values()),
            "Final deletion coverage is smaller than the demonstrated baseline")

    parent_check_path = parent_dir / "final-browser-check.json"
    parent_check = read_json(parent_check_path)
    require(parent_check.get("sourceSha") == SOURCE, "Parent browser is not final source")
    for key in ("cleanBefore", "cleanAfter", "headUnchanged"):
        require(parent_check.get(key) is True, f"Parent browser: {key} is not true")
    require(parent_check.get("exitCode") == 0 and bool(parent_check.get("startedUTC")),
            "Parent deletion browser run is unfinished or failed")
    require(bool(parent_check.get("finishedUTC")), "Parent deletion browser completion timestamp is missing")
    require(digest(ROOT / "tests/screenshots/parent-deletion.mjs") == parent_check.get("scriptSha256"),
            "Parent deletion browser script changed")
    require(bool(parent_check.get("artifactsSha256")), "Parent deletion artifact hashes are missing")
    for name, expected_hash in parent_check["artifactsSha256"].items():
        require(digest(ROOT / name) == expected_hash, f"Parent deletion artifact changed: {name}")
    parent_browser_path = parent_dir / "final-browser/results.json"
    parent_browser = read_json(parent_browser_path)
    require(parent_browser.get("passed") is True and parent_browser.get("errors") == [],
            "Final parent deletion browser scenario failed")
    require(parent_browser["pendingSave"]["rootInert"] is True
            and parent_browser["pendingSave"]["deletes"] == 0
            and parent_browser["pendingDelete"]["rootInert"] is True
            and parent_browser["released"]["rootInert"] is False
            and parent_browser["released"]["latePortalInert"] is False
            and parent_browser["released"]["focused"].startswith("<button"),
            "Parent deletion containment/release/focus evidence is incomplete")
    add(parent_check_path, "parent-deletion/browser-check.json")
    add(parent_browser_path, "parent-deletion/browser-results.json")
    parent_final_log = ROOT / parent_check["log"] if "log" in parent_check else parent_dir / "final-browser.log"
    if "logSha256" in parent_check:
        require(digest(parent_final_log) == parent_check["logSha256"], "Parent browser log hash changed")
    add(parent_final_log, "parent-deletion/browser.log")
    for filename in ("pending-delete.png", "failure-released.png", "success-usable.png"):
        add(parent_dir / "final-browser" / filename, "parent-deletion/" + filename)

    pre_deletion_ci_path = WORK / "ci-pre-deletion-review.json"
    pre_deletion_ci = read_json(pre_deletion_ci_path)
    require(pre_deletion_ci.get("headSha") == DELETION_BASELINE
            and pre_deletion_ci.get("workflowRunId") == 44841272562
            and pre_deletion_ci.get("status") == "completed"
            and pre_deletion_ci.get("conclusion") == "success",
            "Missing green pre-deletion CI evidence")
    add(pre_deletion_ci_path, "ci-pre-deletion/summary.json")
    for log in pre_deletion_ci["logs"]:
        path = ROOT / log["path"]
        require(digest(path) == log["sha256"], "Pre-deletion CI log changed")
        add(path, "ci-pre-deletion/" + path.name)
    for name in pre_deletion_ci["metadataFiles"]:
        path = ROOT / name
        add(path, "ci-pre-deletion/" + path.name)
    require(not git("diff", "--name-only", f"{DELETION_BASELINE}..{SOURCE}", "--", "src-tauri"),
            "Native source changed since the pre-deletion Rust1.100 CI pass")

    additional_observations = sorted((WORK / "additional-before").glob("*"))
    for path in additional_observations:
        add(path, "before/transcript-observations/" + path.name)

    baseline_logs = {
        "dailySnapshot": ROOT / "work/native-daily-before.log",
        "dailyRetention": ROOT / "work/native-retention-before.log",
        "fullBackupRollback": ROOT / "work/preservation-verification/recovery-portability-before.log",
        "materialsSettlement": ROOT / "work/preservation-verification/materials-barrier-before.log",
    }
    for name, path in baseline_logs.items():
        text = path.read_text(errors="replace")
        require("FAILED" in text or "FAIL " in text, f"Missing failing-before evidence: {name}")
        add(path, "before/" + path.name)

    attempt = WORK / "attempt-b5a0c8c"
    first_front = read_json(attempt / "frontend-tests.json")
    first_front_counts = frontend_counts(first_front)
    require(first_front_counts["reportedFailedTests"] == 7, "Historical seven-test failure evidence changed")
    first_front_record = read_json(attempt / "frontend-checks.json")
    first_native_record = read_json(attempt / "native-checks.json")
    require(first_front_record.get("sourceSha") == FIRST_SOURCE
            and first_native_record.get("sourceSha") == FIRST_SOURCE,
            "Wrong historical full-run identity")
    first_native_text = (attempt / "final-rust-tests.log").read_text(errors="replace")
    require("old_dangling_citation_rows_do_not_stop_the_upgrade ... FAILED" in first_native_text,
            "Missing historical citation failure")
    for name in ("frontend-checks.json", "frontend-tests.json", "native-checks.json",
                 "final-frontend-tests.log", "final-rust-tests.log"):
        add(attempt / name, "first-full-run/" + name)
    for record, check_name, log_name in (
        (first_front_record, "frontend-tests", "final-frontend-tests.log"),
        (first_native_record, "rust-tests", "final-rust-tests.log"),
    ):
        item = next(check for check in record["checks"] if check["name"] == check_name)
        require(item["exitCode"] != 0 and digest(attempt / log_name) == item["logSha256"],
                f"Historical failure log is mismatched: {log_name}")
    before_mock = read_json(WORK / "mock-import-before.json")
    after_mock = read_json(WORK / "mock-import-after.json")
    require(before_mock["numFailedTests"] == 7 and after_mock["success"] is True
            and after_mock["numFailedTests"] == 0, "Focused mock-order reproduction/resolution missing")
    add(WORK / "mock-import-before.json", "before/mock-import-before.json")
    add(WORK / "mock-import-after.json", "before/mock-import-after.json")

    changed = git("diff", "--name-only", f"{BASELINE}..{SOURCE}").splitlines()
    protected = [".github/workflows/", "resources/", "src-tauri/resources/",
                 "src-tauri/migrations/", "src-tauri/src/db/migrations.rs",
                 "src-tauri/src/scripture/", "src-tauri/src/library/",
                 "src-tauri/src/db/bible", "src-tauri/src/commands/bible",
                 "src-tauri/src/commands/scripture.rs"]
    forbidden = [path for path in changed if any(path.startswith(prefix) for prefix in protected)]
    require(not forbidden, f"Protected source changed: {forbidden}")
    require(not any("recogn" in path.lower() or "collection-h" in path.lower() for path in changed),
            "Recognition/withheld-evaluation path changed")
    add(Path("/workspace/work/recovery-native-env.sh"), "native-environment.sh")
    add(WORK / "run-checks.py", "run-checks.py")
    add(Path(__file__).resolve(), "assemble-evidence.py")

    file_manifest = [{"path": relative(DEST / name), "sourcePath": str(path),
                      "sha256": digest(path), "bytes": path.stat().st_size}
                     for path, name in copies.items()]
    evidence_path = "evidence/recovery-20261120/"
    command_records = []
    for mode, record in records.items():
        for check in record["checks"]:
            command_records.append({**check, "mode": mode,
                                    "log": evidence_path + Path(check["log"]).name})
    document = {
        "milestone": "Recovery, writing preservation and publication readiness",
        "assembledUTC": datetime.now(timezone.utc).isoformat(),
        "implementationSourceSha": SOURCE,
        "commitPrivacyCorrection": metadata_correction,
        "firstGitHubCIFailure": ci_failure,
        "ciLintCorrection": {"source": "e760fad122bee4b31301cb7cabdfa7c9d671e2d2", "cause": "Rust1.100 Clippy needlessly conditional Boolean after Windows cfg-elision", "change": "Equivalent per-platform symlink/reparse expressions; no path-validation behavior or migration change", "localRust": "1.99.1", "firstCIRust": "1.100.0", "finalCI": "Exact pushed-head result is recorded in PR56 after publication of evidence"},
        "preDeletionGitHubCI": {"headSha": DELETION_BASELINE, "runId": 44841272562,
                                "conclusion": "success", "summary": evidence_path + "ci-pre-deletion/summary.json",
                                "meaning": "This green run predates the newly demonstrated deletion failures and does not certify the final implementation. Native source remains unchanged, but complete final local suites are rerun."},
        "branch": "feature/daily-snapshot-recovery-20261120",
        "baselineSourceSha": BASELINE,
        "draftPullRequest": "https://github.com/pistio/abidingword/pull/56",
        "stackedOn": "https://github.com/pistio/abidingword/pull/55",
        "execution": "Codex Cloud; local Linux frontend/Rust/browser verification with invented disposable data",
        "result": "All required local commands passed on the clean frozen implementation; native Windows acceptance remains pending.",
        "sourceChecks": {mode: {key: record[key] for key in
                                  ("sourceSha", "startedUTC", "completedUTC", "cleanBefore", "cleanAfter", "sourceUnchanged", "headUnchanged")}
                         for mode, record in records.items()},
        "sourceInputFileCount": len(records["frontend"]["inputHashes"]),
        "sourceProvenance": evidence_path + "source-provenance-final.json",
        "commands": command_records,
        "frontend": frontend,
        "native": native,
        "browser": {"sourceSha": SOURCE, "check": browser_check,
                    "platform": browser_results["platform"], "passedFlows": len(flows), "failedFlows": 0,
                    "languages": ["en-GB", "de", "ko", "fr", "es", "pt-BR"],
                    "viewports": ["1280x900 CSS / DPR1", "480x640 CSS / DPR1", "640x450 CSS / DPR2"],
                    "restores": ["daily database", "full backup"],
                    "results": evidence_path + "browser-results.json",
                    "scope": "Real React UI with invented in-memory backend: availability, confirmation, keyboard cancel, pending containment, long paths, focus trap and reload request. Separate real SQLite tests establish database preservation.",
                    "notEstablished": ["Windows/WebView2", "actual Windows DPI", "screen-reader acceptance", "native file picker or Office rendering"]},
        "deletionBrowser": {"sourceSha": SOURCE, "check": parent_check,
                            "results": evidence_path + "parent-deletion/browser-results.json",
                            "passedScenarios": 1, "platform": parent_browser["platform"],
                            "scope": "Actual React application in Linux Chromium, invented backend: delayed saving and removal, physical input containment including a late portal, failed-delete release/focus and successful retry. No native Windows or SQLite acceptance."},
        "beforeEvidence": {
            "originalCI": {"url": "https://github.com/pistio/abidingword/actions/runs/44825652838",
                           "result": "failed", "frontendPassedAssertions": 1389, "frontendFiles": 129,
                           "reason": "Unhandled Invented save failure rejection despite passing assertions; rejection observer now installed before triggering rejection."},
            "baselineNativeSource": BASELINE,
            "transcriptObservations": [evidence_path + "before/transcript-observations/" + path.name for path in additional_observations],
            "transcriptEvidenceLimit": "FieldsEditor creation and pending backup/project capture failures were observed on dirty intermediate implementations; no exact tree hash or original standalone stdout log was retained. Metadata explicitly labels the transcript excerpts. Detached-draft overwrite was found by review, with corrected regressions but no executed failing-before case.",
            "failingBeforeLogs": {key: evidence_path + "before/" + path.name for key, path in baseline_logs.items()},
            "dailyBaselineCounts": native_counts(baseline_logs["dailySnapshot"].read_text()),
            "retentionBaselineCounts": native_counts(baseline_logs["dailyRetention"].read_text()),
            "rollbackBaselineCounts": native_counts(baseline_logs["fullBackupRollback"].read_text()),
            "materialsFailure": "Actual Teaching Q&A preview opened with Earlier question while the current save was pending.",
            "deletion": {"baselineSourceSha": DELETION_BASELINE,
                         "baselineProductionClean": True,
                         "exactBaselineTestSources": [evidence_path + path for path in baseline_test_source_paths],
                         "testSourceNote": "Exact baseline test bytes are hash-verified against original metadata and retained as .txt; no executable .tsx test files are added under docs. The initial harness text is separately excluded setup evidence.",
                         "executedBaselineFailures": deletion_before_counts,
                         "executedBaselineFailureTotal": sum(deletion_before_counts.values()),
                         "finalPassingTestsByFile": final_deletion_counts,
                         "finalPassingTestTotal": sum(final_deletion_counts.values()),
                         "newCoverageNote": "Additional corrected-source cases are not counted as executed failing-before cases. Final counts are derived from actual complete-suite assertions.",
                         "excludedSetupAttempts": "Initial block harness had six real early-delete failures and two incorrect Teaching selectors; only the subsequent eight-case clean baseline is counted. Missing-test and browser selector/error-fixture setup attempts are retained separately and excluded.",
                         "focusObservation": "The earlier Chromium run passed its old assertions but recorded focus on body after failure. It is an observed focus defect, not a claimed failed automated baseline. Final procedure explicitly asserts return to enabled initiating control.",
                         "baselineTestLimit": "The baseline note/point test included an unsupported TypeScript-only query option; runtime cases executed and failed meaningful assertions. The corrected test removes it; no baseline typecheck pass is claimed.",
                         "scope": "Editable blocks, Q&A, entries, findings, sections, legacy Teaching points and Project/Study/Teaching deletion; controlled invented API failures, not native Windows evidence."},
            "finalPassingNativeRegressions": required_native,
            "finalFrontendRegressionFiles": [Path(item["name"]).name for item in report["testResults"]
                                              if any(part in item["name"] for part in
                                                     ("recovery-", "materials-save-barrier", "portability.test"))],
        },
        "firstFullRunFailures": {
            "sourceAtStart": FIRST_SOURCE,
            "frontend": {"counts": first_front_counts,
                         "cleanBefore": first_front_record["cleanBefore"],
                         "sourceUnchanged": first_front_record["sourceUnchanged"],
                         "headUnchanged": first_front_record["headUnchanged"],
                         "limitation": "The first frontend run overlapped a subsequent native correction; its recorded input/HEAD change prevents treating it as a complete immutable-source certification.",
                         "cause": "Test setup imported useAutosave and loaded the Tauri API before per-file native mocks; seven native-boundary assertions failed.",
                         "correction": "Mechanical extraction of shared autosave state into src/lib/autosaveState.ts removes that setup import side effect; native API assertions remain unchanged.",
                         "focusedBeforeFailed": before_mock["numFailedTests"],
                         "focusedAfterPassed": after_mock["numPassedTests"],
                         "resolvedBy": "The complete final frontend suite at implementationSourceSha passes."},
            "native": {"failedTest": "old_dangling_citation_rows_do_not_stop_the_upgrade",
                       "cause": "Strict foreign-key validation blocked migration16's established repair of historical dangling citation references.",
                       "correction": "Only canonical schemas3–15 finding_sources references repaired by unchanged migration16 may pass pre-upgrade validation. Original rows remain in the safety copy; strict post-upgrade validation precedes restored-file installation.",
                       "resolvedBy": "The complete final Rust suite passes; current schema27 and unrelated broken references remain rejected."},
            "logs": [evidence_path + "first-full-run/final-frontend-tests.log", evidence_path + "first-full-run/final-rust-tests.log"],
        },
        "preservation": {
            "schemaVersion": 27, "migration0028": "unchanged/reserved",
            "protectedSourcePrefixesCompared": protected, "protectedChanges": forbidden,
            "changedFilesSinceBaseline": changed,
            "unchanged": ["migrations", "bundled resources", "Scripture recognition", "GitHub workflows", "owner teaching materials"],
            "dailyRestore": "Database only; imported source/image/Bible file bytes unchanged. Current writing retained in verified safety database before replacement.",
            "fullRestore": "Verified database safety copy and tracked source-file undo operations; both rollback branches attempted on failure, retained-copy location and locked UI on incomplete rollback.",
            "writing": "Shared settlement drains saves and retains failed drafts across remounts. Successful restoration prevents subsequent API calls in the same webview until reload.",
            "deletion": "Confirmed deletion settles saves before invalidating targets and freezes the affected editor/workspace until removal finishes. Save failure keeps writing and blocks deletion; removal failure releases the temporary pause and restores focus without overriding a newer focus choice. Later backup/export/recovery is not poisoned by a detached missing-record retry.",
            "integratedScenario": "Invented Study/Q&A, multiple linked and independent Teaching records, private leader text/Scripture and citations survive project transfer, full backup, daily restore and reopen. Generated native fixture exercises Materials, Markdown and DOCX privacy.",
            "withheldEvaluation": "Separate frozen recognition/Collection H lane was not accessed or integrated by this work.",
        },
        "limits": [
            "No native Windows acceptance or matching Windows binary is claimed.",
            "Linux browser DPR emulation does not establish native high-DPI, WebView2, screen-reader, file-lock/reparse or Office behaviour.",
            "Six-language review is AI editorial review, not human native-speaker approval.",
            "Single-process application-owned profile assumptions do not certify hostile concurrent filesystem replacement or external SQLite writers.",
            "Resource integrity checks do not choose a software licence or complete legal/distribution review.",
            "No Windows build, installer, paid CI, merge, release, signing, deployment, installed-application or personal-profile change was performed by this evidence assembly.",
        ],
        "githubCI": {"status": "Record the ordinary Ubuntu CI result in the PR body/checks after the final evidence commit is pushed.",
                     "authoritativeLocation": "https://github.com/pistio/abidingword/pull/56",
                     "rule": "PR body/checks must identify the exact pushed head SHA and workflow run IDs. Local evidence is pinned above; CI metadata is intentionally not recursively committed into this file."},
        "documents": ["daily-recovery-20261120.md", "recovery-language-20261120.md", "publish-readiness-20261120.md", "windows-owner-acceptance-20261120.md"],
        "files": file_manifest,
    }

    # Everything above is read-only. Validate the complete JSON before copying.
    encoded = json.dumps(document, indent=2, ensure_ascii=False) + "\n"
    for path, name in copies.items():
        target = DEST / name
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(path, target)
        require(digest(target) == digest(path), f"Copied evidence changed: {target}")
    RECORD.parent.mkdir(parents=True, exist_ok=True)
    RECORD.write_text(encoded)
    print(json.dumps({"record": relative(RECORD), "sourceSha": SOURCE,
                      "frontend": frontend, "nativePassed": native["passed"],
                      "nativeIgnored": native["ignored"], "browserFlows": len(flows),
                      "evidenceFiles": len(copies)}, indent=2))


if __name__ == "__main__":
    main()
