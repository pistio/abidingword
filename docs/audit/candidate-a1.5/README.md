# Phase A1.5 evidence

Qualification of the evaluation tools, independent reviews, the fixes they
led to, the regression on all 113 disclosed editions, and the material for
the owner's Set H decision. Everything here is on disclosed documents.

| File | What it holds |
|---|---|
| **`SET-H-APPROVAL.md`** | **The owner's decision document: identities, protocol, environment, guarantees, steps, readiness verdict** |
| `FREEZE.md` | The new candidate: SHA, source checksums, toolchain, binary digests, results, limitations |
| `evaluation-audit/` | The independent audit of the evaluation system (blockers B1–B3, finding F1, conditions C1–C5) |
| **`evaluation-e2/`** | **Evaluation candidate A1.5-E2: the corrected evaluation tools for the same frozen recogniser, their rules, identities and before/after evidence. Not sealed, not run on any withheld set** |
| `evaluation-e2-review/` | The independent review of E2: not ready (R1 reference text read by the app's reader; R2 a run verifying nothing passes; R3 owner decisions) |
| **`evaluation-e3/`** | **Evaluation candidate A1.5-E3: E2 corrected after its review (the evaluation's own USFM text readers, compared verse by verse; no pass without evidence; tools pinned; a gate until the owner decides). Rules, identities, before/after evidence, owner decisions prepared, review checklist. Not sealed, not run on any withheld set** |
| `auditor-qualification.md` | PR #52's mutation suite executed; the independent attack on the completeness auditor (33 escapes in version 1); version 2 and its results |
| `reviews/security-review.md` | Independent review of resources, robustness and hostile input, with triage |
| `reviews/canon-review.md` | Independent review of canon and versification, with triage |
| `regression.md` | The 113 editions against the A1.3 candidate, edition by edition |
| `set-h-decision-package.md` | The protocol, criteria, ceilings and exact run proposed for Set H. Not authorised. |
| `rehearsal/` | Four sealed rehearsal sets of disclosed documents, each run once with the frozen binary: seals, run records, scores (the first three rescored against their committed record). Not Set H |
| `data/isolation-test-2026-11-20/` | The independent-environment check: what a restricted session can and cannot reach |
| `data/review-2026-11-19/` | Receipts of the follow-up review of `4dd9f8e`: its probes before and after the fixes |
| `data/` | Completeness audits, probe results, per-edition reports |

The handover for Codex is `docs/handover-scripture-recognition-codex.md`.
The files a withheld set's builder receives, and the disclosed-edition
exclusions, are in `docs/evaluation/`.
