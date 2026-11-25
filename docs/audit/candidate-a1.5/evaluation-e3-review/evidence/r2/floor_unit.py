#!/usr/bin/env python3
"""The evidence floor (aw-evidence-1) of score-set.py, called directly with
protocol variants a careless or hostile revision might contain, and the
repeated-flag behaviour of the option parser.

    floor_unit.py <repo> <out.json>
"""
import importlib.util, json, math, os, sys

repo, out = sys.argv[1:3]
spec = importlib.util.spec_from_file_location("score_set", os.path.join(repo, "scripts", "evaluation", "score-set.py"))
ss = importlib.util.module_from_spec(spec)
spec.loader.exec_module(ss)


def docs(*faithful):
    return [{"observed": {"verified_faithful": f}} for f in faithful]


base = {"minimum_verified_faithful": 1, "minimum_documents_with_verified_faithful": None,
        "minimum_verified_faithful_share_of_printed_scorable": None, "maximum_scored_documents_refused_whole": None}
cases = [
    ("committed default, 0 faithful", base, 0, 100, [0], 0, False),
    ("committed default, 1 faithful of 31146", base, 1, 31146, [1, 0], 0, True),
    ("no evidence_floor key", None, 5, 10, [5], 0, False),
    ("evidence_floor not an object", "1", 5, 10, [5], 0, False),
    ("minimum 0", {**base, "minimum_verified_faithful": 0}, 5, 10, [5], 0, False),
    ("minimum -1", {**base, "minimum_verified_faithful": -1}, 5, 10, [5], 0, False),
    ("minimum as string", {**base, "minimum_verified_faithful": "1"}, 5, 10, [5], 0, False),
    ("minimum as float 1.0", {**base, "minimum_verified_faithful": 1.0}, 5, 10, [5], 0, False),
    ("minimum true", {**base, "minimum_verified_faithful": True}, 5, 10, [5], 0, False),
    ("minimum missing, others set", {k: v for k, v in base.items() if k != "minimum_verified_faithful"}, 5, 10, [5], 0, False),
    ("unknown measure", {**base, "minimum_verified_faithfull": 3}, 5, 10, [5], 0, False),
    ("documents floor 2, one document", {**base, "minimum_documents_with_verified_faithful": 2}, 5, 10, [5, 0], 0, False),
    ("documents floor 2, two documents", {**base, "minimum_documents_with_verified_faithful": 2}, 5, 10, [3, 2], 0, True),
    ("share 0.5 met", {**base, "minimum_verified_faithful_share_of_printed_scorable": 0.5}, 5, 10, [5], 0, True),
    ("share 0.5 unmet", {**base, "minimum_verified_faithful_share_of_printed_scorable": 0.5}, 4, 10, [4], 0, False),
    ("share as string", {**base, "minimum_verified_faithful_share_of_printed_scorable": "0.5"}, 5, 10, [5], 0, False),
    ("share NaN", {**base, "minimum_verified_faithful_share_of_printed_scorable": math.nan}, 5, 10, [5], 0, False),
    ("refused-whole max 0, one refused", {**base, "maximum_scored_documents_refused_whole": 0}, 5, 10, [5], 1, False),
    ("refused-whole max NaN", {**base, "maximum_scored_documents_refused_whole": math.nan}, 5, 10, [5], 0, False),
    ("refused-whole max -inf", {**base, "maximum_scored_documents_refused_whole": -math.inf}, 5, 10, [5], 0, False),
    ("note key only extra", {**base, "note": "x"}, 1, 10, [1], 0, True),
]
res = []
for name, floor, faithful, printed, per_doc, refused, expect in cases:
    proto = {} if floor is None else {"evidence_floor": floor}
    ok, measures, reasons = ss.evidence_floor(proto, {"verified_faithful": faithful, "printed": printed}, docs(*per_doc), refused)
    res.append({"case": name, "sufficient": ok, "reviewer_expects": expect, "as_expected": ok == expect, "reasons": reasons})
# Repeated flags (no value) in the shared option parser.
cs = ss.check_set
for argv, allowed, flags in ((["a", "b", "--write", "--write"], ("inventory", "protocol"), ("write",)),
                             (["a", "b", "--diagnostic", "--diagnostic"], ss.OPTIONS, ("diagnostic",))):
    args, opts, given, bad = cs.parse_options(argv, allowed, flags)
    res.append({"case": f"repeated flag {argv[2]}", "refused": bool(bad), "note": "a repeated flag is accepted (harmless: idempotent)"})
json.dump(res, open(out, "w", encoding="utf-8"), indent=1, default=str)
for r in res:
    print(r)
