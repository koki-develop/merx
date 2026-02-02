# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

merx is an interpreter for programs written in Mermaid flowchart syntax. It parses `.mmd` files containing Mermaid flowchart definitions and executes them as programs. The language supports variables, arithmetic/comparison/logical operators, type casting, input/output, and conditional branching.

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

The project follows a traditional interpreter architecture with parsing and AST construction. Execution is not yet implemented.

### Core Components

```
src/
├── main.rs          # CLI entry point (clap-based)
├── lib.rs           # Library exports (ast, parser modules)
├── grammar.pest     # PEG grammar for Mermaid flowchart syntax
├── parser/
│   ├── mod.rs       # Parser implementation using pest
│   └── error.rs     # ParseError type
└── ast/
    ├── mod.rs       # Re-exports all AST types
    ├── flowchart.rs # Flowchart, Direction
    ├── node.rs      # Node enum (Start, End, Process, Condition)
    ├── edge.rs      # Edge, EdgeLabel
    ├── expr.rs      # Expr, BinaryOp, UnaryOp, TypeName
    └── stmt.rs      # Statement enum
```

### Parser

- Uses **pest** with PEG grammar defined in `src/grammar.pest`
- Entry point: `parser::parse(&str) -> Result<Flowchart, ParseError>`
- Handles operator precedence manually in code (see `build_expr_with_precedence`)
- Validates that condition nodes have exactly one `Yes` and one `No` edge

### AST

- `Flowchart`: Top-level structure containing direction, nodes, and edges
- `Node`: Enum with variants `Start`, `End`, `Process { id, statements }`, `Condition { id, condition }`
- `Edge`: Connects nodes with optional labels (`Yes`, `No`, or custom)
- `Expr`: Expression tree supporting literals, variables, binary/unary ops, casts, and `input`
- `Statement`: `Print`, `Error`, or `Assign`

All AST types derive `Serialize` for JSON output (currently the only output format).

