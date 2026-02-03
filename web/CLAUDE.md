# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Overview

This is the documentation website for [merx](https://github.com/koki-develop/merx), built with VitePress. It is deployed to `https://koki-develop.github.io/merx/`.

## Commands

```bash
# Install dependencies (uses Bun as package manager)
bun install

# Development server with hot reload
bun run docs:dev

# Production build (output: .vitepress/dist/)
bun run docs:build

# Preview built site locally
bun run docs:preview

# Format code
bunx prettier --write .
```

## Architecture

Pure documentation site with no custom components or scripts. All content is Markdown-based.

- `.vitepress/config.mts` - VitePress configuration (base path `/merx/`, nav, sidebar, mermaid plugin)
- `index.md` - Home page (VitePress hero layout)
- `getting-started/` - Installation and quick start guides
- `guide/` - Language reference (program structure, nodes/edges, built-in functions, variables/types, operators, control flow)
- `examples.md` - Executable program examples (Fibonacci, FizzBuzz)

## Conventions

- Package manager is **Bun** with exact version pinning (`bunfig.toml: exact = true`)
- Mermaid diagrams are rendered via `vitepress-plugin-mermaid` - use ` ```mermaid ` code blocks for rendered diagrams and ` ```mmd ` for raw source examples
- Info/warning boxes use VitePress syntax: `::: info` / `::: warning`
- Sidebar order is defined manually in `.vitepress/config.mts`, not auto-generated from directory structure
