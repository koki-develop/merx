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
| C version | 0.51 | 0.46 | 0.72 | 1 |
| Rust 1.93.0 | 0.73 | 0.68 | 1.12 | 1.43 |
| **merx 0.1.4** | **0.97** | **0.9** | **1.09** | **1.91** |
| Go 1.25.6 | 1.24 | 1.12 | 1.44 | 2.43 |
| Python 3.14.2 | 11.22 | 10.78 | 15.34 | 21.93 |
| Node.js v24.13.0 | 23.68 | 22.18 | 26.98 | 46.29 |
| Ruby 3.4.8 | 51.78 | 49.78 | 55.22 | 101.21 |

## Fibonacci (n=30)

Programs: [./programs/fibonacci/](./programs/fibonacci/)

| Language | Mean (ms) | Min (ms) | Max (ms) | Relative |
|----------|-----------|----------|----------|----------|
| C version | 0.5 | 0.45 | 0.59 | 1 |
| Rust 1.93.0 | 0.7 | 0.64 | 0.97 | 1.42 |
| **merx 0.1.4** | **0.89** | **0.83** | **1.03** | **1.8** |
| Go 1.25.6 | 1.18 | 1.06 | 1.46 | 2.38 |
| Python 3.14.2 | 11.26 | 10.79 | 12.56 | 22.72 |
| Node.js v24.13.0 | 22.96 | 21.33 | 24.82 | 46.33 |
| Ruby 3.4.8 | 51.85 | 50.2 | 56.98 | 104.61 |

## GCD Sum (n=100)

Programs: [./programs/gcdsum/](./programs/gcdsum/)

| Language | Mean (ms) | Min (ms) | Max (ms) | Relative |
|----------|-----------|----------|----------|----------|
| C version | 0.65 | 0.6 | 0.74 | 1 |
| Rust 1.93.0 | 0.85 | 0.78 | 0.96 | 1.31 |
| Go 1.25.6 | 1.36 | 1.26 | 1.68 | 2.09 |
| **merx 0.1.4** | **10.4** | **10.2** | **10.96** | **16.02** |
| Python 3.14.2 | 15.48 | 14.44 | 22.85 | 23.84 |
| Node.js v24.13.0 | 23.63 | 21.94 | 27.46 | 36.37 |
| Ruby 3.4.8 | 53.42 | 51.77 | 59.3 | 82.25 |

## Prime Count (n=10000)

Programs: [./programs/primecount/](./programs/primecount/)

| Language | Mean (ms) | Min (ms) | Max (ms) | Relative |
|----------|-----------|----------|----------|----------|
| C version | 0.76 | 0.7 | 0.87 | 1 |
| Rust 1.93.0 | 0.93 | 0.88 | 1.14 | 1.23 |
| Go 1.25.6 | 1.47 | 1.36 | 1.57 | 1.94 |
| Python 3.14.2 | 23.04 | 21.85 | 28.67 | 30.29 |
| Node.js v24.13.0 | 24.02 | 22.62 | 25.39 | 31.59 |
| **merx 0.1.4** | **30.04** | **29.44** | **31.98** | **39.5** |
| Ruby 3.4.8 | 56.98 | 54.82 | 61.35 | 74.93 |

## Collatz Conjecture (n=10000)

Programs: [./programs/collatz/](./programs/collatz/)

| Language | Mean (ms) | Min (ms) | Max (ms) | Relative |
|----------|-----------|----------|----------|----------|
| C version | 1.36 | 1.31 | 1.45 | 1 |
| Rust 1.93.0 | 1.56 | 1.5 | 1.69 | 1.15 |
| Go 1.25.6 | 2.45 | 2.28 | 2.67 | 1.8 |
| Node.js v24.13.0 | 25.93 | 23.88 | 28.75 | 19.08 |
| Ruby 3.4.8 | 93.58 | 91.49 | 96.48 | 68.84 |
| Python 3.14.2 | 105.86 | 100.61 | 119.1 | 77.87 |
| **merx 0.1.4** | **207.83** | **204.66** | **229.1** | **152.89** |

## String Concatenation (n=10000)

Programs: [./programs/strconcat/](./programs/strconcat/)

| Language | Mean (ms) | Min (ms) | Max (ms) | Relative |
|----------|-----------|----------|----------|----------|
| C version | 0.57 | 0.53 | 0.65 | 1 |
| Rust 1.93.0 | 0.72 | 0.66 | 0.81 | 1.26 |
| **merx 0.1.4** | **4.57** | **4.45** | **5.19** | **7.99** |
| Go 1.25.6 | 8.3 | 7.18 | 10.54 | 14.51 |
| Python 3.14.2 | 12.97 | 12.55 | 14.43 | 22.67 |
| Node.js v24.13.0 | 22.48 | 21.38 | 24.87 | 39.31 |
| Ruby 3.4.8 | 66.53 | 64.68 | 70.71 | 116.32 |

