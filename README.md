# DevCore

**Track why code changed, what it touches, and what could break.**

DevCore is a Rust workspace with three CLI tools that solve the #1 developer frustration of 2026: **reviewer fatigue** from AI-generated code. Every commit gets a structured change receipt with intent, blast radius, and risk scoring.

```
┌─ Change Receipt ─────────────────────────────────────────┐
│ Commit:    a3f2b1c9e4f7                                   │
│ Timestamp: 2026-06-29 14:32:00 UTC                        │
│ Source:    AI (Cursor)                                    │
│ Intent:    Add rate limiting to auth endpoints            │
│ Risk:      MED (6)                                        │
├─ Blast Radius ──────────────────────────────────────────┤
│  Direct:   3                                              │
│  Indirect: 7                                              │
├─ Risks ─────────────────────────────────────────────────┤
│  [Medium] 2 downstream file(s) depend on this            │
└──────────────────────────────────────────────────────────┘
```

## The Problem

In 2026, developers using AI coding agents (Cursor, Copilot, Claude Code) ship code faster than ever — but **lose all context** about why decisions were made. Two weeks later, something breaks and there's zero trail of the reasoning behind the changes.

> *"My entire job has shifted to acting as a high-level systems architect and a suspicious QA tester."*
> — r/cursor, 99 upvotes

> *"Context inversion is the actual problem — writing code yourself means you had the 'why' before the 'what'."*
> — r/cursor

> *"22.7% of AI-introduced issues still survive at the latest version of the repository."*
> — arXiv:2603.28592 (302,600 AI commits studied)

## The Solution: Three Tools, One System

| Tool | What it does | When to use |
|------|-------------|-------------|
| **ShipForge** | Generates structured change receipts at commit time | Every commit |
| **CodeTrail** | Queries receipts, shows history, finds hotspots | Before modifying files |
| **DevPulse** | Tracks where time actually goes, detects bottlenecks | Daily workflow review |

All three share a core library (`devcore-core`) with git analysis, SQLite storage, AI detection, and blast radius computation.

## Quick Start

### Install from source

```bash
git clone https://github.com/krauz/devcore.git
cd devcore
cargo build --release
```

Binaries are in `target/release/`:
- `shipforge.exe`
- `codetrail.exe`
- `devpulse.exe`

### Initialize a project

```bash
cd your-project
shipforge init
```

This creates `.devcore/devcore.db` (SQLite) and `.devcore/config.toml`.

## ShipForge — Change Receipt Generator

Every commit gets a structured receipt with intent, risks, blast radius, and AI source detection.

```bash
# Generate receipt for HEAD commit
shipforge receipt

# Generate for a specific commit
shipforge receipt -c a3f2b1c

# List all receipts
shipforge log

# Only AI-generated commits
shipforge log --ai-only

# Why does this file exist?
shipforge explain src/auth.rs

# What breaks if I change this?
shipforge blast src/middleware.rs
```

### How it works

1. Parses the git commit (diff, author, message)
2. Detects AI source (Cursor, Copilot, Claude, Windsurf, Aider)
3. Builds blast radius graph (what imports this file?)
4. Scores risk (0-10) based on change size, AI source, and blast radius
5. Stores receipt in SQLite for querying

### Risk scoring

| Score | Level | Meaning |
|-------|-------|---------|
| 0-3 | LOW | Small change, few dependents |
| 4-6 | MEDIUM | Moderate change or AI-generated |
| 7-8 | HIGH | Large change, many dependents |
| 9-10 | CRITICAL | Massive change, critical path |

## CodeTrail — Change Receipt Querying

Query stored receipts, find project hotspots, and understand file history.

```bash
# Change history for a file
codetrail history src/auth.rs

# All AI-generated receipts
codetrail ai-log

# Filter by source
codetrail ai-log --source cursor

# Project risk summary
codetrail risk

# Files with highest blast radius (most depended upon)
codetrail hotspots

# Detailed blast radius for a file
codetrail blast src/middleware.rs

# Full context: history + dependents + risks
codetrail explain src/auth.rs
```

### Hotspot detection

```
Hotspots (files with highest blast radius)
======================================================================
  10  src/middleware.rs           ██████████
   7  src/models.rs              ███████
   4  src/config.rs              ████
```

