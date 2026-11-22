#!/usr/bin/env python3
"""The builder's check before sealing, and the seal itself (Phase A1.5).

    seal-set.py <seal folder> <documents folder> [--inventory=FILE] [--protocol=FILE] [--write]

Without --write: runs every check of check-set.py on the folder as it
would be sealed now, and reports. The reviewer's ledger may still be
missing (noted, not refused).

With --write: the same checks, and the reviewer's signed review-ledger.md
and access-records.md must exist and not be empty; only then is SEAL.txt
written (the UTC time and the SHA-256 of every other file). A folder that
already has SEAL.txt is never sealed again.

Exit 0 when the checks pass (and, with --write, SEAL.txt was written);
otherwise 2. check-set.py is used unchanged. --protocol (E2): the protocol
whose required composition the set must meet (default: the committed
docs/evaluation/protocol-e3.json). Any other option is refused (E3).
"""
import datetime
import hashlib
import importlib.util
import json
import os
import shutil
import sys
import tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
_spec = importlib.util.spec_from_file_location("check_set", os.path.join(HERE, "check-set.py"))
check_set = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(check_set)


def seal_text(folder, when):
    """SEAL.txt for the regular files of `folder`; links are never followed
    (check-set.py refuses them)."""
    entries = []
    for rel in check_set.plain_files(folder, [], "seal folder"):
        if rel != "SEAL.txt":
            data = open(os.path.join(folder, *rel.split("/")), "rb").read()
            entries.append((rel, hashlib.sha256(data).hexdigest()))
    return "\n".join([f"sealed {when}"] + [f"{h}  {rel}" for rel, h in sorted(entries)]) + "\n"


def main():
    args, opts, flags, bad = check_set.parse_options(sys.argv[1:], ("inventory", "protocol"), ("write",))
    if bad:
        print(f"refused: unknown or repeated option(s) {', '.join(bad)}; options are --inventory=..., "
              f"--protocol=... and --write", file=sys.stderr)
        sys.exit(2)
    if len(args) != 2:
        sys.exit(__doc__)
    seal, docs = args
    write = "write" in flags
    if os.path.exists(os.path.join(seal, "SEAL.txt")):
        print("refused: SEAL.txt exists; a sealed set is never sealed again", file=sys.stderr)
        sys.exit(2)
    notes = []
    for name in ("review-ledger.md", "access-records.md"):
        p = os.path.join(seal, name)
        if not os.path.exists(p) or not open(p, encoding="utf-8").read().strip():
            if write:
                print(f"refused: {name} is missing or empty; it must be written before the seal", file=sys.stderr)
                sys.exit(2)
            notes.append(f"{name} not written yet")
    when = datetime.datetime.now(datetime.timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")
    with tempfile.TemporaryDirectory() as tmp:
        trial = os.path.join(tmp, "seal")
        shutil.copytree(seal, trial, symlinks=True)  # a link stays a link, and is refused
        for name in ("review-ledger.md", "access-records.md"):
            p = os.path.join(trial, name)
            if not os.path.exists(p):
                open(p, "w", encoding="utf-8").write("(not written yet)\n")
        text = seal_text(trial, when)
        open(os.path.join(trial, "SEAL.txt"), "w", encoding="utf-8").write(text)
        report = check_set.check(trial, docs, None, opts.get("inventory"), opts.get("protocol"))
    report["seal_folder"] = seal
    report.setdefault("notes", []).extend(notes)
    print(json.dumps(report, ensure_ascii=False, indent=1))
    for e in report["errors"]:
        print("refused:", e, file=sys.stderr)
    for n in report["notes"]:
        print("note:", n, file=sys.stderr)
    if not report["ok"]:
        sys.exit(2)
    if write:
        open(os.path.join(seal, "SEAL.txt"), "w", encoding="utf-8").write(seal_text(seal, when))
        print(f"sealed {when}", file=sys.stderr)


if __name__ == "__main__":
    main()
