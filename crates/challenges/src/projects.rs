use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectFile {
    pub path: String,
    pub content: String,
    pub is_executable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectStage {
    pub name: String,
    pub description: String,
    pub files: Vec<ProjectFile>,
    pub tests: Vec<ProjectFile>,
    pub solution_files: Vec<ProjectFile>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectPack {
    pub id: String,
    pub name: String,
    pub description: String,
    pub difficulty: String,
    pub language: String,
    pub stages: Vec<ProjectStage>,
    pub readme: String,
}

pub struct ProjectEngine {
    builtin: Vec<ProjectPack>,
    installed: Vec<ProjectPack>,
    projects_dir: PathBuf,
}

pub fn builtin_projects() -> Vec<ProjectPack> {
    vec![
        shell_project(),
        git_project(),
        http_server_project(),
        redis_project(),
        compiler_project(),
    ]
}

fn shell_project() -> ProjectPack {
    ProjectPack {
        id: "build-your-own-shell".to_string(),
        name: "Build Your Own Shell".to_string(),
        description: "Implement a Unix-like shell with command execution, piping, and built-in commands.".to_string(),
        difficulty: "medium".to_string(),
        language: "rust".to_string(),
        readme: "# Build Your Own Shell\n\nImplement a minimal Unix shell in Rust.\n\n## Stages\n\n1. **Basic REPL** — Read input, parse commands, execute external programs\n2. **Built-ins** — Implement `cd`, `exit`, `echo`, `type`\n3. **Piping** — Support `|` to chain commands\n4. **Redirection** — Support `>` and `<` for file I/O\n5. **Job Control** — Background processes with `&`\n\n## Getting Started\n\nEach stage builds on the previous one. Start with stage 1 and work your way up.\n\nRun tests with `cargo test` after implementing each stage.".to_string(),
        stages: vec![
            ProjectStage {
                name: "Stage 1: Basic REPL".to_string(),
                description: "Parse user input and execute external commands using std::process::Command.".to_string(),
                files: vec![
                    ProjectFile {
                        path: "Cargo.toml".to_string(),
                        content: r#"[package]
name = "my-shell"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "shell"
path = "src/main.rs"
"#.to_string(),
                        is_executable: false,
                    },
                    ProjectFile {
                        path: "src/main.rs".to_string(),
                        content: r#"use std::io::{self, Write};

fn main() {
    let stdin = io::stdin();
    loop {
        io::stdout().write_all(b"$ ").unwrap();
        io::stdout().flush().unwrap();
        let mut input = String::new();
        match stdin.read_line(&mut input) {
            Ok(0) => break,
            Ok(_) => {
                let input = input.trim();
                if input.is_empty() {
                    continue;
                }
                execute_command(input);
            }
            Err(e) => {
                eprintln!("Error reading input: {}", e);
                break;
            }
        }
    }
}

fn execute_command(input: &str) {
    let parts: Vec<&str> = input.split_whitespace().collect();
    if parts.is_empty() {
        return;
    }
    let program = parts[0];
    let args = &parts[1..];
    let result = std::process::Command::new(program)
        .args(args)
        .output();
    match result {
        Ok(output) => {
            io::stdout().write_all(&output.stdout).unwrap();
            io::stderr().write_all(&output.stderr).unwrap();
        }
        Err(e) => {
            eprintln!("{}: command not found", program);
        }
    }
}
"#.to_string(),
                        is_executable: false,
                    },
                ],
                tests: vec![
                    ProjectFile {
                        path: "tests/shell_test.rs".to_string(),
                        content: r#"#[test]
fn test_shell_executes_ls() {
    let output = std::process::Command::new("cargo")
        .args(["run", "--", "ls"])
        .output()
        .expect("Failed to execute shell");
    assert!(output.status.success() || !output.stderr.is_empty());
}
"#.to_string(),
                        is_executable: false,
                    },
                ],
                solution_files: vec![
                    ProjectFile {
                        path: "src/main.rs".to_string(),
                        content: r#"use std::io::{self, Write};

fn main() {
    let stdin = io::stdin();
    loop {
        io::stdout().write_all(b"$ ").unwrap();
        io::stdout().flush().unwrap();
        let mut input = String::new();
        match stdin.read_line(&mut input) {
            Ok(0) => break,
            Ok(_) => {
                let input = input.trim();
                if input.is_empty() {
                    continue;
                }
                execute_command(input);
            }
            Err(_) => break,
        }
    }
}

fn execute_command(input: &str) {
    let parts: Vec<&str> = input.split_whitespace().collect();
    if parts.is_empty() {
        return;
    }
    let program = parts[0];
    let args = &parts[1..];
    match std::process::Command::new(program).args(args).output() {
        Ok(output) => {
            io::stdout().write_all(&output.stdout).unwrap();
            io::stderr().write_all(&output.stderr).unwrap();
        }
        Err(_) => {
            eprintln!("{}: command not found", program);
        }
    }
}
"#.to_string(),
                        is_executable: false,
                    },
                ],
            },
            ProjectStage {
                name: "Stage 2: Built-in Commands".to_string(),
                description: "Implement cd, exit, echo, and type built-in commands without spawning a process.".to_string(),
                files: vec![
                    ProjectFile {
                        path: "src/main.rs".to_string(),
                        content: r#"use std::env;
use std::io::{self, Write};

fn main() {
    let stdin = io::stdin();
    loop {
        io::stdout().write_all(b"$ ").unwrap();
        io::stdout().flush().unwrap();
        let mut input = String::new();
        match stdin.read_line(&mut input) {
            Ok(0) => break,
            Ok(_) => {
                let input = input.trim();
                if input.is_empty() {
                    continue;
                }
                let parts: Vec<&str> = input.split_whitespace().collect();
                let cmd = parts[0];
                let args = &parts[1..];
                match cmd {
                    "exit" => break,
                    "echo" => {
                        println!("{}", args.join(" "));
                    }
                    "type" => {
                        if !args.is_empty() {
                            match args[0] {
                                "echo" | "exit" | "type" | "cd" => {
                                    println!("{} is a shell builtin", args[0]);
                                }
                                other => {
                                    eprintln!("{}: not found", other);
                                }
                            }
                        }
                    }
                    "cd" => {
                        let path = if args.is_empty() {
                            env::var("HOME").unwrap_or_default()
                        } else {
                            args[0].to_string()
                        };
                        if let Err(e) = env::set_current_dir(&path) {
                            eprintln!("cd: {}: No such file or directory", path);
                        }
                    }
                    _ => {
                        execute_command(cmd, args);
                    }
                }
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                break;
            }
        }
    }
}

fn execute_command(program: &str, args: &[&str]) {
    match std::process::Command::new(program).args(args).output() {
        Ok(output) => {
            io::stdout().write_all(&output.stdout).unwrap();
            io::stderr().write_all(&output.stderr).unwrap();
        }
        Err(_) => {
            eprintln!("{}: command not found", program);
        }
    }
}
"#.to_string(),
                        is_executable: false,
                    },
                ],
                tests: vec![
                    ProjectFile {
                        path: "tests/shell_test.rs".to_string(),
                        content: r#"#[test]
fn test_echo_command() {
    let output = std::process::Command::new("cargo")
        .args(["run", "--", "echo", "hello", "world"])
        .output()
        .expect("Failed to execute");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("hello world"));
}
"#.to_string(),
                        is_executable: false,
                    },
                ],
                solution_files: vec![],
            },
            ProjectStage {
                name: "Stage 3: Piping".to_string(),
                description: "Support piping output from one command into another using |.".to_string(),
                files: vec![
                    ProjectFile {
                        path: "src/main.rs".to_string(),
                        content: r#"use std::env;
use std::io::{self, Read, Write};

fn main() {
    let stdin = io::stdin();
    loop {
        io::stdout().write_all(b"$ ").unwrap();
        io::stdout().flush().unwrap();
        let mut input = String::new();
        match stdin.read_line(&mut input) {
            Ok(0) => break,
            Ok(_) => {
                let input = input.trim();
                if input.is_empty() {
                    continue;
                }
                if input.contains('|') {
                    execute_pipe(input);
                } else {
                    execute_single(input);
                }
            }
            Err(_) => break,
        }
    }
}

fn execute_single(input: &str) {
    let parts: Vec<&str> = input.split_whitespace().collect();
    if parts.is_empty() {
        return;
    }
    match parts[0] {
        "exit" => std::process::exit(0),
        "echo" => println!("{}", parts[1..].join(" ")),
        "type" => {
            if !parts.is_empty() && parts.len() > 1 {
                match parts[1] {
                    "echo" | "exit" | "type" | "cd" => println!("{} is a shell builtin", parts[1]),
                    other => eprintln!("{}: not found", other),
                }
            }
        }
        "cd" => {
            let path = if parts.len() < 2 {
                env::var("HOME").unwrap_or_default()
            } else {
                parts[1].to_string()
            };
            let _ = env::set_current_dir(&path);
        }
        _ => {
            let _ = std::process::Command::new(parts[0])
                .args(&parts[1..])
                .status();
        }
    }
}

fn execute_pipe(input: &str) {
    let commands: Vec<&str> = input.split('|').map(|s| s.trim()).collect();
    let mut prev_output: Vec<u8> = Vec::new();
    for cmd_str in &commands {
        let parts: Vec<&str> = cmd_str.split_whitespace().collect();
        if parts.is_empty() {
            continue;
        }
        let result = std::process::Command::new(parts[0])
            .args(&parts[1..])
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::inherit())
            .output();
        match result {
            Ok(output) => {
                prev_output = output.stdout;
            }
            Err(_) => {
                eprintln!("{}: command not found", parts[0]);
                return;
            }
        }
    }
    io::stdout().write_all(&prev_output).unwrap();
}
"#.to_string(),
                        is_executable: false,
                    },
                ],
                tests: vec![
                    ProjectFile {
                        path: "tests/shell_test.rs".to_string(),
                        content: r#"#[test]
fn test_pipe_commands() {
    let output = std::process::Command::new("cargo")
        .args(["run", "--", "echo", "hello world | cat"])
        .output()
        .expect("Failed to execute");
    assert!(output.status.success() || !output.stderr.is_empty());
}
"#.to_string(),
                        is_executable: false,
                    },
                ],
                solution_files: vec![],
            },
            ProjectStage {
                name: "Stage 4: Redirection".to_string(),
                description: "Support > for output redirection and < for input redirection.".to_string(),
                files: vec![
                    ProjectFile {
                        path: "src/main.rs".to_string(),
                        content: r#"use std::env;
use std::fs::File;
use std::io::{self, Read, Write};

fn main() {
    let stdin = io::stdin();
    loop {
        io::stdout().write_all(b"$ ").unwrap();
        io::stdout().flush().unwrap();
        let mut input = String::new();
        match stdin.read_line(&mut input) {
            Ok(0) => break,
            Ok(_) => {
                let input = input.trim();
                if input.is_empty() {
                    continue;
                }
                let (cmd, redirect_out, redirect_in) = parse_redirection(input);
                execute_with_redirect(&cmd, redirect_out.as_deref(), redirect_in.as_deref());
            }
            Err(_) => break,
        }
    }
}

fn parse_redirection(input: &str) -> (String, Option<String>, Option<String>) {
    let mut cmd = input.to_string();
    let mut redirect_out = None;
    let mut redirect_in = None;
    if let Some(pos) = cmd.find('>') {
        let after = cmd[pos + 1..].trim().to_string();
        cmd = cmd[..pos].trim().to_string();
        redirect_out = Some(after);
    }
    if let Some(pos) = cmd.find('<') {
        let after = cmd[pos + 1..].trim().to_string();
        cmd = cmd[..pos].trim().to_string();
        redirect_in = Some(after);
    }
    (cmd, redirect_out, redirect_in)
}

fn execute_with_redirect(cmd: &str, redirect_out: Option<&str>, redirect_in: Option<&str>) {
    let parts: Vec<&str> = cmd.split_whitespace().collect();
    if parts.is_empty() {
        return;
    }
    match parts[0] {
        "exit" => std::process::exit(0),
        "echo" => {
            let text = parts[1..].join(" ");
            if let Some(file) = redirect_out {
                let _ = std::fs::write(file, format!("{}\n", text));
            } else {
                println!("{}", text);
            }
        }
        "cd" => {
            let path = if parts.len() < 2 {
                env::var("HOME").unwrap_or_default()
            } else {
                parts[1].to_string()
            };
            let _ = env::set_current_dir(&path);
        }
        _ => {
            let mut child = std::process::Command::new(parts[0])
                .args(&parts[1..]);
            if let Some(file) = redirect_in {
                if let Ok(f) = File::open(file) {
                    child = child.stdin(f);
                }
            }
            if let Some(file) = redirect_out {
                if let Ok(f) = File::create(file) {
                    child = child.stdout(f);
                }
            }
            let _ = child.status();
        }
    }
}
"#.to_string(),
                        is_executable: false,
                    },
                ],
                tests: vec![
                    ProjectFile {
                        path: "tests/shell_test.rs".to_string(),
                        content: r#"#[test]
fn test_output_redirection() {
    let output = std::process::Command::new("cargo")
        .args(["run", "--", "echo", "hello > /tmp/test_output.txt"])
        .output()
        .expect("Failed to execute");
    let content = std::fs::read_to_string("/tmp/test_output.txt").unwrap_or_default();
    assert!(content.contains("hello"));
}
"#.to_string(),
                        is_executable: false,
                    },
                ],
                solution_files: vec![],
            },
            ProjectStage {
                name: "Stage 5: Job Control".to_string(),
                description: "Support background processes with & and process management.".to_string(),
                files: vec![
                    ProjectFile {
                        path: "src/main.rs".to_string(),
                        content: r#"use std::env;
use std::fs::File;
use std::io::{self, Write};

fn main() {
    let stdin = io::stdin();
    loop {
        io::stdout().write_all(b"$ ").unwrap();
        io::stdout().flush().unwrap();
        let mut input = String::new();
        match stdin.read_line(&mut input) {
            Ok(0) => break,
            Ok(_) => {
                let input = input.trim();
                if input.is_empty() {
                    continue;
                }
                let background = input.ends_with('&');
                let input = if background {
                    input[..input.len() - 1].trim()
                } else {
                    input
                };
                let (cmd, redirect_out, redirect_in) = parse_redirection(input);
                execute_with_options(&cmd, redirect_out.as_deref(), redirect_in.as_deref(), background);
            }
            Err(_) => break,
        }
    }
}

fn parse_redirection(input: &str) -> (String, Option<String>, Option<String>) {
    let mut cmd = input.to_string();
    let mut redirect_out = None;
    let mut redirect_in = None;
    if let Some(pos) = cmd.find('>') {
        let after = cmd[pos + 1..].trim().to_string();
        cmd = cmd[..pos].trim().to_string();
        redirect_out = Some(after);
    }
    if let Some(pos) = cmd.find('<') {
        let after = cmd[pos + 1..].trim().to_string();
        cmd = cmd[..pos].trim().to_string();
        redirect_in = Some(after);
    }
    (cmd, redirect_out, redirect_in)
}

fn execute_with_options(cmd: &str, redirect_out: Option<&str>, redirect_in: Option<&str>, background: bool) {
    let parts: Vec<&str> = cmd.split_whitespace().collect();
    if parts.is_empty() {
        return;
    }
    match parts[0] {
        "exit" => std::process::exit(0),
        "echo" => {
            let text = parts[1..].join(" ");
            if let Some(file) = redirect_out {
                let _ = std::fs::write(file, format!("{}\n", text));
            } else {
                println!("{}", text);
            }
        }
        "cd" => {
            let path = if parts.len() < 2 {
                env::var("HOME").unwrap_or_default()
            } else {
                parts[1].to_string()
            };
            let _ = env::set_current_dir(&path);
        }
        _ => {
            let mut child = std::process::Command::new(parts[0])
                .args(&parts[1..]);
            if let Some(file) = redirect_in {
                if let Ok(f) = File::open(file) {
                    child = child.stdin(f);
                }
            }
            if let Some(file) = redirect_out {
                if let Ok(f) = File::create(file) {
                    child = child.stdout(f);
                }
            }
            if background {
                child = child.stdin(std::process::Stdio::null());
                match child.spawn() {
                    Ok(child) => println!("[{}]", child.id()),
                    Err(e) => eprintln!("Failed to start background process: {}", e),
                }
            } else {
                let _ = child.status();
            }
        }
    }
}
"#.to_string(),
                        is_executable: false,
                    },
                ],
                tests: vec![
                    ProjectFile {
                        path: "tests/shell_test.rs".to_string(),
                        content: r#"#[test]
fn test_background_process() {
    let output = std::process::Command::new("cargo")
        .args(["run", "--", "sleep", "0.1 &"])
        .output()
        .expect("Failed to execute");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains('['));
}
"#.to_string(),
                        is_executable: false,
                    },
                ],
                solution_files: vec![],
            },
        ],
    }
}

