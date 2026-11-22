# Harness binary builds (evaluation candidate A1.5-E3)

## Environment

```
rustc 1.97.0 (2d8144b78 2026-07-07)
cargo 1.97.0 (c980f4866 2026-06-30)
Linux 6.18.44-fc-v726 x86_64
PRETTY_NAME="Ubuntu 24.04.5 LTS"
cpus: 4
webkit2gtk-4.1 2.52.6
Tauri system libraries installed with apt exactly as in .github/workflows/ci.yml
```

## Build E2-R: the E2 harness, reproduced before any change (failing-before evidence)

Source: commit 7772d0b (E2 harness sources of 97502d2, unchanged), at /home/user/abidingword/src-tauri, fresh target directory, empty ../dist.

```
cargo test --release --locked --test real_document_editions --no-run
real	8m47.370s
user	15m8.845s
sys	0m50.551s
sha256: 800fb43c9d09e85e7c7475702ff3332ebc45a3ce44a0204220f3d64756454473
```

Identical to the E2 pin (protocol-e2.json) and to the reviewer's build A: the third reproduction at this path and toolchain.

## Build E3: the E3 harness, pinned in docs/evaluation/protocol-e3.json

Started 2026-11-22T03:15:37Z in /home/user/abidingword/src-tauri, the same target directory (incremental: the app crate unchanged since build E2-R, the harness test crate rebuilt), empty ../dist (0 files). The harness sources at the start of the build (verified unchanged afterwards and at the candidate commit):

```
15939f5de631926768b6c6ddf74c1a63ea6dd49d4eb0897aaa6e240d7b7ccc54  src-tauri/tests/real_document_editions.rs
92b1a0a8df61aaeb25e7b6f03cbd1a74137d95510a9412752e45e5d11c45c90a  src-tauri/tests/eval_support/fidelity.rs
25da0cfdf802c98a1328b0606b848d88a2b5f644bafd5c4e2ae20583e3dc95fa  src-tauri/tests/eval_support/reference.rs
f468a4b50823739ea22abb71ba9cc8a48d4a17a53911ac7d778fc7b1b4787929  src-tauri/tests/eval_support/usfm_text.rs
```

```
cargo test --release --locked --test real_document_editions --no-run
real	6m33.959s
user	7m18.390s
sys	0m10.921s
binary: target/release/deps/real_document_editions-d64932b26d8aee1f
sha256: b4f51a62a51ef5085ca12900ba575ee04cc04f0b99f0fccf88db8890a141b63d
embedded build path occurrences of /home/user/abidingword/src-tauri: 1
```

## Reproducibility: what the pin means

* The pin is **environment-specific**. The harness compiles its own path
  in (`CARGO_MANIFEST_DIR`, to find `docs/corpus`). For the same source,
  toolchain, dependency lock, system libraries and path the build has been
  reproducible in practice: E2's binary was reproduced exactly here (build
  E2-R), as the reviewer reproduced it before, and E3's clean rebuild is
  recorded below.
* The same source built at **another path gives another SHA-256** (shown for
  E2 by its author and by the reviewer: `d3cdcd77…` at another path). Not
  repeated for E3 here.
* The frontend build state (`../dist`) did not change the E2 harness binary
  (reviewer's build D). E3's pin was built with an empty `../dist`.
* An assessor who rebuilds in another environment should expect another
  SHA-256 and must have the protocol **re-pinned in a reviewed revision**,
  never overridden at scoring time (`--frozen-binary` is diagnostic only).

## Build E3-clean: the same source again, from a clean app crate

After `npm run build` had filled `../dist` (for the frontend check) and it
was emptied again, `cargo clean --release -p abiding-word` removed the app
crate's and the harness's release artefacts; then, started 2026-11-22T21:57:27Z:

```
cargo test --release --locked --test real_document_editions --no-run
real	7m21.982s
user	7m32.521s
sys	0m13.878s
sha256: b4f51a62a51ef5085ca12900ba575ee04cc04f0b99f0fccf88db8890a141b63d
```

**Identical to build E3** (the incremental one): the pin
`b4f51a62a51ef5085ca12900ba575ee04cc04f0b99f0fccf88db8890a141b63d` is
reproducible in this environment and at this path.
