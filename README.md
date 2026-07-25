# DevCore

A local-first developer productivity toolkit for college students. Built in Rust with a fully interactive TUI dashboard, system tray, and CLI.

## Features

### TUI Dashboard (fully interactive)
- **Dashboard tab** — semester info, SGPA gauge, deadlines, streak, quick actions
- **Academic tab** — add courses, deadlines, grades; set current semester; view SGPA/CGPA
- **Git tab** — repo analysis (commits, insertions, deletions, languages), streak, skill progression, add XP
- **Challenges tab** — browse 1788 problems, install/remove packs, filter by difficulty, view problem details, browse projects

### Academic Tracker
- Manage semesters, courses, grades (SGPA/CGPA)
- Track deadlines with urgency colors (red=today, yellow=this week, cyan=soon, green=later)
- Add courses, deadlines, grades interactively in TUI

### Git Streak & Skills
- Track commit streaks (current, longest, total)
- 5-axis skill progression with XP system
- Repository analysis (commits, insertions, deletions, languages)

### DSA Challenges
- 1788 offline LeetCode problems (embedded at compile time)
- 5 builtin problem packs (arrays, linked lists, stacks, trees, graphs)
- Install/remove packs on demand
- LeetCode GraphQL API integration
- 10 build-your-own projects (shell, git, HTTP server, Redis, compiler, text-editor, interpreter, database, regex, rustlings)

### System Tray
- Cloudflare-style daily dashboard
- Shows deadlines, streak, SGPA in tooltip

## Installation

```bash
git clone https://github.com/krauzX/devcore.git
cd devcore
cargo build --release
```

## Usage

### TUI (recommended)
```bash
devcore tui
```

### Keyboard Shortcuts
| Key | Tab | Action |
|-----|-----|--------|
| 1-4 | Any | Switch tabs |
| Tab | Any | Next tab |
| q/Esc | Any | Quit |
| c | Academic | Add course |
| d | Academic | Add deadline |
| g | Academic | Add grade |
| s | Academic | Set current semester |
| x | Git | Add XP to skill |
| i | Challenges | Install pack |
| r | Challenges | Remove pack |
| Enter | Challenges | View problem detail |
| e/m/h/a | Challenges | Filter Easy/Medium/Hard/All |
| o | Challenges | Toggle projects view |
| n/p | Challenges | Next/prev page |
| Up/Down | Any | Navigate items |

### CLI
```bash
devcore academic list / current / deadlines / dashboard / grade / course / sgpa / cgpa
devcore git analyze / streak / skills / xp
devcore dsa list / install / remove / browse / leetcode / project
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

| Crate | Purpose |
|-------|---------|
| `devcore-core` | Shared types, config, KV storage |
| `devcore-academic` | Semester, course, grade, deadline |
| `devcore-devtrack` | Git analysis, skill progression, streaks |
| `devcore-challenges` | DSA engine, LeetCode API, projects |
| `devcore-tray` | System tray daemon |
| `devcore-tui` | Interactive terminal dashboard |
| `devcore` | CLI entry point |

## Testing

```bash
cargo test --workspace    # 6 integration tests
cargo clippy --workspace  # 0 warnings
```

## License

MIT