fn git_project() -> ProjectPack {
    ProjectPack {
        id: "build-your-own-git".to_string(),
        name: "Build Your Own Git".to_string(),
        description: "Implement basic git operations: init, add, commit, log, and diff.".to_string(),
        difficulty: "hard".to_string(),
        language: "rust".to_string(),
        readme: "# Build Your Own Git\n\nImplement a simplified version of git in Rust.\n\n## Stages\n\n1. **Init** — Create .devgit directory structure\n2. **Add** — Stage files by computing blob hashes\n3. **Commit** — Create commit objects with tree structure\n4. **Log** — View commit history\n5. **Diff** — Compare staged vs committed files\n\n## Data Model\n\nGit stores objects as:\n- **blob**: file content, hashed with SHA-1\n- **tree**: directory listing (name → blob/tree hash)\n- **commit**: snapshot + metadata + parent\n\n## Getting Started\n\nEach stage builds on the previous one.".to_string(),
        stages: vec![
            ProjectStage {
                name: "Stage 1: Init".to_string(),
                description: "Create the .devgit directory structure with objects and refs directories.".to_string(),
                files: vec![
                    ProjectFile {
                        path: "Cargo.toml".to_string(),
                        content: r#"[package]
name = "my-git"
version = "0.1.0"
edition = "2021"

[dependencies]
sha1 = "0.10"
hex = "0.4"
"#.to_string(),
                        is_executable: false,
                    },
                    ProjectFile {
                        path: "src/main.rs".to_string(),
                        content: r#"use std::env;
use std::fs;
use std::path::Path;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: my-git <command>");
        std::process::exit(1);
    }
    match args[1].as_str() {
        "init" => init_repo(),
        _ => {
            eprintln!("Unknown command: {}", args[1]);
            std::process::exit(1);
        }
    }
}

fn init_repo() {
    let git_dir = Path::new(".devgit");
    if git_dir.exists() {
        println!("Repository already initialized");
        return;
    }
    fs::create_dir_all(git_dir.join("objects")).expect("Failed to create objects directory");
    fs::create_dir_all(git_dir.join("refs/heads")).expect("Failed to create refs directory");
    fs::write(git_dir.join("HEAD"), "ref: refs/heads/main\n").expect("Failed to write HEAD");
    println!("Initialized empty repository in .devgit/");
}
"#.to_string(),
                        is_executable: false,
                    },
                ],
                tests: vec![
                    ProjectFile {
                        path: "tests/git_test.rs".to_string(),
                        content: r#"use std::fs;
use std::process::Command;

#[test]
fn test_init_creates_directories() {
    let _ = fs::remove_dir_all(".devgit");
    let output = Command::new("cargo")
        .args(["run", "--", "init"])
        .output()
        .expect("Failed to run init");
    assert!(output.status.success());
    assert!(Path::new(".devgit/objects").exists());
    assert!(Path::new(".devgit/refs/heads").exists());
    assert!(Path::new(".devgit/HEAD").exists());
    let _ = fs::remove_dir_all(".devgit");
}
"#.to_string(),
                        is_executable: false,
                    },
                ],
                solution_files: vec![],
            },
            ProjectStage {
                name: "Stage 2: Add".to_string(),
                description: "Stage files by computing SHA-1 blob hashes and storing content.".to_string(),
                files: vec![
                    ProjectFile {
                        path: "src/main.rs".to_string(),
                        content: r#"use sha1::{Digest, Sha1};
use std::env;
use std::fs;
use std::path::Path;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: my-git <command>");
        std::process::exit(1);
    }
    match args[1].as_str() {
        "init" => init_repo(),
        "add" => {
            if args.len() < 3 {
                eprintln!("Usage: my-git add <file>");
                std::process::exit(1);
            }
            add_file(&args[2]);
        }
        _ => {
            eprintln!("Unknown command: {}", args[1]);
            std::process::exit(1);
        }
    }
}

fn init_repo() {
    let git_dir = Path::new(".devgit");
    if git_dir.exists() {
        println!("Repository already initialized");
        return;
    }
    fs::create_dir_all(git_dir.join("objects")).expect("Failed to create objects directory");
    fs::create_dir_all(git_dir.join("refs/heads")).expect("Failed to create refs directory");
    fs::write(git_dir.join("HEAD"), "ref: refs/heads/main\n").expect("Failed to write HEAD");
    println!("Initialized empty repository in .devgit/");
}

fn hash_content(content: &[u8]) -> String {
    let mut hasher = Sha1::new();
    hasher.update(content);
    hex::encode(hasher.finalize())
}

fn add_file(filename: &str) {
    let content = fs::read(filename).unwrap_or_else(|e| {
        eprintln!("fatal: path '{}' does not exist", filename);
        std::process::exit(1);
    });
    let hash = hash_content(&content);
    let git_dir = Path::new(".devgit");
    let dir = &hash[..2];
    let file = &hash[2..];
    let obj_dir = git_dir.join("objects").join(dir);
    fs::create_dir_all(&obj_dir).expect("Failed to create object directory");
    fs::write(obj_dir.join(file), &content).expect("Failed to write object");
    let index_path = git_dir.join("index");
    let mut entries = if index_path.exists() {
        fs::read_to_string(&index_path).unwrap_or_default()
    } else {
        String::new()
    };
    entries.push_str(&format!("{} {}\n", hash, filename));
    fs::write(&index_path, entries).expect("Failed to update index");
    println!("Added '{}'", filename);
}
"#.to_string(),
                        is_executable: false,
                    },
                ],
                tests: vec![
                    ProjectFile {
                        path: "tests/git_test.rs".to_string(),
                        content: r#"use std::fs;

#[test]
fn test_add_creates_blob() {
    let _ = fs::remove_dir_all(".devgit");
    std::process::Command::new("cargo")
        .args(["run", "--", "init"])
        .output()
        .unwrap();
    fs::write("test_file.txt", "hello world").unwrap();
    let output = std::process::Command::new("cargo")
        .args(["run", "--", "add", "test_file.txt"])
        .output()
        .unwrap();
    assert!(output.status.success());
    assert!(Path::new(".devgit/index").exists());
    let _ = fs::remove_dir_all(".devgit");
    let _ = fs::remove_file("test_file.txt");
}
"#.to_string(),
                        is_executable: false,
                    },
                ],
                solution_files: vec![],
            },
            ProjectStage {
                name: "Stage 3: Commit".to_string(),
                description: "Create commit objects that snapshot staged files with metadata.".to_string(),
                files: vec![
                    ProjectFile {
                        path: "src/main.rs".to_string(),
                        content: r#"use sha1::{Digest, Sha1};
use std::env;
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: my-git <command>");
        std::process::exit(1);
    }
    match args[1].as_str() {
        "init" => init_repo(),
        "add" => {
            if args.len() < 3 {
                eprintln!("Usage: my-git add <file>");
                std::process::exit(1);
            }
            add_file(&args[2]);
        }
        "commit" => {
            let msg = if args.len() >= 4 && args[2] == "-m" {
                args[3..].join(" ")
            } else {
                eprintln!("Usage: my-git commit -m <message>");
                std::process::exit(1);
            };
            commit(&msg);
        }
        _ => {
            eprintln!("Unknown command: {}", args[1]);
            std::process::exit(1);
        }
    }
}

fn init_repo() {
    let git_dir = Path::new(".devgit");
    if git_dir.exists() {
        println!("Repository already initialized");
        return;
    }
    fs::create_dir_all(git_dir.join("objects")).expect("Failed to create objects directory");
    fs::create_dir_all(git_dir.join("refs/heads")).expect("Failed to create refs directory");
    fs::write(git_dir.join("HEAD"), "ref: refs/heads/main\n").expect("Failed to write HEAD");
    println!("Initialized empty repository in .devgit/");
}

fn hash_content(content: &[u8]) -> String {
    let mut hasher = Sha1::new();
    hasher.update(content);
    hex::encode(hasher.finalize())
}

fn add_file(filename: &str) {
    let content = fs::read(filename).unwrap_or_else(|e| {
        eprintln!("fatal: path '{}' does not exist", filename);
        std::process::exit(1);
    });
    let hash = hash_content(&content);
    let git_dir = Path::new(".devgit");
    let dir = &hash[..2];
    let file = &hash[2..];
    let obj_dir = git_dir.join("objects").join(dir);
    fs::create_dir_all(&obj_dir).expect("Failed to create object directory");
    fs::write(obj_dir.join(file), &content).expect("Failed to write object");
    let index_path = git_dir.join("index");
    let mut entries = if index_path.exists() {
        fs::read_to_string(&index_path).unwrap_or_default()
    } else {
        String::new()
    };
    entries.push_str(&format!("{} {}\n", hash, filename));
    fs::write(&index_path, entries).expect("Failed to update index");
}

fn get_head_commit() -> Option<String> {
    let git_dir = Path::new(".devgit");
    let refs_dir = git_dir.join("refs/heads");
    let head = fs::read_to_string(git_dir.join("HEAD")).unwrap_or_default();
    let head = head.trim();
    if let Some(branch) = head.strip_prefix("ref: refs/heads/") {
        let branch_file = refs_dir.join(branch);
        if branch_file.exists() {
            return fs::read_to_string(branch_file).ok().map(|s| s.trim().to_string());
        }
    }
    None
}

fn commit(message: &str) {
    let git_dir = Path::new(".devgit");
    let index_path = git_dir.join("index");
    if !index_path.exists() {
        eprintln!("nothing to commit");
        return;
    }
    let index = fs::read_to_string(&index_path).unwrap_or_default();
    let parent = get_head_commit();
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let mut commit_content = format!("tree {}\n", index.trim());
    if let Some(ref parent_hash) = parent {
        commit_content.push_str(&format!("parent {}\n", parent_hash));
    }
    commit_content.push_str(&format!("author DevCore <dev@core> {}\n", timestamp));
    commit_content.push_str(&format!("committer DevCore <dev@core> {}\n\n", timestamp));
    commit_content.push_str(message);
    commit_content.push('\n');
    let hash = hash_content(commit_content.as_bytes());
    let dir = &hash[..2];
    let file = &hash[2..];
    let obj_dir = git_dir.join("objects").join(dir);
    fs::create_dir_all(&obj_dir).expect("Failed to create object directory");
    fs::write(obj_dir.join(file), commit_content).expect("Failed to write commit object");
    let refs_dir = git_dir.join("refs/heads");
    fs::create_dir_all(&refs_dir).expect("Failed to create refs directory");
    fs::write(refs_dir.join("main"), &hash).expect("Failed to update ref");
    println!("[main {}] {}", &hash[..7], message);
}
"#.to_string(),
                        is_executable: false,
                    },
                ],
                tests: vec![],
                solution_files: vec![],
            },
            ProjectStage {
                name: "Stage 4: Log".to_string(),
                description: "View the commit history starting from HEAD.".to_string(),
                files: vec![
                    ProjectFile {
                        path: "src/main.rs".to_string(),
                        content: r#"use sha1::{Digest, Sha1};
use std::env;
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: my-git <command>");
        std::process::exit(1);
    }
    match args[1].as_str() {
        "init" => init_repo(),
        "add" => {
            if args.len() < 3 {
                eprintln!("Usage: my-git add <file>");
                std::process::exit(1);
            }
            add_file(&args[2]);
        }
        "commit" => {
            let msg = if args.len() >= 4 && args[2] == "-m" {
                args[3..].join(" ")
            } else {
                eprintln!("Usage: my-git commit -m <message>");
                std::process::exit(1);
            };
            commit(&msg);
        }
        "log" => show_log(),
        _ => {
            eprintln!("Unknown command: {}", args[1]);
            std::process::exit(1);
        }
    }
}

fn init_repo() {
    let git_dir = Path::new(".devgit");
    if git_dir.exists() {
        println!("Repository already initialized");
        return;
    }
    fs::create_dir_all(git_dir.join("objects")).expect("Failed to create objects directory");
    fs::create_dir_all(git_dir.join("refs/heads")).expect("Failed to create refs directory");
    fs::write(git_dir.join("HEAD"), "ref: refs/heads/main\n").expect("Failed to write HEAD");
    println!("Initialized empty repository in .devgit/");
}

fn hash_content(content: &[u8]) -> String {
    let mut hasher = Sha1::new();
    hasher.update(content);
    hex::encode(hasher.finalize())
}

fn add_file(filename: &str) {
    let content = fs::read(filename).unwrap_or_else(|e| {
        eprintln!("fatal: path '{}' does not exist", filename);
        std::process::exit(1);
    });
    let hash = hash_content(&content);
    let git_dir = Path::new(".devgit");
    let dir = &hash[..2];
    let file = &hash[2..];
    let obj_dir = git_dir.join("objects").join(dir);
    fs::create_dir_all(&obj_dir).expect("Failed to create object directory");
    fs::write(obj_dir.join(file), &content).expect("Failed to write object");
    let index_path = git_dir.join("index");
    let mut entries = if index_path.exists() {
        fs::read_to_string(&index_path).unwrap_or_default()
    } else {
        String::new()
    };
    entries.push_str(&format!("{} {}\n", hash, filename));
    fs::write(&index_path, entries).expect("Failed to update index");
}

fn get_head_commit() -> Option<String> {
    let git_dir = Path::new(".devgit");
    let refs_dir = git_dir.join("refs/heads");
    let head = fs::read_to_string(git_dir.join("HEAD")).unwrap_or_default();
    let head = head.trim();
    if let Some(branch) = head.strip_prefix("ref: refs/heads/") {
        let branch_file = refs_dir.join(branch);
        if branch_file.exists() {
            return fs::read_to_string(branch_file).ok().map(|s| s.trim().to_string());
        }
    }
    None
}

fn read_object(hash: &str) -> Option<String> {
    let git_dir = Path::new(".devgit");
    let dir = &hash[..2];
    let file = &hash[2..];
    fs::read_to_string(git_dir.join("objects").join(dir).join(file)).ok()
}

fn commit(message: &str) {
    let git_dir = Path::new(".devgit");
    let index_path = git_dir.join("index");
    if !index_path.exists() {
        eprintln!("nothing to commit");
        return;
    }
    let index = fs::read_to_string(&index_path).unwrap_or_default();
    let parent = get_head_commit();
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let mut commit_content = format!("tree {}\n", index.trim());
    if let Some(ref parent_hash) = parent {
        commit_content.push_str(&format!("parent {}\n", parent_hash));
    }
    commit_content.push_str(&format!("author DevCore <dev@core> {}\n", timestamp));
    commit_content.push_str(&format!("committer DevCore <dev@core> {}\n\n", timestamp));
    commit_content.push_str(message);
    commit_content.push('\n');
    let hash = hash_content(commit_content.as_bytes());
    let dir = &hash[..2];
    let file = &hash[2..];
    let obj_dir = git_dir.join("objects").join(dir);
    fs::create_dir_all(&obj_dir).expect("Failed to create object directory");
    fs::write(obj_dir.join(file), commit_content).expect("Failed to write commit object");
    let refs_dir = git_dir.join("refs/heads");
    fs::create_dir_all(&refs_dir).expect("Failed to create refs directory");
    fs::write(refs_dir.join("main"), &hash).expect("Failed to update ref");
    println!("[main {}] {}", &hash[..7], message);
}

fn show_log() {
    let mut current = get_head_commit();
    while let Some(hash) = current {
        if let Some(content) = read_object(&hash) {
            let mut message = String::new();
            let mut parent = None;
            let mut in_body = false;
            for line in content.lines() {
                if in_body {
                    message.push_str(line);
                    message.push('\n');
                } else if line.starts_with("parent ") {
                    parent = Some(line[7..].to_string());
                } else if line.is_empty() {
                    in_body = true;
                }
            }
            println!("commit {}", hash);
            println!("Author: DevCore <dev@core>");
            println!();
            println!("{}", message.trim());
            println!();
            current = parent;
        } else {
            break;
        }
    }
}
"#.to_string(),
                        is_executable: false,
                    },
                ],
                tests: vec![],
                solution_files: vec![],
            },
            ProjectStage {
                name: "Stage 5: Diff".to_string(),
                description: "Compare staged files against the last committed version.".to_string(),
                files: vec![
                    ProjectFile {
                        path: "src/main.rs".to_string(),
                        content: r#"use sha1::{Digest, Sha1};
use std::env;
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: my-git <command>");
        std::process::exit(1);
    }
    match args[1].as_str() {
        "init" => init_repo(),
        "add" => {
            if args.len() < 3 {
                eprintln!("Usage: my-git add <file>");
                std::process::exit(1);
            }
            add_file(&args[2]);
        }
        "commit" => {
            let msg = if args.len() >= 4 && args[2] == "-m" {
                args[3..].join(" ")
            } else {
                eprintln!("Usage: my-git commit -m <message>");
                std::process::exit(1);
            };
            commit(&msg);
        }
        "log" => show_log(),
        "diff" => show_diff(),
        _ => {
            eprintln!("Unknown command: {}", args[1]);
            std::process::exit(1);
        }
    }
}

fn init_repo() {
    let git_dir = Path::new(".devgit");
    if git_dir.exists() {
        println!("Repository already initialized");
        return;
    }
    fs::create_dir_all(git_dir.join("objects")).expect("Failed to create objects directory");
    fs::create_dir_all(git_dir.join("refs/heads")).expect("Failed to create refs directory");
    fs::write(git_dir.join("HEAD"), "ref: refs/heads/main\n").expect("Failed to write HEAD");
    println!("Initialized empty repository in .devgit/");
}

fn hash_content(content: &[u8]) -> String {
    let mut hasher = Sha1::new();
    hasher.update(content);
    hex::encode(hasher.finalize())
}

fn add_file(filename: &str) {
    let content = fs::read(filename).unwrap_or_else(|e| {
        eprintln!("fatal: path '{}' does not exist", filename);
        std::process::exit(1);
    });
    let hash = hash_content(&content);
    let git_dir = Path::new(".devgit");
    let dir = &hash[..2];
    let file = &hash[2..];
    let obj_dir = git_dir.join("objects").join(dir);
    fs::create_dir_all(&obj_dir).expect("Failed to create object directory");
    fs::write(obj_dir.join(file), &content).expect("Failed to write object");
    let index_path = git_dir.join("index");
    let mut entries = if index_path.exists() {
        fs::read_to_string(&index_path).unwrap_or_default()
    } else {
        String::new()
    };
    entries.push_str(&format!("{} {}\n", hash, filename));
    fs::write(&index_path, entries).expect("Failed to update index");
}

fn get_head_commit() -> Option<String> {
    let git_dir = Path::new(".devgit");
    let refs_dir = git_dir.join("refs/heads");
    let head = fs::read_to_string(git_dir.join("HEAD")).unwrap_or_default();
    let head = head.trim();
    if let Some(branch) = head.strip_prefix("ref: refs/heads/") {
        let branch_file = refs_dir.join(branch);
        if branch_file.exists() {
            return fs::read_to_string(branch_file).ok().map(|s| s.trim().to_string());
        }
    }
    None
}

fn read_object(hash: &str) -> Option<String> {
    let git_dir = Path::new(".devgit");
    let dir = &hash[..2];
    let file = &hash[2..];
    fs::read_to_string(git_dir.join("objects").join(dir).join(file)).ok()
}

fn commit(message: &str) {
    let git_dir = Path::new(".devgit");
    let index_path = git_dir.join("index");
    if !index_path.exists() {
        eprintln!("nothing to commit");
        return;
    }
    let index = fs::read_to_string(&index_path).unwrap_or_default();
    let parent = get_head_commit();
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let mut commit_content = format!("tree {}\n", index.trim());
    if let Some(ref parent_hash) = parent {
        commit_content.push_str(&format!("parent {}\n", parent_hash));
    }
    commit_content.push_str(&format!("author DevCore <dev@core> {}\n", timestamp));
    commit_content.push_str(&format!("committer DevCore <dev@core> {}\n\n", timestamp));
    commit_content.push_str(message);
    commit_content.push('\n');
    let hash = hash_content(commit_content.as_bytes());
    let dir = &hash[..2];
    let file = &hash[2..];
    let obj_dir = git_dir.join("objects").join(dir);
    fs::create_dir_all(&obj_dir).expect("Failed to create object directory");
    fs::write(obj_dir.join(file), commit_content).expect("Failed to write commit object");
    let refs_dir = git_dir.join("refs/heads");
    fs::create_dir_all(&refs_dir).expect("Failed to create refs directory");
    fs::write(refs_dir.join("main"), &hash).expect("Failed to update ref");
    println!("[main {}] {}", &hash[..7], message);
}

fn show_log() {
    let mut current = get_head_commit();
    while let Some(hash) = current {
        if let Some(content) = read_object(&hash) {
            let mut message = String::new();
            let mut parent = None;
            let mut in_body = false;
            for line in content.lines() {
                if in_body {
                    message.push_str(line);
                    message.push('\n');
                } else if line.starts_with("parent ") {
                    parent = Some(line[7..].to_string());
                } else if line.is_empty() {
                    in_body = true;
                }
            }
            println!("commit {}", hash);
            println!("Author: DevCore <dev@core>");
            println!();
            println!("{}", message.trim());
            println!();
            current = parent;
        } else {
            break;
        }
    }
}

fn show_diff() {
    let git_dir = Path::new(".devgit");
    let index_path = git_dir.join("index");
    if !index_path.exists() {
        println!("No staged changes");
        return;
    }
    let index = fs::read_to_string(&index_path).unwrap_or_default();
    let mut staged_files: Vec<(String, String)> = Vec::new();
    for line in index.lines() {
        let parts: Vec<&str> = line.splitn(2, ' ').collect();
        if parts.len() == 2 {
            staged_files.push((parts[0].to_string(), parts[1].to_string()));
        }
    }
    let parent = get_head_commit();
    for (hash, filename) in &staged_files {
        let new_content = fs::read_to_string(filename).unwrap_or_default();
        let old_content = if let Some(ref parent_hash) = parent {
            if let Some(tree_line) = read_object(parent_hash) {
                let tree_lines: Vec<&str> = tree_line.lines().collect();
                if !tree_lines.is_empty() {
                    let tree_hash = tree_lines[0].trim_start_matches("tree ");
                    if let Some(old_index) = read_object(tree_hash) {
                        for line in old_index.lines() {
                            let parts: Vec<&str> = line.splitn(2, ' ').collect();
                            if parts.len() == 2 && parts[1] == *filename {
                                if let Some(content) = read_object(parts[0]) {
                                    content
                                } else {
                                    String::new()
                                }
                                break;
                            }
                        }
                        String::new()
                    } else {
                        String::new()
                    }
                } else {
                    String::new()
                }
            } else {
                String::new()
            }
        } else {
            String::new()
        };
        if old_content != new_content {
            println!("diff --git a/{} b/{}", filename, filename);
            println!("--- a/{}", filename);
            println!("+++ b/{}", filename);
            for line in new_content.lines() {
                println!("+{}", line);
            }
        }
    }
}
"#.to_string(),
                        is_executable: false,
                    },
                ],
                tests: vec![],
                solution_files: vec![],
            },
        ],
    }
}

