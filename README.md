# DevCore

[![CI](https://github.com/krauz/devcore/actions/workflows/ci.yml/badge.svg)](https://github.com/krauz/devcore/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-1.75+-orange.svg)](https://rustup.rs)
[![Tests](https://img.shields.io/badge/tests-23%20passing-brightgreen.svg)](#testing)

**Track why code changed, what it touches, and what could break.**

Three CLI tools that solve reviewer fatigue from AI-generated code. Every commit gets a structured change receipt with intent, blast radius, and risk scoring.

```
$ shipforge receipt

┌─ Change Receipt ─────────────────────────────────────────┐
│ Commit:    a3f2b1c                                       │
│ Source:    AI (Cursor)                                    │
│ Intent:    Add rate limiting to auth endpoints            │
│ Risk:      MED (6)                                        │
├─ Blast Radius ──────────────────────────────────────────┤
│  Direct:   3   Indirect: 7                               │
└──────────────────────────────────────────────────────────┘
```

## Quick Start

```bash
# Build from source
git clone https://github.com/krauz/devcore.git && cd devcore
cargo build --release

# Initialize in your project
cd your-project && shipforge init

# Generate a receipt for your last commit
shipforge receipt
```

## The Problem

Developers using AI coding agents ship faster but **lose all context**. Two weeks later, something breaks with zero trail of the reasoning.

> *"22.7% of AI-introduced issues still survive at the latest version."* — arXiv:2603.28592

## Tools

| Tool | Purpose | Example |
|------|---------|---------|
| **ShipForge** | Generate change receipts | `shipforge receipt` |
| **CodeTrail** | Query receipts, find hotspots | `codetrail hotspots` |
| **DevPulse** | Track workflow time | `devpulse report --period week` |

## ShipForge

```bash
shipforge receipt                        # Receipt for HEAD
shipforge receipt -c a3f2b1c             # Receipt for specific commit
shipforge log                            # List all receipts
shipforge log --ai-only                  # Only AI commits
shipforge explain src/auth.rs            # Why this file exists
shipforge blast src/middleware.rs        # What breaks if I change it
shipforge show a3f2b1c --format json     # JSON output
```

**Risk scoring:** 0-3 LOW | 4-6 MEDIUM | 7-8 HIGH | 9-10 CRITICAL

## CodeTrail

```bash
codetrail history src/auth.rs            # Change history for a file
codetrail ai-log                         # All AI-generated receipts
codetrail ai-log --source cursor         # Filter by AI tool
codetrail risk                           # Project risk summary
codetrail hotspots                       # Most depended-upon files
codetrail blast src/middleware.rs        # Blast radius analysis
codetrail explain src/auth.rs            # Full context
```

## DevPulse

```bash
devpulse event -k coding -m 45 -d "Built auth"    # Record time
devpulse report --period week                       # Weekly report
devpulse chart --period day                         # Hourly chart
devpulse suggest                                    # Productivity tips
```

**Categories:** coding, review, build, test, search, meeting, ai

## Configuration

Create `.devcore/config.toml`:

```toml
source_extensions = ["ts", "tsx", "js", "jsx", "rs", "go", "py"]
exclude_dirs = ["node_modules", "target", ".git", ".devcore"]
max_file_size_bytes = 10485760  # 10MB
```

## Architecture

```
devcore/
├── crates/
│   ├── core/            # Shared: git analysis, storage, AI detection, blast radius
│   ├── shipforge/       # CLI: change receipt generator
│   ├── codetrail/       # CLI: receipt querying + hotspots
│   └── devpulse/        # CLI: workflow analyzer
├── .github/workflows/   # CI/CD (check, test, clippy, fmt, release)
└── Cargo.toml           # Workspace root
```

## Testing

```bash
cargo test --all            # 23 tests (14 unit + 9 integration)
cargo clippy -- -D warnings # Zero warnings
cargo fmt --check           # Clean formatting
```

## CI/CD

GitHub Actions on every push/PR: check, test, clippy, fmt. Release job builds for Linux, macOS (x86+ARM), Windows on tag push.

## Security

- Parameterized SQL queries (no injection)
- LIKE patterns escaped with `ESCAPE '\\'`
- Mutex poisoning handled gracefully
- Symlinks not followed in filesystem traversal
- File size limits prevent memory exhaustion

## License

[MIT](LICENSE)
