# Owner follow-up — 19 November 2026

Base: `d26981e6404de72a8c4717ae1d3a0b0c53431b66`, the current PR #50 branch.
The delivered Windows installer contains application source
`539b54d65c2cecf010cd1573b8bd51c91ac640bd` and predates these wording changes.

## German-only study acronym

The owner asks to remove POZETEK everywhere outside German and keep each
language's alternative consistent. The five non-German catalogues already
had descriptive method names; twenty remaining strings still used the German
acronym in the application-step explanation and repair controls.

Those strings and two English resource fallbacks are corrected. The existing
names are retained and documented in [the language policy](../i18n.md#german-only-pozetek-name).
German is unchanged. The exact former default guidance remains compatibility
metadata so existing unedited studies display the new translation. Stable
IDs and the user's own text are preserved.

Fourteen new regression cases cover all non-German catalogues, the German
name, current resource fallbacks, historical default guidance and custom
wording. Both generated multilingual review documents have been refreshed
without losing their historical change section.

## Bottom-border symptom accepted

The owner reports that the bottom-border problem no longer recurs and
considers it solved. Issue #49 is closed on that report. The
[acceptance record](issue-49-original/owner-acceptance-20261119.md) distinguishes
the owner's current observation from the earlier measured native comparison.
PR #50 remains draft.

## Source-reference work resumed

The latest recorded actual MacArthur failure remains
[the 8 November owner report](https://github.com/pistio/abidingword/issues/35#issuecomment-6726664888):
reference indexing took 304,681 ms before timing out, at 448/1,689 blocks.
The transaction rolled back, retaining older analysis. Its query was null,
so that report does not diagnose the result for `Apg 6`.

The subsequent indexing optimization from
[PR #43](https://github.com/pistio/abidingword/pull/43) is already in delivered
`539b54d`. The GitHub comparison has merge base
`38c785be723f4ff4ca7ba8c885aeabff3a89df7a` (PR #43), 38 commits ahead and zero
behind. No cherry-pick or new build is needed to make that optimization
available in the delivered application.

Current code prioritizes embedded/printed Contents, then real headings, then
literal body mentions within a structured PDF. Existing invented-document
regressions distinguish an opening page from the Contents listing and retain
the broader book-introduction fallback. These regressions do not establish
the result on the owner's actual MacArthur edition.

`pdf_build::ensure_contents` intentionally reuses saved structure, including
legacy empty results. Installing an update does not silently rebuild it.
If that source has not been rebuilt since updating, the previous analysis can
therefore remain in use. The installed application already provides the
required rebuild and diagnostic controls:

1. Open the existing source and choose **… → Chapters and headings → Rebuild
   structure**. Let the operation finish and retain its displayed outcome.
2. Select Acts 6 in Scripture and open this source's result. Note the actual
   destination and the expected Acts-introduction page.
3. Open **Diagnostic report (metadata only)**, explicitly type **Apg 6** in
   **Passage to check (optional)**, and choose **Save report…**. Saving creates
   a fresh report directly; **Create report** is not a prerequisite.

If rebuilding fails, save the report with the query anyway; the result records
the failed attempt separately from the saved analysis. The report can
distinguish processing, heading extraction, ranking and reader-target issues.
The original PDF is not required just to collect this metadata.

One concrete display defect was found and corrected: a result based on a
printed Contents/index omitted its evidence suffix. `PassageSourceList` now
shows the existing translated **Printed book index** label. The new UI
regression failed before the fix, passes afterward, verifies English/German
display and confirms the unchanged source/segment/heading opening target.

The owner's latest source-reference question was incomplete. A description
of the current symptom is still needed before attributing it to this
particular MacArthur case. Issue #35 remains open. The separate
Scripture-recognition workstream
(Issue #36 / PR #48) is outside this follow-up.

## Verification

Typecheck, lint, translation consistency, the generated corpus check and the
production frontend build pass. All fifteen new regression cases pass. The
complete frontend run passes **1,312/1,314**; both failed tests pass unchanged
on one targeted rerun. They concern presenter-button focus and typing a
Scripture passage immediately after saving a note. Existing deferred focus
restoration is a plausible race, but the exact event sequence was not
instrumented. Their production paths and tests have no current diff. This is
recorded as a full run with two intermittent failures, not one entirely green
full-suite run.

Final Rust formatting and all-target locked Clippy with warnings denied pass.
The complete locked Rust suite passes **1,103 tests, zero failures, eight
existing ignored**. Its initial run found a stale template-name expectation;
only that assertion's display-name literal was corrected before the full
native checks were rerun.

[The verification record](owner-followup-20261119-verification.json) retains
both initial results, the final native checks and exact changed-input hashes.
The existing frontend chunk-size warning remains. No active workflow,
production Rust code, dependency lock or migration is changed. No Windows
build, installer, merge, release or deployment is part of this update.