fn http_server_project() -> ProjectPack {
    ProjectPack {
        id: "build-your-own-http-server".to_string(),
        name: "Build Your Own HTTP Server".to_string(),
        description: "Implement a simple HTTP server that handles GET and POST requests with routing.".to_string(),
        difficulty: "medium".to_string(),
        language: "rust".to_string(),
        readme: "# Build Your Own HTTP Server\n\nImplement a basic HTTP server from scratch in Rust.\n\n## Stages\n\n1. **TCP Listener** — Accept connections and read raw HTTP requests\n2. **Request Parser** — Parse method, path, headers, and body\n3. **Router** — Map paths to handler functions\n4. **Response Builder** — Construct proper HTTP responses with status codes\n5. **Static Files** — Serve files from a directory\n\n## Getting Started\n\nWork through each stage sequentially. Each builds on the previous.".to_string(),
        stages: vec![
            ProjectStage {
                name: "Stage 1: TCP Listener".to_string(),
                description: "Accept TCP connections and read raw HTTP request data.".to_string(),
                files: vec![
                    ProjectFile {
                        path: "Cargo.toml".to_string(),
                        content: r#"[package]
name = "my-http-server"
version = "0.1.0"
edition = "2021"
"#.to_string(),
                        is_executable: false,
                    },
                    ProjectFile {
                        path: "src/main.rs".to_string(),
                        content: r#"use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080").expect("Failed to bind");
    println!("Server listening on 127.0.0.1:8080");
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                handle_connection(stream);
            }
            Err(e) => {
                eprintln!("Connection failed: {}", e);
            }
        }
    }
}

fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 512];
    let bytes_read = stream.read(&mut buffer).unwrap_or(0);
    let request = String::from_utf8_lossy(&buffer[..bytes_read]);
    println!("Request:\n{}", request);
    let response = "HTTP/1.1 200 OK\r\n\r\nHello, World!";
    stream.write_all(response.as_bytes()).unwrap_or(());
    stream.flush().unwrap_or(());
}
"#.to_string(),
                        is_executable: false,
                    },
                ],
                tests: vec![],
                solution_files: vec![],
            },
            ProjectStage {
                name: "Stage 2: Request Parser".to_string(),
                description: "Parse HTTP method, path, headers, and body from raw request data.".to_string(),
                files: vec![
                    ProjectFile {
                        path: "src/main.rs".to_string(),
                        content: r#"use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

struct HttpRequest {
    method: String,
    path: String,
    headers: HashMap<String, String>,
    body: String,
}

fn parse_request(raw: &str) -> Option<HttpRequest> {
    let mut lines = raw.lines();
    let request_line = lines.next()?;
    let parts: Vec<&str> = request_line.split_whitespace().collect();
    if parts.len() < 3 {
        return None;
    }
    let mut headers = HashMap::new();
    let mut body = String::new();
    let mut body_started = false;
    for line in lines {
        if body_started {
            body.push_str(line);
            body.push('\n');
        } else if line.is_empty() {
            body_started = true;
        } else if let Some((key, value)) = line.split_once(':') {
            headers.insert(key.trim().to_lowercase(), value.trim().to_string());
        }
    }
    Some(HttpRequest {
        method: parts[0].to_string(),
        path: parts[1].to_string(),
        headers,
        body: body.trim().to_string(),
    })
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080").expect("Failed to bind");
    println!("Server listening on 127.0.0.1:8080");
    for stream in listener.incoming() {
        if let Ok(stream) = stream {
            handle_connection(stream);
        }
    }
}

fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 4096];
    let bytes_read = stream.read(&mut buffer).unwrap_or(0);
    let request = String::from_utf8_lossy(&buffer[..bytes_read]);
    if let Some(req) = parse_request(&request) {
        println!("{} {} HTTP/1.1", req.method, req.path);
        for (k, v) in &req.headers {
            println!("  {}: {}", k, v);
        }
        let response = format!("HTTP/1.1 200 OK\r\nContent-Length: 13\r\n\r\nHello, World!");
        stream.write_all(response.as_bytes()).unwrap_or(());
    }
}
"#.to_string(),
                        is_executable: false,
                    },
                ],
                tests: vec![],
                solution_files: vec![],
            },
            ProjectStage {
                name: "Stage 3: Router".to_string(),
                description: "Map URL paths to handler functions for GET and POST requests.".to_string(),
                files: vec![
                    ProjectFile {
                        path: "src/main.rs".to_string(),
                        content: r#"use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

struct HttpRequest {
    method: String,
    path: String,
    headers: HashMap<String, String>,
    body: String,
}

struct HttpResponse {
    status: u16,
    status_text: String,
    headers: HashMap<String, String>,
    body: String,
}

type Handler = fn(&HttpRequest) -> HttpResponse;

struct Router {
    routes: Vec<(String, String, Handler)>,
}

impl Router {
    fn new() -> Self {
        Self { routes: Vec::new() }
    }

    fn get(&mut self, path: &str, handler: Handler) {
        self.routes.push(("GET".to_string(), path.to_string(), handler));
    }

    fn post(&mut self, path: &str, handler: Handler) {
        self.routes.push(("POST".to_string(), path.to_string(), handler));
    }

    fn handle(&self, req: &HttpRequest) -> HttpResponse {
        for (method, path, handler) in &self.routes {
            if req.method == *method && req.path == *path {
                return handler(req);
            }
        }
        HttpResponse {
            status: 404,
            status_text: "Not Found".to_string(),
            headers: HashMap::new(),
            body: "Not Found".to_string(),
        }
    }
}

fn parse_request(raw: &str) -> Option<HttpRequest> {
    let mut lines = raw.lines();
    let request_line = lines.next()?;
    let parts: Vec<&str> = request_line.split_whitespace().collect();
    if parts.len() < 3 {
        return None;
    }
    let mut headers = HashMap::new();
    let mut body = String::new();
    let mut body_started = false;
    for line in lines {
        if body_started {
            body.push_str(line);
            body.push('\n');
        } else if line.is_empty() {
            body_started = true;
        } else if let Some((key, value)) = line.split_once(':') {
            headers.insert(key.trim().to_lowercase(), value.trim().to_string());
        }
    }
    Some(HttpRequest {
        method: parts[0].to_string(),
        path: parts[1].to_string(),
        headers,
        body: body.trim().to_string(),
    })
}

fn format_response(resp: &HttpResponse) -> String {
    let mut response = format!("HTTP/1.1 {} {}\r\n", resp.status, resp.status_text);
    let body_bytes = resp.body.as_bytes();
    response.push_str(&format!("Content-Length: {}\r\n", body_bytes.len()));
    for (k, v) in &resp.headers {
        response.push_str(&format!("{}: {}\r\n", k, v));
    }
    response.push_str("\r\n");
    response.push_str(&resp.body);
    response
}

fn index_handler(_req: &HttpRequest) -> HttpResponse {
    HttpResponse {
        status: 200,
        status_text: "OK".to_string(),
        headers: HashMap::new(),
        body: "Welcome to the HTTP server!".to_string(),
    }
}

fn echo_handler(req: &HttpRequest) -> HttpResponse {
    HttpResponse {
        status: 200,
        status_text: "OK".to_string(),
        headers: HashMap::new(),
        body: req.body.clone(),
    }
}

fn main() {
    let mut router = Router::new();
    router.get("/", index_handler);
    router.post("/echo", echo_handler);
    let listener = TcpListener::bind("127.0.0.1:8080").expect("Failed to bind");
    println!("Server listening on 127.0.0.1:8080");
    for stream in listener.incoming() {
        if let Ok(stream) = stream {
            handle_connection(stream, &router);
        }
    }
}

fn handle_connection(mut stream: TcpStream, router: &Router) {
    let mut buffer = [0; 4096];
    let bytes_read = stream.read(&mut buffer).unwrap_or(0);
    let request = String::from_utf8_lossy(&buffer[..bytes_read]);
    if let Some(req) = parse_request(&request) {
        let resp = router.handle(&req);
        let response = format_response(&resp);
        stream.write_all(response.as_bytes()).unwrap_or(());
    }
}
"#.to_string(),
                        is_executable: false,
                    },
                ],
                tests: vec![],
                solution_files: vec![],
            },
            ProjectStage {
                name: "Stage 4: Response Builder".to_string(),
                description: "Build proper HTTP responses with headers, status codes, and content types.".to_string(),
                files: vec![
                    ProjectFile {
                        path: "src/main.rs".to_string(),
                        content: r#"use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

struct HttpRequest {
    method: String,
    path: String,
    headers: HashMap<String, String>,
    body: String,
}

struct HttpResponse {
    status: u16,
    status_text: String,
    headers: HashMap<String, String>,
    body: String,
}

impl HttpResponse {
    fn ok(body: &str) -> Self {
        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), "text/plain".to_string());
        Self {
            status: 200,
            status_text: "OK".to_string(),
            headers,
            body: body.to_string(),
        }
    }

    fn json(body: &str) -> Self {
        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), "application/json".to_string());
        Self {
            status: 200,
            status_text: "OK".to_string(),
            headers,
            body: body.to_string(),
        }
    }

    fn not_found() -> Self {
        Self {
            status: 404,
            status_text: "Not Found".to_string(),
            headers: HashMap::new(),
            body: "Not Found".to_string(),
        }
    }

    fn method_not_allowed() -> Self {
        Self {
            status: 405,
            status_text: "Method Not Allowed".to_string(),
            headers: HashMap::new(),
            body: "Method Not Allowed".to_string(),
        }
    }
}

