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
| Rust 1.93.0 | 0.71 | 0.65 | 0.81 | 1 |
| **merx 0.1.4** | **1** | **0.88** | **1.56** | **1.41** |
| Go 1.25.6 | 1.21 | 1.12 | 1.37 | 1.71 |
| Python 3.14.2 | 11.15 | 10.86 | 12.82 | 15.75 |
| Node.js v24.13.0 | 22.56 | 21.69 | 25.85 | 31.87 |
| Ruby 3.4.8 | 50.3 | 49.42 | 52.2 | 71.06 |

## Fibonacci (n=30)

Programs: [./programs/fibonacci/](./programs/fibonacci/)

| Language | Mean (ms) | Min (ms) | Max (ms) | Relative |
|----------|-----------|----------|----------|----------|
| Rust 1.93.0 | 0.7 | 0.65 | 0.83 | 1 |
| **merx 0.1.4** | **0.91** | **0.85** | **1.16** | **1.3** |
| Go 1.25.6 | 1.22 | 1.12 | 1.38 | 1.74 |
| Python 3.14.2 | 11.01 | 10.78 | 11.59 | 15.68 |
| Node.js v24.13.0 | 21.83 | 21.29 | 23.28 | 31.09 |
| Ruby 3.4.8 | 50.42 | 49.41 | 52.86 | 71.82 |

## GCD Sum (n=100)

Programs: [./programs/gcdsum/](./programs/gcdsum/)

| Language | Mean (ms) | Min (ms) | Max (ms) | Relative |
|----------|-----------|----------|----------|----------|
| Rust 1.93.0 | 0.84 | 0.79 | 0.93 | 1 |
| Go 1.25.6 | 1.35 | 1.22 | 1.63 | 1.6 |
| **merx 0.1.4** | **10.39** | **10.19** | **11.75** | **12.33** |
| Python 3.14.2 | 14.97 | 14.5 | 16.14 | 17.76 |
| Node.js v24.13.0 | 22.88 | 21.61 | 26.18 | 27.16 |
| Ruby 3.4.8 | 52.54 | 51.59 | 54.62 | 62.35 |

## Prime Count (n=10000)

Programs: [./programs/primecount/](./programs/primecount/)

| Language | Mean (ms) | Min (ms) | Max (ms) | Relative |
|----------|-----------|----------|----------|----------|
| Rust 1.93.0 | 0.92 | 0.86 | 1.02 | 1 |
| Go 1.25.6 | 1.46 | 1.35 | 1.6 | 1.58 |
| Node.js v24.13.0 | 22.62 | 21.55 | 24.28 | 24.46 |
| Python 3.14.2 | 23.1 | 22 | 27 | 24.97 |
| **merx 0.1.4** | **30.09** | **29.49** | **40.37** | **32.53** |
| Ruby 3.4.8 | 55.63 | 54.52 | 58.26 | 60.14 |

## Collatz Conjecture (n=10000)

Programs: [./programs/collatz/](./programs/collatz/)

| Language | Mean (ms) | Min (ms) | Max (ms) | Relative |
|----------|-----------|----------|----------|----------|
| Rust 1.93.0 | 1.57 | 1.51 | 1.67 | 1 |
| Go 1.25.6 | 2.5 | 2.34 | 2.92 | 1.59 |
| Node.js v24.13.0 | 25.35 | 24.05 | 27.24 | 16.12 |
| Ruby 3.4.8 | 92.64 | 91.65 | 95.01 | 58.9 |
| Python 3.14.2 | 106.74 | 100.99 | 133.58 | 67.87 |
| **merx 0.1.4** | **207.14** | **204.84** | **220.43** | **131.71** |

## String Concatenation (n=10000)

Programs: [./programs/strconcat/](./programs/strconcat/)

| Language | Mean (ms) | Min (ms) | Max (ms) | Relative |
|----------|-----------|----------|----------|----------|
| Rust 1.93.0 | 0.71 | 0.66 | 0.81 | 1 |
| **merx 0.1.4** | **4.54** | **4.44** | **4.82** | **6.42** |
| Go 1.25.6 | 8.51 | 7.65 | 10.65 | 12.03 |
| Python 3.14.2 | 12.85 | 12.54 | 13.76 | 18.17 |
| Node.js v24.13.0 | 22.69 | 21.42 | 24.75 | 32.08 |
| Ruby 3.4.8 | 66.13 | 64.41 | 68.37 | 93.5 |

