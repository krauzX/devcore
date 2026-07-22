# DevCore

A local-first developer productivity toolkit for college students. Built in Rust with a TUI dashboard, system tray integration, and CLI.

## Features

- **Academic Tracker** — manage semesters, courses, grades (SGPA/CGPA), and deadlines
- **Git Streak & Skills** — track commit streaks and skill progression across 5 engineering axes
- **DSA Challenges** — install/remove problem packs, solve problems with hints and test cases
- **LeetCode Integration** — fetch problems, daily challenges, and problem details via GraphQL API
- **Project-Based Learning** — build-your-own projects (shell, git, HTTP server, Redis, compiler)
- **TUI Dashboard** — interactive terminal UI with Catppuccin Mocha theme
- **System Tray** — background monitoring with quick access menu
- **CLI** — full command-line interface for all features

## Build

```bash
cargo build --release
```

## Usage

```bash
devcore tui              # Launch TUI dashboard
devcore tray             # Start system tray daemon

devcore academic list    # List semesters
devcore academic current # Show current semester
devcore academic deadlines # Show upcoming deadlines

devcore git analyze      # Analyze current repo
devcore git streak       # Show commit streak
devcore git skills       # Show skill progression

devcore dsa list         # List available DSA packs
devcore dsa install arrays-easy  # Install a pack
devcore dsa leetcode list --difficulty easy
devcore dsa leetcode daily
devcore dsa project list
devcore dsa project install build-your-own-shell
```

## Crates

| Crate | Purpose |
|-------|---------|
| `devcore-core` | Shared types, config, KV storage |
| `devcore-academic` | Semester, course, grade, deadline management |
| `devcore-devtrack` | Git analysis, skill progression, streaks |
| `devcore-challenges` | DSA problem engine, LeetCode API, project-based learning |
| `devcore-tray` | System tray daemon |
| `devcore-tui` | Interactive terminal dashboard |
| `devcore` | CLI entry point |

## License

MIT
