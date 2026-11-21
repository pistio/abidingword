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


SOURCE = "e760fad122bee4b31301cb7cabdfa7c9d671e2d2"
BASELINE = "b8d9af869c95395a44cc6d9a3867626ab83d9804"
FIRST_SOURCE = "b5a0c8c800da83f45f475cbf3639897c4f4bf977"
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
        "ciLintCorrection": {"source": SOURCE, "cause": "Rust1.100 Clippy needlessly conditional Boolean after Windows cfg-elision", "change": "Equivalent per-platform symlink/reparse expressions; no path-validation behavior or migration change", "localRust": "1.99.1", "firstCIRust": "1.100.0", "finalCI": "Exact pushed-head result is recorded in PR56 after publication of evidence"},
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