fn parse_request(raw: &str) -> Option<HttpRequest> {
    let mut lines = raw.lines();
    let request_line = lines.next()?;
    let parts: Vec<&str> = request_line.split_whitespace().collect();
    if parts.len() < 3 {
        return None;
    }
    let mut headers = HashMap::new();
    let mut body = String::new();
    let mut body_started = false;
    for line in lines {
        if body_started {
            body.push_str(line);
            body.push('\n');
        } else if line.is_empty() {
            body_started = true;
        } else if let Some((key, value)) = line.split_once(':') {
            headers.insert(key.trim().to_lowercase(), value.trim().to_string());
        }
    }
    Some(HttpRequest {
        method: parts[0].to_string(),
        path: parts[1].to_string(),
        headers,
        body: body.trim().to_string(),
    })
}

fn format_response(resp: &HttpResponse) -> String {
    let mut response = format!("HTTP/1.1 {} {}\r\n", resp.status, resp.status_text);
    let body_bytes = resp.body.as_bytes();
    response.push_str(&format!("Content-Length: {}\r\n", body_bytes.len()));
    for (k, v) in &resp.headers {
        response.push_str(&format!("{}: {}\r\n", k, v));
    }
    response.push_str("\r\n");
    response.push_str(&resp.body);
    response
}

fn handle_request(req: &HttpRequest) -> HttpResponse {
    match req.path.as_str() {
        "/" => HttpResponse::ok("Welcome to the HTTP server!"),
        "/health" => HttpResponse::json(r#"{"status": "ok"}"#),
        "/echo" if req.method == "POST" => HttpResponse::ok(&req.body),
        "/echo" => HttpResponse::method_not_allowed(),
        _ => HttpResponse::not_found(),
    }
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080").expect("Failed to bind");
    println!("Server listening on 127.0.0.1:8080");
    for stream in listener.incoming() {
        if let Ok(stream) = stream {
            handle_connection(stream);
        }
    }
}

fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 4096];
    let bytes_read = stream.read(&mut buffer).unwrap_or(0);
    let request = String::from_utf8_lossy(&buffer[..bytes_read]);
    if let Some(req) = parse_request(&request) {
        let resp = handle_request(&req);
        let response = format_response(&resp);
        stream.write_all(response.as_bytes()).unwrap_or(());
    }
}
"#.to_string(),
                        is_executable: false,
                    },
                ],
                tests: vec![],
                solution_files: vec![],
            },
            ProjectStage {
                name: "Stage 5: Static Files".to_string(),
                description: "Serve static files from a directory with proper content types.".to_string(),
                files: vec![
                    ProjectFile {
                        path: "src/main.rs".to_string(),
                        content: r#"use std::collections::HashMap;
use std::fs;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::Path;

struct HttpRequest {
    method: String,
    path: String,
    headers: HashMap<String, String>,
    body: String,
}

struct HttpResponse {
    status: u16,
    status_text: String,
    headers: HashMap<String, String>,
    body: Vec<u8>,
}

impl HttpResponse {
    fn ok_text(body: &str) -> Self {
        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), "text/plain".to_string());
        Self {
            status: 200,
            status_text: "OK".to_string(),
            headers,
            body: body.as_bytes().to_vec(),
        }
    }

    fn not_found() -> Self {
        Self {
            status: 404,
            status_text: "Not Found".to_string(),
            headers: HashMap::new(),
            body: b"Not Found".to_vec(),
        }
    }

    fn internal_error() -> Self {
        Self {
            status: 500,
            status_text: "Internal Server Error".to_string(),
            headers: HashMap::new(),
            body: b"Internal Server Error".to_vec(),
        }
    }
}

fn content_type_for(path: &str) -> &str {
    if path.ends_with(".html") { "text/html" }
    else if path.ends_with(".css") { "text/css" }
    else if path.ends_with(".js") { "application/javascript" }
    else if path.ends_with(".json") { "application/json" }
    else if path.ends_with(".png") { "image/png" }
    else if path.ends_with(".jpg") || path.ends_with(".jpeg") { "image/jpeg" }
    else if path.ends_with(".txt") { "text/plain" }
    else { "application/octet-stream" }
}

fn parse_request(raw: &str) -> Option<HttpRequest> {
    let mut lines = raw.lines();
    let request_line = lines.next()?;
    let parts: Vec<&str> = request_line.split_whitespace().collect();
    if parts.len() < 3 {
        return None;
    }
    let mut headers = HashMap::new();
    let mut body = String::new();
    let mut body_started = false;
    for line in lines {
        if body_started {
            body.push_str(line);
            body.push('\n');
        } else if line.is_empty() {
            body_started = true;
        } else if let Some((key, value)) = line.split_once(':') {
            headers.insert(key.trim().to_lowercase(), value.trim().to_string());
        }
    }
    Some(HttpRequest {
        method: parts[0].to_string(),
        path: parts[1].to_string(),
        headers,
        body: body.trim().to_string(),
    })
}

fn serve_static(root: &Path, path: &str) -> HttpResponse {
    let file_path = if path == "/" {
        root.join("index.html")
    } else {
        root.join(path.trim_start_matches('/'))
    };
    match fs::read(&file_path) {
        Ok(content) => {
            let ct = content_type_for(file_path.to_str().unwrap_or(""));
            let mut headers = HashMap::new();
            headers.insert("Content-Type".to_string(), ct.to_string());
            HttpResponse {
                status: 200,
                status_text: "OK".to_string(),
                headers,
                body: content,
            }
        }
        Err(_) => HttpResponse::not_found(),
    }
}

fn main() {
    let static_dir = Path::new("public");
    let _ = fs::create_dir_all(static_dir);
    let _ = fs::write(static_dir.join("index.html"), "<h1>Hello, Static World!</h1>");
    let listener = TcpListener::bind("127.0.0.1:8080").expect("Failed to bind");
    println!("Server listening on 127.0.0.1:8080 (serving from ./public)");
    for stream in listener.incoming() {
        if let Ok(stream) = stream {
            handle_connection(stream, static_dir);
        }
    }
}

fn handle_connection(mut stream: TcpStream, root: &Path) {
    let mut buffer = [0; 4096];
    let bytes_read = stream.read(&mut buffer).unwrap_or(0);
    let request = String::from_utf8_lossy(&buffer[..bytes_read]);
    if let Some(req) = parse_request(&request) {
        let resp = serve_static(root, &req.path);
        let mut response = format!("HTTP/1.1 {} {}\r\n", resp.status, resp.status_text);
        for (k, v) in &resp.headers {
            response.push_str(&format!("{}: {}\r\n", k, v));
        }
        response.push_str(&format!("Content-Length: {}\r\n", resp.body.len()));
        response.push_str("\r\n");
        let mut bytes = response.into_bytes();
        bytes.extend_from_slice(&resp.body);
        stream.write_all(&bytes).unwrap_or(());
    }
}
"#.to_string(),
                        is_executable: false,
                    },
                ],
                tests: vec![],
                solution_files: vec![],
            },
        ],
    }
}

