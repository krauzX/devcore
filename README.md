# DevCore

A local-first developer productivity toolkit for college students. Built in Rust with a TUI dashboard, system tray integration, and CLI.

## Features

### Academic Tracker
- Manage semesters, courses, grades (SGPA/CGPA)
- Track deadlines with urgency indicators
- Color-coded deadline display (red=today, yellow=this week, cyan=soon, green=later)

### Git Streak & Skills
- Track commit streaks (current, longest, total)
- 5-axis skill progression: commit hygiene, testing, documentation, code review, architecture
- XP-based leveling system

### DSA Challenges
- 1788 offline LeetCode problems (embedded at compile time)
- 5 builtin problem packs (arrays, linked lists, stacks, trees, graphs)
- Install/remove packs on demand
- LeetCode GraphQL API integration (daily challenges, problem lookup)
- 5 build-your-own projects (shell, git, HTTP server, Redis, compiler)

### TUI Dashboard
- Interactive terminal UI with Catppuccin Mocha theme
- 4 tabs: Dashboard, Academic, Git, Challenges
- Modular widget system (reusable components)
- Keyboard navigation

### System Tray
- Cloudflare-style daily dashboard
- Shows deadlines, streak, SGPA in tooltip
- Quick access to TUI

### CLI
- Full command interface for all features
- Subcommands: tui, tray, academic, git, dsa

## Installation

### From Source

```bash
git clone https://github.com/krauzX/devcore.git
cd devcore
cargo build --release
```

### Via npm (coming soon)

```bash
npm install -g @krauz/devcore
```

## Usage

### TUI Dashboard

```bash
devcore tui
```

### System Tray

```bash
devcore tray
```

### Academic Management

```bash
devcore academic list              # List semesters
devcore academic current           # Show current semester
devcore academic deadlines         # Show upcoming deadlines
devcore academic dashboard         # Show full dashboard
devcore academic course --code CS301 --name "Data Structures" --credits 4
devcore academic grade --course CS301 --exam mid --obtained 45 --total 50
devcore academic sgpa              # Show SGPA
devcore academic cgpa              # Show CGPA
```

### Git Tracking

```bash
devcore git analyze                # Analyze current repo
devcore git streak                 # Show commit streak
devcore git skills                 # Show skill progression
devcore git xp --axis testing --amount 50 --reason "wrote tests"
```

### DSA Challenges

```bash
devcore dsa list                   # List available packs
devcore dsa install arrays-easy    # Install a pack
devcore dsa problems arrays-easy   # List problems in pack
devcore dsa show arrays-easy two-sum  # Show problem details
devcore dsa browse                 # Browse all 1788 problems
devcore dsa leetcode list --difficulty easy
devcore dsa leetcode daily
devcore dsa project list
devcore dsa project install build-your-own-shell
```

## Architecture

```
devcore/
├── crates/
│   ├── core/          Shared types, config, KV storage
│   ├── academic/      Semester, course, grade, deadline
│   ├── devtrack/      Git analysis, skills, streaks
│   ├── challenges/    DSA engine, LeetCode, projects
│   └── tray/          System tray daemon
├── tui/               Interactive terminal dashboard
├── cli/               CLI entry point
└── docs/              Verification documents (not in git)
```

## Crates

| Crate | Purpose | Lines |
|-------|---------|-------|
| `devcore-core` | Shared types, config, KV storage | ~120 |
| `devcore-academic` | Semester, course, grade, deadline | ~400 |
| `devcore-devtrack` | Git analysis, skill progression, streaks | ~430 |
| `devcore-challenges` | DSA engine, LeetCode API, projects | ~1200 |
| `devcore-tray` | System tray daemon | ~140 |
| `devcore-tui` | Interactive terminal dashboard | ~800 |
| `devcore` | CLI entry point | ~500 |

## Datasets

- `leetcode_official.json` — 1788 LeetCode problems (embedded at compile time)
- `leetcode_clean.json` — 68 problems with tags/hints/description
- `project_tutorials.md` — project-based-learning tutorials
- `projectlearn.md` — ProjectLearn data

## Testing

```bash
cargo test --workspace    # Run all tests (16 tests)
cargo clippy --workspace  # Check for warnings (0 warnings)
```

## License

MIT
