#!/bin/sh
export RUSTUP_HOME=/Volumes/Samsung-SSD/rust/.rustup
export CARGO_HOME=/Volumes/Samsung-SSD/rust/.cargo
export PATH="$CARGO_HOME/bin:$PATH"
cd /Volumes/Samsung-SSD/developer/VSCodeProjects/nomic || exit 1
cargo fetch > /tmp/nomic-fetch.log 2>&1
echo "fetch_exit=$?" >> /tmp/nomic-fetch.log
cargo check > /tmp/nomic-check.log 2>&1
echo "check_exit=$?" >> /tmp/nomic-check.log
echo done > /tmp/nomic-build.done