fn redis_project() -> ProjectPack {
    ProjectPack {
        id: "build-your-own-redis".to_string(),
        name: "Build Your Own Redis".to_string(),
        description: "Implement a key-value store with SET, GET, DEL, and expiration support.".to_string(),
        difficulty: "medium".to_string(),
        language: "rust".to_string(),
        readme: "# Build Your Own Redis\n\nImplement a simplified Redis-like key-value store in Rust.\n\n## Stages\n\n1. **TCP Server** — Accept connections and parse RESP protocol\n2. **Basic Commands** — Implement SET, GET, DEL, PING\n3. **Expiration** — Add TTL support with SET EX\n4. **Persistence** — Save data to disk with SAVE command\n5. **Lists** — Implement LPUSH, RPUSH, LRANGE for list data structures\n\n## Getting Started\n\nRedis uses the RESP (REdis Serialization Protocol) for communication.".to_string(),
        stages: vec![
            ProjectStage {
                name: "Stage 1: TCP Server".to_string(),
                description: "Accept TCP connections and parse the RESP protocol format.".to_string(),
                files: vec![
                    ProjectFile {
                        path: "Cargo.toml".to_string(),
                        content: r#"[package]
name = "my-redis"
version = "0.1.0"
edition = "2021"
"#.to_string(),
                        is_executable: false,
                    },
                    ProjectFile {
                        path: "src/main.rs".to_string(),
                        content: r#"use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};

struct Store {
    data: HashMap<String, String>,
}

fn parse_resp_line(reader: &mut BufReader<&TcpStream>) -> Option<String> {
    let mut line = String::new();
    reader.read_line(&mut line).ok()?;
    let line = line.trim_end_matches("\r\n").to_string();
    if line.starts_with('*') {
        let count: usize = line[1..].parse().ok()?;
        let mut elements = Vec::new();
        for _ in 0..count {
            let mut len_line = String::new();
            reader.read_line(&mut len_line).ok()?;
            let len: usize = len_line.trim_start_matches('$').trim_end_matches("\r\n").parse().ok()?;
            let mut buf = vec![0u8; len];
            reader.read_exact(&mut buf).ok()?;
            let mut crlf = [0u8; 2];
            reader.read_exact(&mut crlf).ok()?;
            elements.push(String::from_utf8_lossy(&buf).to_string());
        }
        Some(elements.join(" "))
    } else {
        Some(line)
    }
}

fn main() {
    let store = Arc::new(Mutex::new(Store {
        data: HashMap::new(),
    }));
    let listener = TcpListener::bind("127.0.0.1:6379").expect("Failed to bind");
    println!("Redis server listening on 127.0.0.1:6379");
    for stream in listener.incoming() {
        if let Ok(stream) = stream {
            let store = store.clone();
            std::thread::spawn(move || handle_connection(stream, store));
        }
    }
}

fn handle_connection(stream: TcpStream, store: Arc<Mutex<Store>>) {
    let mut writer = stream.try_clone().unwrap();
    let mut reader = BufReader::new(&stream);
    loop {
        match parse_resp_line(&mut reader) {
            Some(line) => {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.is_empty() {
                    continue;
                }
                let response = match parts[0].to_uppercase().as_str() {
                    "PING" => "+PONG\r\n".to_string(),
                    "SET" => {
                        if parts.len() >= 3 {
                            let mut store = store.lock().unwrap();
                            store.data.insert(parts[1].to_string(), parts[2].to_string());
                            "+OK\r\n".to_string()
                        } else {
                            "-ERR wrong number of arguments for 'set' command\r\n".to_string()
                        }
                    }
                    "GET" => {
                        if parts.len() >= 2 {
                            let store = store.lock().unwrap();
                            match store.data.get(parts[1]) {
                                Some(val) => format!("${}\r\n{}\r\n", val.len(), val),
                                None => "$-1\r\n".to_string(),
                            }
                        } else {
                            "-ERR wrong number of arguments for 'get' command\r\n".to_string()
                        }
                    }
                    "DEL" => {
                        if parts.len() >= 2 {
                            let mut store = store.lock().unwrap();
                            let mut deleted = 0;
                            for key in &parts[1..] {
                                if store.data.remove(*key).is_some() {
                                    deleted += 1;
                                }
                            }
                            format!(":{}\r\n", deleted)
                        } else {
                            "-ERR wrong number of arguments for 'del' command\r\n".to_string()
                        }
                    }
                    "QUIT" => {
                        let _ = writer.write_all("+OK\r\n".as_bytes());
                        break;
                    }
                    _ => "-ERR unknown command\r\n".to_string(),
                };
                let _ = writer.write_all(response.as_bytes());
            }
            None => break,
        }
    }
}
"#.to_string(),
                        is_executable: false,
                    },
                ],
                tests: vec![],
                solution_files: vec![],
            },
            ProjectStage {
                name: "Stage 2: Basic Commands".to_string(),
                description: "Implement SET, GET, DEL, and PING commands with proper RESP responses.".to_string(),
                files: vec![
                    ProjectFile {
                        path: "src/main.rs".to_string(),
                        content: r#"use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};

struct Store {
    data: HashMap<String, String>,
}

fn parse_resp(reader: &mut BufReader<&TcpStream>) -> Option<Vec<String>> {
    let mut line = String::new();
    reader.read_line(&mut line).ok()?;
    let line = line.trim();
    if !line.starts_with('*') {
        return Some(vec![line.to_string()]);
    }
    let count: usize = line[1..].parse().ok()?;
    let mut elements = Vec::new();
    for _ in 0..count {
        let mut len_line = String::new();
        reader.read_line(&mut len_line).ok()?;
        let len: usize = len_line.trim().trim_start_matches('$').parse().ok()?;
        let mut buf = vec![0u8; len];
        reader.read_exact(&mut buf).ok()?;
        let mut crlf = [0u8; 2];
        reader.read_exact(&mut crlf).ok()?;
        elements.push(String::from_utf8_lossy(&buf).to_string());
    }
    Some(elements)
}

fn resp_bulk(data: &str) -> String {
    format!("${}\r\n{}\r\n", data.len(), data)
}

fn resp_int(val: i64) -> String {
    format!(":{}\r\n", val)
}

fn resp_ok() -> String {
    "+OK\r\n".to_string()
}

fn resp_err(msg: &str) -> String {
    format!("-ERR {}\r\n", msg)
}

fn resp_null() -> String {
    "$-1\r\n".to_string()
}

fn main() {
    let store = Arc::new(Mutex::new(Store {
        data: HashMap::new(),
    }));
    let listener = TcpListener::bind("127.0.0.1:6379").expect("Failed to bind");
    println!("Listening on 127.0.0.1:6379");
    for stream in listener.incoming().flatten() {
        let store = store.clone();
        std::thread::spawn(move || handle_client(stream, store));
    }
}

fn handle_client(stream: TcpStream, store: Arc<Mutex<Store>>) {
    let mut writer = stream.try_clone().unwrap();
    let mut reader = BufReader::new(&stream);
    loop {
        let Some(args) = parse_resp(&mut reader) else { break };
        if args.is_empty() {
            continue;
        }
        let cmd = args[0].to_uppercase();
        let response = match cmd.as_str() {
            "PING" => resp_ok(),
            "SET" => {
                if args.len() < 3 {
                    resp_err("wrong number of arguments for 'SET'")
                } else {
                    store.lock().unwrap().data.insert(args[1].clone(), args[2].clone());
                    resp_ok()
                }
            }
            "GET" => {
                if args.len() < 2 {
                    resp_err("wrong number of arguments for 'GET'")
                } else {
                    let s = store.lock().unwrap();
                    match s.data.get(&args[1]) {
                        Some(v) => resp_bulk(v),
                        None => resp_null(),
                    }
                }
            }
            "DEL" => {
                if args.len() < 2 {
                    resp_err("wrong number of arguments for 'DEL'")
                } else {
                    let mut s = store.lock().unwrap();
                    let mut count = 0i64;
                    for key in &args[1..] {
                        if s.data.remove(key).is_some() {
                            count += 1;
                        }
                    }
                    resp_int(count)
                }
            }
            "COMMAND" | "COMMAND DOCS" => resp_ok(),
            "QUIT" => {
                let _ = writer.write_all(resp_ok().as_bytes());
                break;
            }
            _ => resp_err("unknown command"),
        };
        let _ = writer.write_all(response.as_bytes());
    }
}
"#.to_string(),
                        is_executable: false,
                    },
                ],
                tests: vec![],
                solution_files: vec![],
            },
            ProjectStage {
                name: "Stage 3: Expiration".to_string(),
                description: "Add TTL support so keys can expire after a specified number of seconds.".to_string(),
                files: vec![
                    ProjectFile {
                        path: "src/main.rs".to_string(),
                        content: r#"use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

struct Entry {
    value: String,
    expires_at: Option<Instant>,
}

struct Store {
    data: HashMap<String, Entry>,
}

impl Store {
    fn get(&self, key: &str) -> Option<&str> {
        self.data.get(key).and_then(|entry| {
            if let Some(expires) = entry.expires_at {
                if Instant::now() >= expires {
                    return None;
                }
            }
            Some(entry.value.as_str())
        })
    }

    fn set(&mut self, key: String, value: String, ttl: Option<Duration>) {
        let expires_at = ttl.map(|d| Instant::now() + d);
        self.data.insert(key, Entry { value, expires_at });
    }

    fn del(&mut self, keys: &[&str]) -> i64 {
        let mut count = 0;
        for key in keys {
            if self.data.remove(*key).is_some() {
                count += 1;
            }
        }
        count
    }

    fn cleanup(&mut self) {
        let now = Instant::now();
        self.data.retain(|_, entry| {
            entry.expires_at.map_or(true, |exp| now < exp)
        });
    }
}

fn parse_resp(reader: &mut BufReader<&TcpStream>) -> Option<Vec<String>> {
    let mut line = String::new();
    reader.read_line(&mut line).ok()?;
    let line = line.trim();
    if !line.starts_with('*') {
        return Some(vec![line.to_string()]);
    }
    let count: usize = line[1..].parse().ok()?;
    let mut elements = Vec::new();
    for _ in 0..count {
        let mut len_line = String::new();
        reader.read_line(&mut len_line).ok()?;
        let len: usize = len_line.trim().trim_start_matches('$').parse().ok()?;
        let mut buf = vec![0u8; len];
        reader.read_exact(&mut buf).ok()?;
        let mut crlf = [0u8; 2];
        reader.read_exact(&mut crlf).ok()?;
        elements.push(String::from_utf8_lossy(&buf).to_string());
    }
    Some(elements)
}

fn resp_bulk(data: &str) -> String { format!("${}\r\n{}\r\n", data.len(), data) }
fn resp_int(val: i64) -> String { format!(":{}\r\n", val) }
fn resp_ok() -> String { "+OK\r\n".to_string() }
fn resp_err(msg: &str) -> String { format!("-ERR {}\r\n", msg) }
fn resp_null() -> String { "$-1\r\n".to_string() }

fn main() {
    let store = Arc::new(Mutex::new(Store { data: HashMap::new() }));
    let listener = TcpListener::bind("127.0.0.1:6379").expect("Failed to bind");
    println!("Listening on 127.0.0.1:6379");
    for stream in listener.incoming().flatten() {
        let store = store.clone();
        std::thread::spawn(move || handle_client(stream, store));
    }
}

fn handle_client(stream: TcpStream, store: Arc<Mutex<Store>>) {
    let mut writer = stream.try_clone().unwrap();
    let mut reader = BufReader::new(&stream);
    loop {
        let Some(args) = parse_resp(&mut reader) else { break };
        if args.is_empty() { continue; }
        let cmd = args[0].to_uppercase();
        let response = match cmd.as_str() {
            "PING" => resp_ok(),
            "SET" => {
                if args.len() < 3 {
                    resp_err("wrong number of arguments for 'SET'")
                } else {
                    let mut ttl = None;
                    let mut i = 3;
                    while i < args.len() {
                        if args[i].to_uppercase() == "EX" && i + 1 < args.len() {
                            if let Ok(secs) = args[i + 1].parse::<u64>() {
                                ttl = Some(Duration::from_secs(secs));
                            }
                            i += 2;
                        } else {
                            i += 1;
                        }
                    }
                    store.lock().unwrap().set(args[1].clone(), args[2].clone(), ttl);
                    resp_ok()
                }
            }
            "GET" => {
                if args.len() < 2 {
                    resp_err("wrong number of arguments for 'GET'")
                } else {
                    let s = store.lock().unwrap();
                    match s.get(&args[1]) {
                        Some(v) => resp_bulk(v),
                        None => resp_null(),
                    }
                }
            }
            "DEL" => {
                if args.len() < 2 {
                    resp_err("wrong number of arguments for 'DEL'")
                } else {
                    let key_refs: Vec<&str> = args[1..].iter().map(|s| s.as_str()).collect();
                    let mut s = store.lock().unwrap();
                    resp_int(s.del(&key_refs))
                }
            }
            "COMMAND" | "COMMAND DOCS" => resp_ok(),
            "QUIT" => {
                let _ = writer.write_all(resp_ok().as_bytes());
                break;
            }
            _ => resp_err("unknown command"),
        };
        let _ = writer.write_all(response.as_bytes());
    }
}
"#.to_string(),
                        is_executable: false,
                    },
                ],
                tests: vec![],
                solution_files: vec![],
            },
            ProjectStage {
                name: "Stage 4: Persistence".to_string(),
                description: "Save the key-value store to disk and reload it on startup.".to_string(),
                files: vec![
                    ProjectFile {
                        path: "src/main.rs".to_string(),
                        content: r#"use std::collections::HashMap;
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

struct Entry {
    value: String,
    expires_at: Option<Instant>,
    original_ttl: Option<u64>,
}

struct Store {
    data: HashMap<String, Entry>,
}

impl Store {
    fn save(&self, path: &Path) -> std::io::Result<()> {
        let mut content = String::new();
        for (key, entry) in &self.data {
            if let Some(expires) = entry.expires_at {
                let remaining = expires.saturating_duration_since(Instant::now());
                content.push_str(&format!("{}|{}|{}\n", key, entry.value, remaining.as_secs()));
            } else {
                content.push_str(&format!("{}|{}|\n", key, entry.value));
            }
        }
        fs::write(path, content)
    }

    fn load(path: &Path) -> Self {
        let mut data = HashMap::new();
        if let Ok(content) = fs::read_to_string(path) {
            for line in content.lines() {
                let parts: Vec<&str> = line.splitn(3, '|').collect();
                if parts.len() >= 2 {
                    let expires_at = if !parts[2].is_empty() {
                        parts[2].parse::<u64>().ok().map(|secs| Instant::now() + Duration::from_secs(secs))
                    } else {
                        None
                    };
                    data.insert(parts[0].to_string(), Entry {
                        value: parts[1].to_string(),
                        expires_at,
                        original_ttl: None,
                    });
                }
            }
        }
        Store { data }
    }

    fn get(&self, key: &str) -> Option<&str> {
        self.data.get(key).and_then(|entry| {
            if let Some(expires) = entry.expires_at {
                if Instant::now() >= expires {
                    return None;
                }
            }
            Some(entry.value.as_str())
        })
    }

    fn set(&mut self, key: String, value: String, ttl: Option<Duration>) {
        let expires_at = ttl.map(|d| Instant::now() + d);
        let original_ttl = ttl.map(|d| d.as_secs());
        self.data.insert(key, Entry { value, expires_at, original_ttl });
    }
}

fn parse_resp(reader: &mut BufReader<&TcpStream>) -> Option<Vec<String>> {
    let mut line = String::new();
    reader.read_line(&mut line).ok()?;
    let line = line.trim();
    if !line.starts_with('*') {
        return Some(vec![line.to_string()]);
    }
    let count: usize = line[1..].parse().ok()?;
    let mut elements = Vec::new();
    for _ in 0..count {
        let mut len_line = String::new();
        reader.read_line(&mut len_line).ok()?;
        let len: usize = len_line.trim().trim_start_matches('$').parse().ok()?;
        let mut buf = vec![0u8; len];
        reader.read_exact(&mut buf).ok()?;
        let mut crlf = [0u8; 2];
        reader.read_exact(&mut crlf).ok()?;
        elements.push(String::from_utf8_lossy(&buf).to_string());
    }
    Some(elements)
}

fn resp_bulk(data: &str) -> String { format!("${}\r\n{}\r\n", data.len(), data) }
fn resp_int(val: i64) -> String { format!(":{}\r\n", val) }
fn resp_ok() -> String { "+OK\r\n".to_string() }
fn resp_err(msg: &str) -> String { format!("-ERR {}\r\n", msg) }
fn resp_null() -> String { "$-1\r\n".to_string() }

fn main() {
    let db_path = Path::new("dump.rdb");
    let store = Arc::new(Mutex::new(Store::load(db_path)));
    let store_clone = store.clone();
    std::thread::spawn(move || {
        loop {
            std::thread::sleep(Duration::from_secs(60));
            let s = store_clone.lock().unwrap();
            let _ = s.save(db_path);
        }
    });
    let listener = TcpListener::bind("127.0.0.1:6379").expect("Failed to bind");
    println!("Listening on 127.0.0.1:6379");
    for stream in listener.incoming().flatten() {
        let store = store.clone();
        std::thread::spawn(move || handle_client(stream, store, db_path));
    }
}

fn handle_client(stream: TcpStream, store: Arc<Mutex<Store>>, db_path: &Path) {
    let mut writer = stream.try_clone().unwrap();
    let mut reader = BufReader::new(&stream);
    loop {
        let Some(args) = parse_resp(&mut reader) else { break };
        if args.is_empty() { continue; }
        let cmd = args[0].to_uppercase();
        let response = match cmd.as_str() {
            "PING" => resp_ok(),
            "SET" => {
                if args.len() < 3 { resp_err("wrong number of arguments for 'SET'") }
                else {
                    let mut ttl = None;
                    let mut i = 3;
                    while i < args.len() {
                        if args[i].to_uppercase() == "EX" && i + 1 < args.len() {
                            if let Ok(secs) = args[i + 1].parse::<u64>() {
                                ttl = Some(Duration::from_secs(secs));
                            }
                            i += 2;
                        } else { i += 1; }
                    }
                    store.lock().unwrap().set(args[1].clone(), args[2].clone(), ttl);
                    resp_ok()
                }
            }
            "GET" => {
                if args.len() < 2 { resp_err("wrong number of arguments for 'GET'") }
                else {
                    let s = store.lock().unwrap();
                    match s.get(&args[1]) { Some(v) => resp_bulk(v), None => resp_null() }
                }
            }
            "DEL" => {
                if args.len() < 2 { resp_err("wrong number of arguments for 'DEL'") }
                else {
                    let mut s = store.lock().unwrap();
                    let mut count = 0i64;
                    for key in &args[1..] { if s.data.remove(key.as_str()).is_some() { count += 1; } }
                    resp_int(count)
                }
            }
            "SAVE" => {
                let s = store.lock().unwrap();
                match s.save(db_path) { Ok(()) => resp_ok(), Err(e) => resp_err(&e.to_string()) }
            }
            "COMMAND" | "COMMAND DOCS" => resp_ok(),
            "QUIT" => {
                let _ = writer.write_all(resp_ok().as_bytes());
                break;
            }
            _ => resp_err("unknown command"),
        };
        let _ = writer.write_all(response.as_bytes());
    }
}
"#.to_string(),
                        is_executable: false,
                    },
                ],
                tests: vec![],
                solution_files: vec![],
            },
            ProjectStage {
                name: "Stage 5: Lists".to_string(),
                description = "Implement LPUSH, RPUSH, LRANGE for list data structures.".to_string(),
                files: vec![
                    ProjectFile {
                        path: "src/main.rs".to_string(),
                        content: r#"use std::collections::HashMap;
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::path::Path;
use std::sync::{Arc, Mutex};

enum Value {
    String(String),
    List(Vec<String>),
}

struct Store {
    data: HashMap<String, Value>,
}

impl Store {
    fn save(&self, path: &Path) -> std::io::Result<()> {
        let mut content = String::new();
        for (key, val) in &self.data {
            match val {
                Value::String(s) => content.push_str(&format!("S|{}|{}\n", key, s)),
                Value::List(list) => {
                    content.push_str(&format!("L|{}|{}\n", key, list.join("\t")));
                }
            }
        }
        fs::write(path, content)
    }

    fn load(path: &Path) -> Self {
        let mut data = HashMap::new();
        if let Ok(content) = fs::read_to_string(path) {
            for line in content.lines() {
                let parts: Vec<&str> = line.splitn(3, '|').collect();
                if parts.len() >= 3 {
                    match parts[0] {
                        "S" => { data.insert(parts[1].to_string(), Value::String(parts[2].to_string())); }
                        "L" => {
                            let list: Vec<String> = parts[2].split('\t').map(|s| s.to_string()).collect();
                            data.insert(parts[1].to_string(), Value::List(list));
                        }
                        _ => {}
                    }
                }
            }
        }
        Store { data }
    }
}

fn parse_resp(reader: &mut BufReader<&TcpStream>) -> Option<Vec<String>> {
    let mut line = String::new();
    reader.read_line(&mut line).ok()?;
    let line = line.trim();
    if !line.starts_with('*') {
        return Some(vec![line.to_string()]);
    }
    let count: usize = line[1..].parse().ok()?;
    let mut elements = Vec::new();
    for _ in 0..count {
        let mut len_line = String::new();
        reader.read_line(&mut len_line).ok()?;
        let len: usize = len_line.trim().trim_start_matches('$').parse().ok()?;
        let mut buf = vec![0u8; len];
        reader.read_exact(&mut buf).ok()?;
        let mut crlf = [0u8; 2];
        reader.read_exact(&mut crlf).ok()?;
        elements.push(String::from_utf8_lossy(&buf).to_string());
    }
    Some(elements)
}

fn resp_bulk(data: &str) -> String { format!("${}\r\n{}\r\n", data.len(), data) }
fn resp_int(val: i64) -> String { format!(":{}\r\n", val) }
fn resp_ok() -> String { "+OK\r\n".to_string() }
fn resp_err(msg: &str) -> String { format!("-ERR {}\r\n", msg) }
fn resp_null() -> String { "$-1\r\n".to_string() }
fn resp_array(items: &[String]) -> String {
    let mut out = format!("*{}\r\n", items.len());
    for item in items {
        out.push_str(&resp_bulk(item));
    }
    out
}

fn main() {
    let db_path = Path::new("dump.rdb");
    let store = Arc::new(Mutex::new(Store::load(db_path)));
    let listener = TcpListener::bind("127.0.0.1:6379").expect("Failed to bind");
    println!("Listening on 127.0.0.1:6379");
    for stream in listener.incoming().flatten() {
        let store = store.clone();
        std::thread::spawn(move || handle_client(stream, store, db_path));
    }
}

fn handle_client(stream: TcpStream, store: Arc<Mutex<Store>>, db_path: &Path) {
    let mut writer = stream.try_clone().unwrap();
    let mut reader = BufReader::new(&stream);
    loop {
        let Some(args) = parse_resp(&mut reader) else { break };
        if args.is_empty() { continue; }
        let cmd = args[0].to_uppercase();
        let response = match cmd.as_str() {
            "PING" => resp_ok(),
            "SET" => {
                if args.len() < 3 { resp_err("wrong number of arguments") }
                else {
                    store.lock().unwrap().data.insert(args[1].clone(), Value::String(args[2].clone()));
                    resp_ok()
                }
            }
            "GET" => {
                if args.len() < 2 { resp_err("wrong number of arguments") }
                else {
                    let s = store.lock().unwrap();
                    match s.data.get(&args[1]) {
                        Some(Value::String(v)) => resp_bulk(v),
                        _ => resp_null(),
                    }
                }
            }
            "DEL" => {
                if args.len() < 2 { resp_err("wrong number of arguments") }
                else {
                    let mut s = store.lock().unwrap();
                    let mut count = 0i64;
                    for key in &args[1..] { if s.data.remove(key.as_str()).is_some() { count += 1; } }
                    resp_int(count)
                }
            }
            "LPUSH" => {
                if args.len() < 3 { resp_err("wrong number of arguments") }
                else {
                    let mut s = store.lock().unwrap();
                    let list = s.data.entry(args[1].clone()).or_insert_with(|| Value::List(Vec::new()));
                    if let Value::List(ref mut l) = *list {
                        for val in args[2..].iter().rev() {
                            l.insert(0, val.clone());
                        }
                        resp_int(l.len() as i64)
                    } else {
                        resp_err("wrong type")
                    }
                }
            }
            "RPUSH" => {
                if args.len() < 3 { resp_err("wrong number of arguments") }
                else {
                    let mut s = store.lock().unwrap();
                    let list = s.data.entry(args[1].clone()).or_insert_with(|| Value::List(Vec::new()));
                    if let Value::List(ref mut l) = *list {
                        for val in &args[2..] {
                            l.push(val.clone());
                        }
                        resp_int(l.len() as i64)
                    } else {
                        resp_err("wrong type")
                    }
                }
            }
            "LRANGE" => {
                if args.len() < 4 { resp_err("wrong number of arguments") }
                else {
                    let s = store.lock().unwrap();
                    match s.data.get(&args[1]) {
                        Some(Value::List(l)) => {
                            let start: isize = args[2].parse().unwrap_or(0);
                            let stop: isize = args[3].parse().unwrap_or(-1);
                            let len = l.len() as isize;
                            let start = if start < 0 { (len + start).max(0) } else { start.min(len) } as usize;
                            let stop = if stop < 0 { (len + stop + 1).max(0) } else { (stop + 1).min(len) } as usize;
                            resp_array(&l[start..stop])
                        }
                        _ => resp_array(&[]),
                    }
                }
            }
            "SAVE" => {
                let s = store.lock().unwrap();
                match s.save(db_path) { Ok(()) => resp_ok(), Err(e) => resp_err(&e.to_string()) }
            }
            "COMMAND" | "COMMAND DOCS" => resp_ok(),
            "QUIT" => { let _ = writer.write_all(resp_ok().as_bytes()); break; }
            _ => resp_err("unknown command"),
        };
        let _ = writer.write_all(response.as_bytes());
    }
}
"#.to_string(),
                        is_executable: false,
                    },
                ],
                tests: vec![],
                solution_files: vec![],
            },
        ],
    }
}

