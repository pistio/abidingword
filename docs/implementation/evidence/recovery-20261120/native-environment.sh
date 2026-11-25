#!/bin/bash
export CARGO_HOME=/workspace/toolchains/cargo
export RUSTUP_HOME=/workspace/toolchains/rustup
export PATH=/workspace/toolchains/cargo/bin:$PATH
export PKG_CONFIG_PATH=/workspace/system-deps/root/usr/lib/x86_64-linux-gnu/pkgconfig:/workspace/system-deps/root/usr/share/pkgconfig
export LIBRARY_PATH=/workspace/system-deps/root/usr/lib/x86_64-linux-gnu
export LD_LIBRARY_PATH=/workspace/system-deps/root/usr/lib/x86_64-linux-gnu
export CARGO_PROFILE_DEV_DEBUG=0
export CARGO_PROFILE_TEST_DEBUG=0
export CARGO_TARGET_DIR=/workspace/abidingword/src-tauri/target
export XDG_DATA_HOME=/workspace/work/daily-recovery/work/xdg/data
export XDG_CONFIG_HOME=/workspace/work/daily-recovery/work/xdg/config
export XDG_CACHE_HOME=/workspace/work/daily-recovery/work/xdg/cache
exec "$@"
