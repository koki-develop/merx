# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

merx is an interpreter for programs written in Mermaid flowchart syntax. It parses `.mmd` files containing Mermaid flowchart definitions and executes them as programs. The language supports variables, arithmetic/comparison/logical operators, string concatenation (`+`), type casting, input/output, conditional branching, and escape sequences in string literals.

## Build and Run Commands

```bash
# Build the project
cargo build

# Run a Mermaid flowchart program
cargo run -- run <file.mmd>

# Run tests
cargo test

# Check code without building
cargo check

# Format code
cargo fmt

# Run clippy lints
cargo clippy
```

## Architecture

The project follows a traditional interpreter architecture: parsing, AST construction, and execution.

### Core Components

```
src/
├── main.rs          # CLI entry point (clap-based)
├── lib.rs           # Library exports (ast, parser, runtime modules)
├── grammar.pest     # PEG grammar for Mermaid flowchart syntax
├── parser/
│   ├── mod.rs       # Parser implementation using pest
│   ├── error.rs     # SyntaxError, ValidationError, AnalysisError types
│   ├── expr.rs      # Expression parser (precedence, unary, cast, primary)
│   └── validate.rs  # Semantic validation (node/edge constraints)
├── ast/
│   ├── mod.rs       # Re-exports all AST types
│   ├── flowchart.rs # Flowchart, Direction
│   ├── node.rs      # Node enum (Start, End, Process, Condition)
│   ├── edge.rs      # Edge, EdgeLabel
│   ├── expr.rs      # Expr, BinaryOp, UnaryOp, TypeName
│   └── stmt.rs      # Statement enum
└── runtime/
    ├── mod.rs       # Re-exports runtime types
    ├── value.rs     # Value enum (Int, Str, Bool)
    ├── error.rs     # RuntimeError type
    ├── env.rs       # Environment (variable storage)
    ├── eval.rs      # Expression evaluation
    ├── exec.rs      # Statement execution
    ├── interpreter.rs # Interpreter main loop
    └── test_helpers.rs # Shared test mocks (MockInputReader, MockOutputWriter) [cfg(test)]

examples/                        # Example .mmd programs (fibonacci, fizzbuzz, hello)

tests/
├── integration_tests.rs  # Integration tests for the interpreter
└── fixtures/
    ├── valid/       # Valid .mmd files for testing successful execution
    └── invalid/     # Invalid .mmd files for testing error handling

benchmarks/                      # Benchmark suite comparing merx with other languages
├── run.sh                       # Benchmark runner (requires: hyperfine, jq, merx, python3, node, ruby, go)
├── README.md                    # Generated benchmark results (Markdown)
└── programs/                    # Equivalent programs in each language
    ├── fizzbuzz/                 # FizzBuzz implementations (.mmd, .py, .js, .rb, .go)
    └── fibonacci/               # Fibonacci implementations (.mmd, .py, .js, .rb, .go)

dockerfiles/                     # Dockerfiles for pre-built binary images
web/                             # VitePress documentation site (separate from Rust codebase)
```

### Parser

- Uses **pest** with PEG grammar defined in `src/grammar.pest`
- Entry point: `parser::parse(&str) -> Result<Flowchart, AnalysisError>`
- Handles operator precedence manually in code (see `build_expr_with_precedence`)
- Validates that condition nodes have exactly one `Yes` and one `No` edge
- Validates Start and End node existence at parse time
- Validates End node has no outgoing edges
- Validates non-condition nodes have at most one outgoing edge
- Detects duplicate node definitions at parse time
- Detects undefined node references at parse time
- Validates exit code is only allowed on edges to the `End` node
- Parses edge labels with exit code syntax: `exit N`, `Yes, exit N`, `No, exit N`

### AST

- `Flowchart`: Top-level structure containing direction, nodes, and edges
- `Node`: Enum with variants `Start`, `End`, `Process { id, statements }`, `Condition { id, condition }`
- `Edge`: Connects nodes with optional labels (`Yes`, `No`, or custom) and optional `exit_code: Option<u8>`
- `Expr`: Expression tree supporting literals, variables, binary/unary ops, casts, and `input`
- `Statement`: `Println`, `Print`, `Error`, or `Assign`

### Runtime

- Entry point: `runtime::Interpreter::new(Flowchart) -> Result<Interpreter, RuntimeError>`
- `Interpreter::run()` executes the flowchart from Start to End, returning the exit code (`Result<u8, RuntimeError>`)
- `Value`: Runtime values (`Int(i64)`, `Str(String)`, `Bool(bool)`)
- `Environment`: FxHashMap-based variable storage
- `InputReader` / `OutputWriter` traits for testability (dependency injection)

## Release Workflow

The project uses `release-please` for automated releases. When a release is created, binaries are built for 5 platforms:

| Target | Runner | Notes |
|--------|--------|-------|
| `x86_64-unknown-linux-gnu` | ubuntu-latest | Native build |
| `aarch64-unknown-linux-gnu` | ubuntu-latest | Uses `cross` for cross-compilation |
| `x86_64-apple-darwin` | macos-15-intel | Intel Mac (Tier 2) |
| `aarch64-apple-darwin` | macos-latest | Apple Silicon |
| `x86_64-pc-windows-msvc` | windows-latest | Windows |

Release artifacts:
- Archives: `.tar.gz` (Unix) / `.zip` (Windows)
- SHA256 checksums for each archive
- Includes: binary, LICENSE, README.md

### Homebrew Tap

After release assets are uploaded, the `homebrew-tap` job automatically generates and pushes `merx.rb` to `koki-develop/homebrew-tap`. The formula is generated dynamically by downloading SHA256 checksums from the release and templating the Ruby file. Supported platforms for Homebrew (Windows excluded):
- `x86_64-apple-darwin`, `aarch64-apple-darwin` (macOS)
- `x86_64-unknown-linux-gnu`, `aarch64-unknown-linux-gnu` (Linux)

## Docker

- Base image: `debian:bookworm-slim` (release binaries are glibc-linked, so Alpine is not compatible)
- Multi-arch support: amd64 (`x86_64-unknown-linux-gnu`) and arm64 (`aarch64-unknown-linux-gnu`)
- Binary is downloaded from GitHub Releases and verified against SHA256 checksum
- Build: `docker build -f dockerfiles/bookworm-slim.Dockerfile --build-arg VERSION=<version> .`
- Published to `ghcr.io/koki-develop/merx` on release via the `docker` job in the release workflow
- Image tags: `<version>` (e.g., `0.1.1`), `<major>.<minor>` (e.g., `0.1`), `<major>` (e.g., `0`), `latest`

## GitHub Actions Conventions

- Use `github.token` instead of `secrets.GITHUB_TOKEN`
- Pass template variables via environment variables to prevent template injection:
  ```yaml
  env:
    TARGET: ${{ matrix.target }}
  steps:
    - run: echo "$TARGET"  # Safe
    # - run: echo "${{ matrix.target }}"  # Unsafe
  ```
- Pin actions by commit hash (e.g., `actions/checkout@de0fac2e4...`)
- Use `.github/actions/setup` for Rust toolchain setup (via mise)