fn compiler_project() -> ProjectPack {
    ProjectPack {
        id: "build-your-own-compiler".to_string(),
        name: "Build Your Own Compiler".to_string(),
        description: "Implement a simple expression compiler with lexer, parser, and code generation.".to_string(),
        difficulty: "hard".to_string(),
        language: "rust".to_string(),
        readme: "# Build Your Own Compiler\n\nImplement a simple compiler for arithmetic expressions in Rust.\n\n## Stages\n\n1. **Lexer** — Tokenize input into numbers, operators, and parentheses\n2. **Parser** — Build an AST using recursive descent parsing\n3. **Evaluator** — Walk the AST to compute results\n4. **Variables** — Add variable assignment and lookup\n5. **Code Generator** — Emit stack-based virtual machine bytecode\n\n## Grammar\n\n```\nprogram    = statement*\nstatement  = IDENT '=' expr | expr\nexpr       = term (('+' | '-') term)*\nterm       = factor (('*' | '/') factor)*\nfactor     = NUMBER | IDENT | '(' expr ')' | '-' factor\n```\n\n## Getting Started\n\nStart with the lexer and work up to code generation.".to_string(),
        stages: vec![
            ProjectStage {
                name: "Stage 1: Lexer".to_string(),
                description = "Tokenize input strings into numbers, operators, and parentheses.".to_string(),
                files: vec![
                    ProjectFile {
                        path: "Cargo.toml".to_string(),
                        content: r#"[package]
name = "my-compiler"
version = "0.1.0"
edition = "2021"
"#.to_string(),
                        is_executable: false,
                    },
                    ProjectFile {
                        path: "src/main.rs".to_string(),
                        content: r#"#[derive(Debug, Clone, PartialEq)]
enum Token {
    Number(f64),
    Plus,
    Minus,
    Star,
    Slash,
    LParen,
    RParen,
    Eof,
}

struct Lexer {
    input: Vec<char>,
    pos: usize,
}

impl Lexer {
    fn new(input: &str) -> Self {
        Self {
            input: input.chars().collect(),
            pos: 0,
        }
    }

    fn next_token(&mut self) -> Token {
        self.skip_whitespace();
        if self.pos >= self.input.len() {
            return Token::Eof;
        }
        let ch = self.input[self.pos];
        match ch {
            '+' => { self.pos += 1; Token::Plus }
            '-' => { self.pos += 1; Token::Minus }
            '*' => { self.pos += 1; Token::Star }
            '/' => { self.pos += 1; Token::Slash }
            '(' => { self.pos += 1; Token::LParen }
            ')' => { self.pos += 1; Token::RParen }
            c if c.is_ascii_digit() || c == '.' => self.read_number(),
            _ => {
                eprintln!("Unexpected character: '{}'", ch);
                self.pos += 1;
                Token::Eof
            }
        }
    }

    fn read_number(&mut self) -> Token {
        let start = self.pos;
        while self.pos < self.input.len() && (self.input[self.pos].is_ascii_digit() || self.input[self.pos] == '.') {
            self.pos += 1;
        }
        let num_str: String = self.input[start..self.pos].iter().collect();
        Token::Number(num_str.parse().unwrap_or(0.0))
    }

    fn skip_whitespace(&mut self) {
        while self.pos < self.input.len() && self.input[self.pos].is_whitespace() {
            self.pos += 1;
        }
    }

    fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        loop {
            let token = self.next_token();
            if token == Token::Eof {
                tokens.push(Token::Eof);
                break;
            }
            tokens.push(token);
        }
        tokens
    }
}

fn main() {
    let input = std::env::args().skip(1).next().unwrap_or_else(|| {
        eprintln!("Usage: my-compiler <expression>");
        std::process::exit(1);
    });
    let mut lexer = Lexer::new(&input);
    let tokens = lexer.tokenize();
    println!("Tokens:");
    for token in &tokens {
        println!("  {:?}", token);
    }
}
"#.to_string(),
                        is_executable: false,
                    },
                ],
                tests: vec![],
                solution_files: vec![],
            },
            ProjectStage {
                name = "Stage 2: Parser",
                description = "Parse tokens into an Abstract Syntax Tree using recursive descent.".to_string(),
                files: vec![
                    ProjectFile {
                        path = "src/main.rs".to_string(),
                        content: r#"#[derive(Debug, Clone, PartialEq)]
enum Token {
    Number(f64),
    Plus,
    Minus,
    Star,
    Slash,
    LParen,
    RParen,
    Eof,
}

#[derive(Debug, Clone)]
enum Expr {
    Number(f64),
    BinaryOp {
        op: char,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    UnaryOp {
        op: char,
        operand: Box<Expr>,
    },
}

struct Lexer { input: Vec<char>, pos: usize }

impl Lexer {
    fn new(input: &str) -> Self { Self { input: input.chars().collect(), pos: 0 } }

    fn next_token(&mut self) -> Token {
        self.skip_whitespace();
        if self.pos >= self.input.len() { return Token::Eof; }
        let ch = self.input[self.pos];
        match ch {
            '+' => { self.pos += 1; Token::Plus }
            '-' => { self.pos += 1; Token::Minus }
            '*' => { self.pos += 1; Token::Star }
            '/' => { self.pos += 1; Token::Slash }
            '(' => { self.pos += 1; Token::LParen }
            ')' => { self.pos += 1; Token::RParen }
            c if c.is_ascii_digit() || c == '.' => self.read_number(),
            _ => { self.pos += 1; Token::Eof }
        }
    }

    fn read_number(&mut self) -> Token {
        let start = self.pos;
        while self.pos < self.input.len() && (self.input[self.pos].is_ascii_digit() || self.input[self.pos] == '.') {
            self.pos += 1;
        }
        Token::Number(self.input[start..self.pos].iter().collect::<String>().parse().unwrap_or(0.0))
    }

    fn skip_whitespace(&mut self) {
        while self.pos < self.input.len() && self.input[self.pos].is_whitespace() { self.pos += 1; }
    }

    fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        loop {
            let token = self.next_token();
            if token == Token::Eof { tokens.push(Token::Eof); break; }
            tokens.push(token);
        }
        tokens
    }
}

struct Parser { tokens: Vec<Token>, pos: usize }

impl Parser {
    fn new(tokens: Vec<Token>) -> Self { Self { tokens, pos: 0 } }

    fn peek(&self) -> &Token { self.tokens.get(self.pos).unwrap_or(&Token::Eof) }

    fn consume(&mut self) -> Token {
        let token = self.tokens.get(self.pos).cloned().unwrap_or(Token::Eof);
        self.pos += 1;
        token
    }

    fn parse_expr(&mut self) -> Expr {
        let mut left = self.parse_term();
        loop {
            match self.peek() {
                Token::Plus => { self.consume(); let right = self.parse_term(); left = Expr::BinaryOp { op: '+', left: Box::new(left), right: Box::new(right) }; }
                Token::Minus => { self.consume(); let right = self.parse_term(); left = Expr::BinaryOp { op: '-', left: Box::new(left), right: Box::new(right) }; }
                _ => break,
            }
        }
        left
    }

    fn parse_term(&mut self) -> Expr {
        let mut left = self.parse_factor();
        loop {
            match self.peek() {
                Token::Star => { self.consume(); let right = self.parse_factor(); left = Expr::BinaryOp { op: '*', left: Box::new(left), right: Box::new(right) }; }
                Token::Slash => { self.consume(); let right = self.parse_factor(); left = Expr::BinaryOp { op: '/', left: Box::new(left), right: Box::new(right) }; }
                _ => break,
            }
        }
        left
    }

    fn parse_factor(&mut self) -> Expr {
        match self.peek().clone() {
            Token::Number(n) => { self.consume(); Expr::Number(n) }
            Token::Minus => { self.consume(); let operand = self.parse_factor(); Expr::UnaryOp { op: '-', operand: Box::new(operand) } }
            Token::LParen => {
                self.consume();
                let expr = self.parse_expr();
                self.consume();
                expr
            }
            _ => Expr::Number(0.0),
        }
    }
}

fn main() {
    let input = std::env::args().skip(1).next().unwrap_or_else(|| {
        eprintln!("Usage: my-compiler <expression>");
        std::process::exit(1);
    });
    let mut lexer = Lexer::new(&input);
    let tokens = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let ast = parser.parse_expr();
    println!("{:#?}", ast);
}
"#.to_string(),
                        is_executable = false,
                    },
                ],
                tests = vec![],
                solution_files = vec![],
            },
            ProjectStage {
                name = "Stage 3: Evaluator",
                description = "Walk the AST to evaluate expression results.".to_string(),
                files = vec![
                    ProjectFile {
                        path = "src/main.rs".to_string(),
                        content = r#"#[derive(Debug, Clone, PartialEq)]
enum Token { Number(f64), Plus, Minus, Star, Slash, LParen, RParen, Eof }

#[derive(Debug, Clone)]
enum Expr {
    Number(f64),
    BinaryOp { op: char, left: Box<Expr>, right: Box<Expr> },
    UnaryOp { op: char, operand: Box<Expr> },
}

struct Lexer { input: Vec<char>, pos: usize }

impl Lexer {
    fn new(input: &str) -> Self { Self { input: input.chars().collect(), pos: 0 } }
    fn next_token(&mut self) -> Token {
        self.skip_whitespace();
        if self.pos >= self.input.len() { return Token::Eof; }
        let ch = self.input[self.pos];
        match ch {
            '+' => { self.pos += 1; Token::Plus }
            '-' => { self.pos += 1; Token::Minus }
            '*' => { self.pos += 1; Token::Star }
            '/' => { self.pos += 1; Token::Slash }
            '(' => { self.pos += 1; Token::LParen }
            ')' => { self.pos += 1; Token::RParen }
            c if c.is_ascii_digit() || c == '.' => self.read_number(),
            _ => { self.pos += 1; Token::Eof }
        }
    }
    fn read_number(&mut self) -> Token {
        let start = self.pos;
        while self.pos < self.input.len() && (self.input[self.pos].is_ascii_digit() || self.input[self.pos] == '.') { self.pos += 1; }
        Token::Number(self.input[start..self.pos].iter().collect::<String>().parse().unwrap_or(0.0))
    }
    fn skip_whitespace(&mut self) { while self.pos < self.input.len() && self.input[self.pos].is_whitespace() { self.pos += 1; } }
    fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        loop { let t = self.next_token(); if t == Token::Eof { tokens.push(Token::Eof); break; } tokens.push(t); }
        tokens
    }
}

struct Parser { tokens: Vec<Token>, pos: usize }

impl Parser {
    fn new(tokens: Vec<Token>) -> Self { Self { tokens, pos: 0 } }
    fn peek(&self) -> &Token { self.tokens.get(self.pos).unwrap_or(&Token::Eof) }
    fn consume(&mut self) -> Token { let t = self.tokens.get(self.pos).cloned().unwrap_or(Token::Eof); self.pos += 1; t }
    fn parse_expr(&mut self) -> Expr {
        let mut left = self.parse_term();
        loop { match self.peek() { Token::Plus => { self.consume(); let r = self.parse_term(); left = Expr::BinaryOp { op: '+', left: Box::new(left), right: Box::new(r) }; } Token::Minus => { self.consume(); let r = self.parse_term(); left = Expr::BinaryOp { op: '-', left: Box::new(left), right: Box::new(r) }; } _ => break, } }
        left
    }
    fn parse_term(&mut self) -> Expr {
        let mut left = self.parse_factor();
        loop { match self.peek() { Token::Star => { self.consume(); let r = self.parse_factor(); left = Expr::BinaryOp { op: '*', left: Box::new(left), right: Box::new(r) }; } Token::Slash => { self.consume(); let r = self.parse_factor(); left = Expr::BinaryOp { op: '/', left: Box::new(left), right: Box::new(r) }; } _ => break, } }
        left
    }
    fn parse_factor(&mut self) -> Expr {
        match self.peek().clone() { Token::Number(n) => { self.consume(); Expr::Number(n) } Token::Minus => { self.consume(); Expr::UnaryOp { op: '-', operand: Box::new(self.parse_factor()) } } Token::LParen => { self.consume(); let e = self.parse_expr(); self.consume(); e } _ => Expr::Number(0.0) }
    }
}

fn evaluate(expr: &Expr) -> f64 {
    match expr {
        Expr::Number(n) => *n,
        Expr::BinaryOp { op, left, right } => {
            let l = evaluate(left);
            let r = evaluate(right);
            match op { '+' => l + r, '-' => l - r, '*' => l * r, '/' => l / r, _ => 0.0 }
        }
        Expr::UnaryOp { op, operand } => {
            let v = evaluate(operand);
            if *op == '-' { -v } else { v }
        }
    }
}

fn main() {
    let input = std::env::args().skip(1).next().unwrap_or_else(|| { eprintln!("Usage: my-compiler <expr>"); std::process::exit(1); });
    let mut lexer = Lexer::new(&input);
    let tokens = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let ast = parser.parse_expr();
    println!("AST: {:#?}", ast);
    println!("Result: {}", evaluate(&ast));
}
"#.to_string(),
                        is_executable = false,
                    },
                ],
                tests = vec![],
                solution_files = vec![],
            },
            ProjectStage {
                name = "Stage 4: Variables",
                description = "Add variable assignment and lookup to the language.".to_string(),
                files = vec![
                    ProjectFile {
                        path = "src/main.rs".to_string(),
                        content = r#"use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
enum Token { Number(f64), Ident(String), Eq, Plus, Minus, Star, Slash, LParen, RParen, Semicolon, Eof }

#[derive(Debug, Clone)]
enum Expr {
    Number(f64),
    Ident(String),
    BinaryOp { op: char, left: Box<Expr>, right: Box<Expr> },
    UnaryOp { op: char, operand: Box<Expr> },
}

#[derive(Debug, Clone)]
enum Stmt {
    Assign { name: String, value: Expr },
    Expr(Expr),
}

struct Lexer { input: Vec<char>, pos: usize }

impl Lexer {
    fn new(input: &str) -> Self { Self { input: input.chars().collect(), pos: 0 } }
    fn next_token(&mut self) -> Token {
        self.skip_whitespace();
        if self.pos >= self.input.len() { return Token::Eof; }
        let ch = self.input[self.pos];
        match ch {
            '+' => { self.pos += 1; Token::Plus }
            '-' => { self.pos += 1; Token::Minus }
            '*' => { self.pos += 1; Token::Star }
            '/' => { self.pos += 1; Token::Slash }
            '(' => { self.pos += 1; Token::LParen }
            ')' => { self.pos += 1; Token::RParen }
            '=' => { self.pos += 1; Token::Eq }
            ';' => { self.pos += 1; Token::Semicolon }
            c if c.is_ascii_digit() || c == '.' => self.read_number(),
            c if c.is_ascii_alphabetic() => self.read_ident(),
            _ => { self.pos += 1; Token::Eof }
        }
    }
    fn read_number(&mut self) -> Token {
        let start = self.pos;
        while self.pos < self.input.len() && (self.input[self.pos].is_ascii_digit() || self.input[self.pos] == '.') { self.pos += 1; }
        Token::Number(self.input[start..self.pos].iter().collect::<String>().parse().unwrap_or(0.0))
    }
    fn read_ident(&mut self) -> Token {
        let start = self.pos;
        while self.pos < self.input.len() && self.input[self.pos].is_ascii_alphanumeric() { self.pos += 1; }
        Token::Ident(self.input[start..self.pos].iter().collect())
    }
    fn skip_whitespace(&mut self) { while self.pos < self.input.len() && self.input[self.pos].is_whitespace() { self.pos += 1; } }
    fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        loop { let t = self.next_token(); if t == Token::Eof { tokens.push(Token::Eof); break; } tokens.push(t); }
        tokens
    }
}

struct Parser { tokens: Vec<Token>, pos: usize }

impl Parser {
    fn new(tokens: Vec<Token>) -> Self { Self { tokens, pos: 0 } }
    fn peek(&self) -> &Token { self.tokens.get(self.pos).unwrap_or(&Token::Eof) }
    fn consume(&mut self) -> Token { let t = self.tokens.get(self.pos).cloned().unwrap_or(Token::Eof); self.pos += 1; t }

    fn parse_stmts(&mut self) -> Vec<Stmt> {
        let mut stmts = Vec::new();
        while *self.peek() != Token::Eof {
            stmts.push(self.parse_stmt());
            if *self.peek() == Token::Semicolon { self.consume(); }
        }
        stmts
    }

    fn parse_stmt(&mut self) -> Stmt {
        if let Token::Ident(name) = self.peek().clone() {
            let saved = self.pos;
            self.consume();
            if *self.peek() == Token::Eq {
                self.consume();
                let value = self.parse_expr();
                return Stmt::Assign { name, value };
            }
            self.pos = saved;
        }
        Stmt::Expr(self.parse_expr())
    }

    fn parse_expr(&mut self) -> Expr {
        let mut left = self.parse_term();
        loop { match self.peek() { Token::Plus => { self.consume(); let r = self.parse_term(); left = Expr::BinaryOp { op: '+', left: Box::new(left), right: Box::new(r) }; } Token::Minus => { self.consume(); let r = self.parse_term(); left = Expr::BinaryOp { op: '-', left: Box::new(left), right: Box::new(r) }; } _ => break, } }
        left
    }

    fn parse_term(&mut self) -> Expr {
        let mut left = self.parse_factor();
        loop { match self.peek() { Token::Star => { self.consume(); let r = self.parse_factor(); left = Expr::BinaryOp { op: '*', left: Box::new(left), right: Box::new(r) }; } Token::Slash => { self.consume(); let r = self.parse_factor(); left = Expr::BinaryOp { op: '/', left: Box::new(left), right: Box::new(r) }; } _ => break, } }
        left
    }

    fn parse_factor(&mut self) -> Expr {
        match self.peek().clone() {
            Token::Number(n) => { self.consume(); Expr::Number(n) }
            Token::Ident(name) => { self.consume(); Expr::Ident(name) }
            Token::Minus => { self.consume(); Expr::UnaryOp { op: '-', operand: Box::new(self.parse_factor()) } }
            Token::LParen => { self.consume(); let e = self.parse_expr(); self.consume(); e }
            _ => Expr::Number(0.0)
        }
    }
}

fn evaluate(expr: &Expr, env: &HashMap<String, f64>) -> f64 {
    match expr {
        Expr::Number(n) => *n,
        Expr::Ident(name) => env.get(name).copied().unwrap_or(0.0),
        Expr::BinaryOp { op, left, right } => {
            let l = evaluate(left, env);
            let r = evaluate(right, env);
            match op { '+' => l + r, '-' => l - r, '*' => l * r, '/' => l / r, _ => 0.0 }
        }
        Expr::UnaryOp { op, operand } => { let v = evaluate(operand, env); if *op == '-' { -v } else { v } }
    }
}

fn main() {
    let input = std::env::args().skip(1).collect::<Vec<_>>().join(" ");
    let input = if input.is_empty() { "x = 10; y = 20; x + y".to_string() } else { input };
    let mut lexer = Lexer::new(&input);
    let tokens = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let stmts = parser.parse_stmts();
    let mut env = HashMap::new();
    for stmt in &stmts {
        match stmt {
            Stmt::Assign { name, value } => {
                let val = evaluate(value, &env);
                env.insert(name.clone(), val);
                println!("{} = {}", name, val);
            }
            Stmt::Expr(expr) => {
                let val = evaluate(expr, &env);
                println!("= {}", val);
            }
        }
    }
}
"#.to_string(),
                        is_executable = false,
                    },
                ],
                tests = vec![],
                solution_files = vec![],
            },
            ProjectStage {
                name = "Stage 5: Code Generator",
                description = "Emit stack-based VM bytecode from the AST.".to_string(),
                files = vec![
                    ProjectFile {
                        path = "src/main.rs".to_string(),
                        content = r#"use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
enum Token { Number(f64), Ident(String), Eq, Plus, Minus, Star, Slash, LParen, RParen, Semicolon, Eof }

#[derive(Debug, Clone)]
enum Expr {
    Number(f64),
    Ident(String),
    BinaryOp { op: char, left: Box<Expr>, right: Box<Expr> },
    UnaryOp { op: char, operand: Box<Expr> },
}

#[derive(Debug, Clone)]
enum Stmt {
    Assign { name: String, value: Expr },
    Expr(Expr),
}

#[derive(Debug, Clone)]
enum Bytecode {
    Push(f64),
    Load(String),
    Store(String),
    Add,
    Sub,
    Mul,
    Div,
    Neg,
    Pop,
}

struct Lexer { input: Vec<char>, pos: usize }

impl Lexer {
    fn new(input: &str) -> Self { Self { input: input.chars().collect(), pos: 0 } }
    fn next_token(&mut self) -> Token {
        self.skip_whitespace();
        if self.pos >= self.input.len() { return Token::Eof; }
        let ch = self.input[self.pos];
        match ch {
            '+' => { self.pos += 1; Token::Plus }
            '-' => { self.pos += 1; Token::Minus }
            '*' => { self.pos += 1; Token::Star }
            '/' => { self.pos += 1; Token::Slash }
            '(' => { self.pos += 1; Token::LParen }
            ')' => { self.pos += 1; Token::RParen }
            '=' => { self.pos += 1; Token::Eq }
            ';' => { self.pos += 1; Token::Semicolon }
            c if c.is_ascii_digit() || c == '.' => self.read_number(),
            c if c.is_ascii_alphabetic() => self.read_ident(),
            _ => { self.pos += 1; Token::Eof }
        }
    }
    fn read_number(&mut self) -> Token {
        let start = self.pos;
        while self.pos < self.input.len() && (self.input[self.pos].is_ascii_digit() || self.input[self.pos] == '.') { self.pos += 1; }
        Token::Number(self.input[start..self.pos].iter().collect::<String>().parse().unwrap_or(0.0))
    }
    fn read_ident(&mut self) -> Token {
        let start = self.pos;
        while self.pos < self.input.len() && self.input[self.pos].is_ascii_alphanumeric() { self.pos += 1; }
        Token::Ident(self.input[start..self.pos].iter().collect())
    }
    fn skip_whitespace(&mut self) { while self.pos < self.input.len() && self.input[self.pos].is_whitespace() { self.pos += 1; } }
    fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        loop { let t = self.next_token(); if t == Token::Eof { tokens.push(Token::Eof); break; } tokens.push(t); }
        tokens
    }
}

struct Parser { tokens: Vec<Token>, pos: usize }

impl Parser {
    fn new(tokens: Vec<Token>) -> Self { Self { tokens, pos: 0 } }
    fn peek(&self) -> &Token { self.tokens.get(self.pos).unwrap_or(&Token::Eof) }
    fn consume(&mut self) -> Token { let t = self.tokens.get(self.pos).cloned().unwrap_or(Token::Eof); self.pos += 1; t }
    fn parse_stmts(&mut self) -> Vec<Stmt> {
        let mut stmts = Vec::new();
        while *self.peek() != Token::Eof { stmts.push(self.parse_stmt()); if *self.peek() == Token::Semicolon { self.consume(); } }
        stmts
    }
    fn parse_stmt(&mut self) -> Stmt {
        if let Token::Ident(name) = self.peek().clone() {
            let saved = self.pos; self.consume();
            if *self.peek() == Token::Eq { self.consume(); let value = self.parse_expr(); return Stmt::Assign { name, value }; }
            self.pos = saved;
        }
        Stmt::Expr(self.parse_expr())
    }
    fn parse_expr(&mut self) -> Expr {
        let mut left = self.parse_term();
        loop { match self.peek() { Token::Plus => { self.consume(); let r = self.parse_term(); left = Expr::BinaryOp { op: '+', left: Box::new(left), right: Box::new(r) }; } Token::Minus => { self.consume(); let r = self.parse_term(); left = Expr::BinaryOp { op: '-', left: Box::new(left), right: Box::new(r) }; } _ => break, } }
        left
    }
    fn parse_term(&mut self) -> Expr {
        let mut left = self.parse_factor();
        loop { match self.peek() { Token::Star => { self.consume(); let r = self.parse_factor(); left = Expr::BinaryOp { op: '*', left: Box::new(left), right: Box::new(r) }; } Token::Slash => { self.consume(); let r = self.parse_factor(); left = Expr::BinaryOp { op: '/', left: Box::new(left), right: Box::new(r) }; } _ => break, } }
        left
    }
    fn parse_factor(&mut self) -> Expr {
        match self.peek().clone() {
            Token::Number(n) => { self.consume(); Expr::Number(n) }
            Token::Ident(name) => { self.consume(); Expr::Ident(name) }
            Token::Minus => { self.consume(); Expr::UnaryOp { op: '-', operand: Box::new(self.parse_factor()) } }
            Token::LParen => { self.consume(); let e = self.parse_expr(); self.consume(); e }
            _ => Expr::Number(0.0)
        }
    }
}

fn compile_expr(expr: &Expr, code: &mut Vec<Bytecode>) {
    match expr {
        Expr::Number(n) => code.push(Bytecode::Push(*n)),
        Expr::Ident(name) => code.push(Bytecode::Load(name.clone())),
        Expr::BinaryOp { op, left, right } => {
            compile_expr(left, code);
            compile_expr(right, code);
            match op { '+' => code.push(Bytecode::Add), '-' => code.push(Bytecode::Sub), '*' => code.push(Bytecode::Mul), '/' => code.push(Bytecode::Div), _ => {} }
        }
        Expr::UnaryOp { op, operand } => {
            compile_expr(operand, code);
            if *op == '-' { code.push(Bytecode::Neg); }
        }
    }
}

fn compile_stmts(stmts: &[Stmt]) -> Vec<Bytecode> {
    let mut code = Vec::new();
    for stmt in stmts {
        match stmt {
            Stmt::Assign { name, value } => {
                compile_expr(value, &mut code);
                code.push(Bytecode::Store(name.clone()));
                code.push(Bytecode::Pop);
            }
            Stmt::Expr(expr) => {
                compile_expr(expr, &mut code);
            }
        }
    }
    code
}

fn run_vm(code: &[Bytecode]) -> f64 {
    let mut stack: Vec<f64> = Vec::new();
    let mut vars: HashMap<String, f64> = HashMap::new();
    for op in code {
        match op {
            Bytecode::Push(n) => stack.push(*n),
            Bytecode::Load(name) => { stack.push(vars.get(name).copied().unwrap_or(0.0)); }
            Bytecode::Store(name) => { if let Some(val) = stack.pop() { vars.insert(name.clone(), val); } }
            Bytecode::Add => { let r = stack.pop().unwrap_or(0.0); let l = stack.pop().unwrap_or(0.0); stack.push(l + r); }
            Bytecode::Sub => { let r = stack.pop().unwrap_or(0.0); let l = stack.pop().unwrap_or(0.0); stack.push(l - r); }
            Bytecode::Mul => { let r = stack.pop().unwrap_or(0.0); let l = stack.pop().unwrap_or(0.0); stack.push(l * r); }
            Bytecode::Div => { let r = stack.pop().unwrap_or(1.0); let l = stack.pop().unwrap_or(0.0); stack.push(l / r); }
            Bytecode::Neg => { if let Some(v) = stack.pop() { stack.push(-v); } }
            Bytecode::Pop => { stack.pop(); }
        }
    }
    stack.pop().unwrap_or(0.0)
}

fn main() {
    let input = std::env::args().skip(1).collect::<Vec<_>>().join(" ");
    let input = if input.is_empty() { "x = 10; y = 20; x * 2 + y".to_string() } else { input };
    let mut lexer = Lexer::new(&input);
    let tokens = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let stmts = parser.parse_stmts();
    let bytecode = compile_stmts(&stmts);
    println!("Bytecode:");
    for (i, bc) in bytecode.iter().enumerate() {
        println!("  {:04}: {:?}", i, bc);
    }
    let result = run_vm(&bytecode);
    println!("Result: {}", result);
}
"#.to_string(),
                        is_executable = false,
                    },
                ],
                tests = vec![],
                solution_files = vec![],
            },
        ],
    }
}

