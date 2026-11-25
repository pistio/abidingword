# Harness builds (independent review of A1.5-E3), 2026-11-25T19:50:33Z

rustc: rustc 1.97.0 (2d8144b78 2026-07-07)
cargo: cargo 1.97.0 (c980f4866 2026-06-30)
webkit2gtk-4.1: 2.52.6
os: Ubuntu 24.04.5 LTS Linux 6.18.44-fc-v880 x86_64
cpus: 4

## A. pinned path, fresh target directory (new container), empty ../dist
command: cargo test --release --locked --test real_document_editions --no-run (in /home/user/abidingword/src-tauri, checkout 9640aea)
real	8m14.239s
user	14m12.768s
sys	0m49.734s
b4f51a62a51ef5085ca12900ba575ee04cc04f0b99f0fccf88db8890a141b63d  target/release/deps/real_document_editions-d64932b26d8aee1f
embedded /home/user/abidingword/src-tauri occurrences: 1
=> identical to the protocol pin b4f51a62a51ef5085ca12900ba575ee04cc04f0b99f0fccf88db8890a141b63d

## B. same commit at another path (git worktree at /tmp/claude-0/elsewhere/abidingword), fresh target, empty ../dist, --offline
real	8m17.301s
user	14m18.041s
sys	0m48.925s
8cab9ddd24332ef21056e5b3d25bfeebead65b5444358e888387515bbf55affb  target/release/deps/real_document_editions-d64932b26d8aee1f
embedded own path occurrences: 1; pinned path occurrences: 0; same size (7508056 bytes); bytes differing from A: 808410
=> path-dependent, as E3 states
