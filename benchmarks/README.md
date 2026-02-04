# merx Benchmarks

## System Information

- **OS**: Linux 6.11.0-1018-azure x86_64
- **CPU**: AMD EPYC 7763 64-Core Processor
- **Date**: 2026-02-04

## Configuration

- **Warmup runs**: 5
- **Benchmark runs**: 100
- **Tool**: [hyperfine](https://github.com/sharkdp/hyperfine)

## FizzBuzz (n=1..100)

Programs: [./programs/fizzbuzz/](./programs/fizzbuzz/)

| Language | Mean (ms) | Min (ms) | Max (ms) | Relative |
|----------|-----------|----------|----------|----------|
| Rust 1.93.0 | 0.72 | 0.66 | 0.86 | 1 |
| **merx 0.1.3** | **0.98** | **0.92** | **1.23** | **1.36** |
| Go 1.25.6 | 1.25 | 1.11 | 1.44 | 1.72 |
| Python 3.14.2 | 10.95 | 10.76 | 11.64 | 15.14 |
| Node.js v24.13.0 | 22.9 | 21.61 | 25.24 | 31.66 |
| Ruby 3.4.8 | 52.63 | 50.2 | 59.3 | 72.76 |

## Fibonacci (n=30)

Programs: [./programs/fibonacci/](./programs/fibonacci/)

| Language | Mean (ms) | Min (ms) | Max (ms) | Relative |
|----------|-----------|----------|----------|----------|
| Rust 1.93.0 | 0.73 | 0.64 | 1.2 | 1 |
| **merx 0.1.3** | **0.92** | **0.83** | **1.09** | **1.26** |
| Go 1.25.6 | 1.15 | 1.02 | 1.34 | 1.58 |
| Python 3.14.2 | 11.34 | 10.87 | 12.17 | 15.5 |
| Node.js v24.13.0 | 22.83 | 21.45 | 24.79 | 31.2 |
| Ruby 3.4.8 | 51.36 | 49.92 | 69.97 | 70.21 |

## GCD Sum (n=100)

Programs: [./programs/gcdsum/](./programs/gcdsum/)

| Language | Mean (ms) | Min (ms) | Max (ms) | Relative |
|----------|-----------|----------|----------|----------|
| Rust 1.93.0 | 0.72 | 0.65 | 0.89 | 1 |
| Go 1.25.6 | 1.21 | 1.1 | 1.35 | 1.68 |
| Python 3.14.2 | 14.73 | 14.28 | 16.46 | 20.44 |
| **merx 0.1.3** | **18.53** | **18.34** | **19.41** | **25.71** |
| Node.js v24.13.0 | 23.42 | 21.75 | 26 | 32.49 |
| Ruby 3.4.8 | 54.84 | 52.41 | 73.84 | 76.08 |