impl ProjectEngine {
    pub fn new(data_dir: &Path) -> Self {
        let projects_dir = data_dir.join("projects");
        let _ = fs::create_dir_all(&projects_dir);
        let installed = Self::load_installed_projects(&projects_dir);
        Self {
            builtin: builtin_projects(),
            installed,
            projects_dir,
        }
    }

    fn load_installed_projects(projects_dir: &Path) -> Vec<ProjectPack> {
        let mut projects = Vec::new();
        if let Ok(entries) = fs::read_dir(projects_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let project_file = path.join("project.json");
                    if let Ok(data) = fs::read_to_string(&project_file) {
                        if let Ok(project) = serde_json::from_str::<ProjectPack>(&data) {
                            projects.push(project);
                        }
                    }
                }
            }
        }
        projects
    }

    pub fn list_available(&self) -> &[ProjectPack] {
        &self.builtin
    }

    pub fn list_installed(&self) -> &[ProjectPack] {
        &self.installed
    }

    pub fn get_project(&self, project_id: &str) -> Option<&ProjectPack> {
        self.builtin.iter().find(|p| p.id == project_id)
    }

    pub fn install_project(&mut self, project_id: &str) -> Result<(), String> {
        if self.installed.iter().any(|p| p.id == project_id) {
            return Err(format!("Project '{}' is already installed", project_id));
        }
        let project = self
            .builtin
            .iter()
            .find(|p| p.id == project_id)
            .ok_or_else(|| format!("Project '{}' not found", project_id))?
            .clone();
        let project_dir = self.projects_dir.join(project_id);
        fs::create_dir_all(&project_dir).map_err(|e| e.to_string())?;
        for (i, stage) in project.stages.iter().enumerate() {
            let stage_dir = project_dir.join(format!("stage_{}", i + 1));
            fs::create_dir_all(&stage_dir).map_err(|e| e.to_string())?;
            for file in &stage.files {
                let file_path = stage_dir.join(&file.path);
                if let Some(parent) = file_path.parent() {
                    fs::create_dir_all(parent).map_err(|e| e.to_string())?;
                }
                fs::write(&file_path, &file.content).map_err(|e| e.to_string())?;
            }
            for file in &stage.tests {
                let file_path = stage_dir.join(&file.path);
                if let Some(parent) = file_path.parent() {
                    fs::create_dir_all(parent).map_err(|e| e.to_string())?;
                }
                fs::write(&file_path, &file.content).map_err(|e| e.to_string())?;
            }
            let solution_dir = stage_dir.join(".solutions");
            fs::create_dir_all(&solution_dir).map_err(|e| e.to_string())?;
            for file in &stage.solution_files {
                let file_path = solution_dir.join(&file.path);
                if let Some(parent) = file_path.parent() {
                    fs::create_dir_all(parent).map_err(|e| e.to_string())?;
                }
                fs::write(&file_path, &file.content).map_err(|e| e.to_string())?;
            }
        }
        let readme_path = project_dir.join("README.md");
        fs::write(&readme_path, &project.readme).map_err(|e| e.to_string())?;
        let json = serde_json::to_string_pretty(&project).map_err(|e| e.to_string())?;
        fs::write(project_dir.join("project.json"), json).map_err(|e| e.to_string())?;
        self.installed.push(project);
        Ok(())
    }

    pub fn remove_project(&mut self, project_id: &str) -> Result<(), String> {
        let idx = self
            .installed
            .iter()
            .position(|p| p.id == project_id)
            .ok_or_else(|| format!("Project '{}' is not installed", project_id))?;
        let project_dir = self.projects_dir.join(project_id);
        fs::remove_dir_all(&project_dir).map_err(|e| e.to_string())?;
        self.installed.remove(idx);
        Ok(())
    }
}
