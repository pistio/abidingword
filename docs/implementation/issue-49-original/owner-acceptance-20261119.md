# Original-view bottom border — owner acceptance, 19 November 2026

The owner reports that the bottom-border problem no longer recurs and
considers it solved. This closes the reported Issue #49 symptom and releases
the hold on source-reference work. PR #50 remains draft; this acceptance does
not merge it or start another Windows build.

The report follows delivery of the normal Windows installer for application
source `539b54d65c2cecf010cd1573b8bd51c91ac640bd`. The successful build and
verified package identities are retained in
[the 18 November build report](installer-build-20261118.md). The earlier
[native comparison](native-acceptance-20261118.md) independently measured the
baseline shell scrolling by 206px and the corrected EPUB interaction keeping
all three pane bottoms at the native client bottom at 100% scaling.

This new acceptance is the owner's observation of their current application.
No new executable hash, screenshots or geometry archive accompanied the
message. It is not a new automated run or a claim about every file format,
monitor, resize path or Windows scaling setting. The prior evidence remains
unchanged and retains its original scope.

The same owner message requests removal of POZETEK from every non-German
interface and resumption of source-reference work. Those follow-ups are
separate from the accepted reader-containment correction. The installed
`539b54d` package predates the remaining terminology cleanup.
