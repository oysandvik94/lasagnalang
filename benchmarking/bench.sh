#!/usr/bin/env bash

hyperfine --warmup 3 'cargo run --release -- benchmarking/bench_fib.las'
