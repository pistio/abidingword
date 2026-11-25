# Robinson–Pierpont Byzantine Textform 2018 (grcmt) — source and licence

| | |
|---|---|
| Text | Robinson–Pierpont Byzantine Textform 2018, the 27 books of the New Testament in Greek (eBible.org id `grcmt`) |
| File | `grcmt_usfm.zip` (USFM, one file per book), exactly as downloaded |
| Downloaded from | https://ebible.org/Scriptures/grcmt_usfm.zip |
| Details page | https://ebible.org/find/details.php?id=grcmt |
| Downloaded on | 2026-10-25 |
| SHA-256 | `fa1a59e1545ee404b2b5306fd5e57e2b44617e3c88bd0da60d44c3bd9ae496d6` |
| Licence | Public Domain |

## Licence statement

From `copr.htm` inside the archive (kept alongside it, unchanged):

> The Greek New Testament according to the Byzantine Text form. Robinson-Pierpont 2018 edition. Public Domain. … Translation by: Maurice A. Robinson and William G. Pierpont. This public domain Bible translation is brought to you courtesy of eBible.org.

Upstream: github.com/byztxt (byztxt v3.3.2, `27a45ff1b7be6c17ccbfeac414f3f55732ae8e28`), README: "All the code and text contained in this folder is in the Public Domain".

The audit behind this entry (every edition considered, and why some were
left out) is in `docs/original-language-catalogue-v0.24.md`.

## What it contains

Strong's numbers and Robinson's morphology codes on every word (140,121 words). The text is **unaccented and lower case**, with a medial sigma at the end of words (λογοσ): this comes from Robinson's parsed source and is shown as it is. Byzantine versification: Romans 14:24–26 holds the doxology and Romans 16 ends at verse 24.

Only the New Testament: Abiding Word reports the 39 Old Testament books as
absent, and passages there open in another Bible.

## How Abiding Word uses it

The archive is shipped unmodified as an application resource. It is not
installed automatically: **Add a Bible… → Available Bibles** shows it, and
adding it goes through the same review screen as any Bible file, after which
it works offline. eBible.org rebuilds its zips from time to time, so the
checksum above identifies this copy, not the current download.
