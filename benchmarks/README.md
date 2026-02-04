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
| **merx 0.1.2** | **1.07** | **1** | **1.2** | **1** |
| Go 1.25.6 | 1.28 | 1.15 | 1.48 | 1.19 |
| Python 3.14.2 | 11.29 | 10.94 | 11.9 | 10.52 |
| Node.js v24.13.0 | 23.25 | 22.66 | 24.38 | 21.68 |
| ruby 3.4.8 | 52.38 | 50.51 | 55.16 | 48.85 |

## Fibonacci (n=30)

Programs: [./programs/fibonacci/](./programs/fibonacci/)

| Language | Mean (ms) | Min (ms) | Max (ms) | Relative |
|----------|-----------|----------|----------|----------|
| **merx 0.1.2** | **0.93** | **0.87** | **1.08** | **1** |
| Go 1.25.6 | 1.24 | 1.08 | 1.71 | 1.33 |
| Python 3.14.2 | 11.15 | 10.83 | 11.99 | 11.99 |
| Node.js v24.13.0 | 23.09 | 22.11 | 24.95 | 24.85 |
| ruby 3.4.8 | 51.39 | 50.17 | 54.99 | 55.3 |

