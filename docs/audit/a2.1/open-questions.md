# A2.1: open questions before any further PDFium integration

**Status:** A2.1 stays an experimental branch. There is no backend switch,
no installer integration and no production code that calls PDFium. Further
work waits until the Codex Windows Source corrections pass acceptance.
This page records what would have to be settled first.

## 1. Word spacing (`Tw`): unresolved

**What happens.** A PDF can justify a line in two ways:

* `TJ` with per-glyph kerning adjustments;
* the word-spacing operator `Tw`, which widens every single-byte space
  (code 32) by the same amount.

pdf-extract adds `Tw` to a space's advance, so the layout sees a wide gap
and the following glyph where it is drawn. PDFium's text page reports
each character's box and origin, but not the advance as drawn. Its
`FPDFText_GetCharBox` for a `Tw`-widened space gives the font's space
width. The helper therefore sees a narrow space followed by a glyph whose
origin jumps. The layout reads the jump as a column gap or as the end of a
word group, and verification suffers.

**Measured effect.** On the disclosed PDFs:

* F03: 10 848 → 6 107 verified;
* engwebu-rl2col: 7 499 → 7 307.

Text coverage is unchanged: every glyph is found. No wrong verified verse
results; the losses are refusals.

**Options, none implemented:**

1. Read the text state from the content stream (`Tw`, `Tc`, `Tz`, font,
   `Tf` size) beside PDFium. This re-implements part of a content-stream
   interpreter, the thing PDFium was meant to replace.
2. Use `FPDFText_GetCharOrigin` for each glyph and the next one, and infer
   the drawn advance from the origin difference on the same line. This
   cannot tell `Tw` from a `TJ` adjustment; both look the same. For the
   layout that may not matter (both are drawn gaps), but it changes the
   space-width statistics that the column and word rules were measured
   on. It would need a full re-measurement on the 57 PDFs.
3. Use the page-object interface (`FPDFPageObj_*`, `FPDFTextObj_*`) to walk
   text objects with their matrices. It exposes the font size and matrix
   per object, but not `Tw` directly in the pinned build.

Recommendation: option 2 is the cheapest experiment, but only after the
Windows Source acceptance, and only with a rerun of all 57 PDFs that
reports wrong verified text separately.

## 2. Windows sandboxing: required before any Windows build ships

The Linux helper is contained by a watchdog, stream checks and
`RLIMIT_AS`. On Windows only the watchdog and the stream checks exist.
Before a Windows helper is shipped, the parent (`PdfiumHelper`) must:

| Requirement | How | Status |
|---|---|---|
| Memory cap | `CreateJobObjectW`; `SetInformationJobObject` with `JOBOBJECT_EXTENDED_LIMIT_INFORMATION` (`JOB_OBJECT_LIMIT_PROCESS_MEMORY`, `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE`, `JOB_OBJECT_LIMIT_DIE_ON_UNHANDLED_EXCEPTION`); start the helper `CREATE_SUSPENDED`, `AssignProcessToJobObject`, then resume. Assigning after start leaves a window in which the helper is uncapped. | Not written |
| No console window | `CREATE_NO_WINDOW` via `std::os::windows::process::CommandExt::creation_flags`. | Not written |
| No child processes | `JOB_OBJECT_LIMIT_ACTIVE_PROCESS` = 1. | Not written |
| Least privilege (desirable) | A restricted token or an AppContainer / low integrity level, so that a compromised helper cannot write to the user's files. The helper needs only to read one temporary file and write to its standard output. | Not designed |
| Temporary file | Created in the app's own temporary directory with an ACL for the current user only; deleted when reading ends, and on the next start if a crash left it. | Linux only; Windows not checked |
| Code signing | Both `aw-pdfium-glyphs.exe` and `pdfium.dll`, with the app's certificate, built with MSVC in CI. `pdfium.dll` checked against `PDFIUM.lock` at build time; the helper loads it only by absolute path from the install directory (no DLL search path). | Not set up |

None of this can be tested in this environment (no Windows machine). It
touches the app's dependency graph (`windows` crate features). It is
deliberately left for after acceptance, by whoever owns the Windows
production profile.

## 3. Security review

**Threat.** A PDF chosen by the user, but possibly crafted by someone
else, is parsed by a large C++ library. PDFium is Chrome's PDF engine and
is fuzzed continuously, but it has a steady record of memory-safety bugs
reachable from a crafted PDF. In 2026 alone, public advisories list
several use-after-free and heap-overflow bugs in PDFium fixed in Chrome
Stable releases, for example:

* CVE-2026-6305 and CVE-2026-6306: heap overflow (Chrome 147);
* CVE-2026-9958: use-after-free (Chrome 148);
* CVE-2026-11304 to -11306 (Chrome 149);
* CVE-2026-14108: use-after-free (fixed in Chrome 150.0.7871.47);
* CVE-2026-18012 (July 2026, details restricted).

These were found by a web search from this session, through third-party
aggregators as well as Red Hat's tracker. They are not a complete or
authoritative list. The Chrome Releases blog and the Chromium issue
tracker are the authority.

**Pinned build.** `chromium/8138` (PDFium 159.0.8138.0) is newer than the
Chrome 150 branch (7871), so it should contain the fixes above. This is
an inference from the version numbers, not verified commit by commit. Any
later fix requires a new pin, a rebuild and a re-measurement.

**What the design does about it:**

* PDFium runs only in a separate helper process, never in the app. A
  crash or hang loses only that document's reading, which is refused.
* The helper opens no network connection and reads one file.
* Linux has an address-space cap. On Windows the Job Object is still to
  be written (§2).

**What it does not do:**

* A helper process limits the damage of a memory-safety bug; it does not
  prevent code execution inside the helper. Without a restricted token or
  AppContainer, code running in the helper has the user's rights. Chrome
  runs PDFium inside its renderer sandbox; this helper has no equivalent
  yet.
* There is no update path. A shipped `pdfium.dll` would need security
  updates on PDFium's cadence, roughly every Chrome Stable release, through
  the signed updater. That is an ongoing maintenance cost, not a one-off.

**Conclusion.** PDFium is acceptable as an *optional, per-document
fallback* only with:

* the Windows sandbox of §2, including least privilege;
* a documented update policy for the pinned build;
* a fresh advisory check at each pin.

It is not acceptable as the default backend on the present evidence. The
current pure-Rust path (pdf-extract and lopdf) has a much smaller
memory-safety exposure.