These are the files where changes have the most downstream impact. Change them with extra caution.

## DevPulse — Developer Workflow Analyzer

Track where time actually goes. Detect bottlenecks. Get suggestions.

```bash
# Record time blocks
devpulse event -k coding -m 45 -d "Built auth module"
devpulse event -k review -m 20 -d "Reviewed PR #42"
devpulse event -k ai -m 30 -d "Cursor generated tests"

# Weekly report (auto-detects from git if no events recorded)
devpulse report --period week

# Visual hourly chart
devpulse chart --period day

# Get productivity suggestions
devpulse suggest
```

### Report output

```
DevPulse Report: Last 7 days
============================================================
Total time tracked: 12.5h (750 minutes)

CATEGORY                 TIME     PCT  BAR
------------------------------------------------------------
Human_Coding            5.2h   41.6%  ████████████████████
AI_Coding               3.1h   24.8%  ████████████
Review                  2.8h   22.4%  ███████████
Build/Test              1.4h   11.2%  █████

⚠ Bottleneck: Human_Coding takes 42% of your time
  Suggestion: Use AI for boilerplate, focus on architecture
```

## Configuration

Create `.devcore/config.toml` in your project root:

```toml
# File extensions to analyze for blast radius
source_extensions = ["ts", "tsx", "js", "jsx", "rs", "go", "py", "java", "kt"]

# Directories to exclude from analysis
exclude_dirs = ["node_modules", "target", ".git", ".devcore", "dist", "build"]

# Maximum file size to analyze (bytes)
max_file_size_bytes = 10485760  # 10MB
```

## Architecture

```
devcore/
├── crates/
│   ├── core/                    # Shared library
│   │   └── src/
│   │       ├── lib.rs           # Module exports
│   │       ├── models.rs        # Data types (ChangeReceipt, BlastRadius, etc.)
│   │       ├── git_analyzer.rs  # Git commit parsing, diff stats
│   │       ├── storage.rs       # SQLite persistence
│   │       ├── ai_detector.rs   # AI source detection (Cursor, Copilot, etc.)
│   │       ├── blast_radius.rs  # Import graph + dependency analysis
│   │       └── config.rs        # Project configuration
│   ├── shipforge/               # CLI: change receipt generator
│   ├── codetrail/               # CLI: receipt querying + hotspots
│   └── devpulse/                # CLI: workflow analyzer
├── Cargo.toml                   # Workspace root
└── README.md
```

### Key design decisions

- **Rust** for single-binary distribution and memory safety
- **SQLite** for zero-config persistent storage
- **git2** for native git operations (no shell out to `git`)
- **walkdir** with symlink protection for safe filesystem traversal
- **clap** for CLI argument parsing with derive macros
- **once_cell** for cached regex compilation

## Testing

```bash
# Run all tests
cargo test --all

# Run with output
cargo test --all -- --nocapture

# Run specific test
cargo test -p devcore-core::blast_radius_test
```

**14 tests** covering:
- AI detection (8 tests): Cursor, Copilot, Claude, bot detection, intent extraction
- Blast radius (4 tests): direct/indirect dependents, isolated files, sorted output
- Storage (2 tests): save/retrieve receipts, ordering

## CI/CD

GitHub Actions workflow runs on every push and PR:

- **Check** — `cargo check --all-targets`
- **Test** — `cargo test --all`
- **Clippy** — `cargo clippy --all-targets -- -D warnings`
- **Format** — `cargo fmt --check`

## Security

- All SQL queries parameterized (no injection)
- LIKE patterns escaped with `ESCAPE '\\'`
- Mutex poisoning handled gracefully (no panics)
- Symlinks not followed in filesystem traversal
- File size limits prevent memory exhaustion
- No hardcoded paths or credentials

## Research Backing

DevCore is grounded in 2026 research on AI code maintenance:

- **arXiv:2603.28592** — "Debt Behind the AI Boom": 302,600 AI commits studied, 22.7% of issues survive long-term
- **arXiv:2603.27524** — "Safer Builders, Risky Maintainers": agents introduce 9.35% breaking changes during maintenance
- **arXiv:2602.20206** — "Epistemic Debt": 77% failure rate on maintenance tasks after unrestricted AI use

## License

MIT
