#!/bin/bash
set -e

AKAME=$(ping -qc1 akame.lan > /dev/null && echo akame || echo akame-tun)

cargo build --release --target x86_64-unknown-linux-musl
scp target/x86_64-unknown-linux-musl/release/break-gt $AKAME:.
