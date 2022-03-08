#!/bin/bash
set -e

cargo build --release --target x86_64-unknown-linux-musl
scp target/x86_64-unknown-linux-musl/release/break-gt akame-tun:.
