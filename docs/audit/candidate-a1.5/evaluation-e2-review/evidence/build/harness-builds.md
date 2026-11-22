# Harness rebuilds by the independent reviewer

## Environment
date (UTC): 2026-11-22T01:57:32Z
rustc 1.97.0 (2d8144b78 2026-07-07)
cargo 1.97.0 (c980f4866 2026-06-30)
host: Linux 6.18.44-fc-v726 x86_64
PRETTY_NAME="Ubuntu 24.04.5 LTS"
cpus: 4
CARGO_*/RUST_* environment during the builds:
CARGO_HTTP_CAINFO=/root/.ccr/ca-bundle.crt
RUSTUP_HOME=/root/.rustup
RUST_BACKTRACE=1
Tauri system libraries: installed with apt as in ci.yml (libwebkit2gtk-4.1-dev build-essential curl wget file libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev)
webkit2gtk-4.1 2.52.6

## Build A: the reviewed source at the pinned path, fresh target directory, empty ../dist (as ci.yml)
command: cargo test --release --locked --test real_document_editions --no-run   (cwd /home/user/abidingword/src-tauri, commit 40d5d59 = 97502d2 tree for all built files)
real	7m36.192s
user	12m49.204s
sys	0m45.264s
result: 800fb43c9d09e85e7c7475702ff3332ebc45a3ce44a0204220f3d64756454473  target/release/deps/real_document_editions-d64932b26d8aee1f
pinned in docs/evaluation/protocol-e2.json: 800fb43c9d09e85e7c7475702ff3332ebc45a3ce44a0204220f3d64756454473

## Build B: the same commit (97502d2) cloned to another path, fresh target directory, empty ../dist
path: <scratchpad>/aw-clone/src-tauri
result: d3cdcd7737432825b6548402a7b6220402ce34e860a8ea881a4dce20f6e472cb
strings: the binary embeds its build path: 1 occurrence(s) of 'aw-clone/src-tauri'; build A: 1 of '/home/user/abidingword/src-tauri'

## Build C: the frozen A1.5 harness source (08c6faa:src-tauri/tests/real_document_editions.rs) in the clone, for the old-against-new comparison only
result: 1dd37d3302d7a6c9ee620a20124bd008669f741e3f08ffca8351a97f62cdab60  (not 920f7e28..., not e396328c...: another path)

## Build D: build B's clone again, after `npm run build` filled ../dist (102 files)
The app crate recompiled (`Compiling abiding-word`), the harness relinked.
result: d3cdcd7737432825b6548402a7b6220402ce34e860a8ea881a4dce20f6e472cb  (identical to build B: the frontend build state does not change the harness binary)
