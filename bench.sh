#!/bin/bash
set -euo pipefail

cargo build -r --workspace --examples

cd target/release/examples
hyperfine -N --warmup 100 \
	-L cmd baseline,boxed,boxed_btree,boxed_sorted,boxed_vec,clap,tree \
	'./{cmd} twenty j asdf'
