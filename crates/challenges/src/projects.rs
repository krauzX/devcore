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
        readme: "# Build Your Own Shell\n\nImplement a minimal Unix shell in Rust.\n\n## Stages\n\n1. Basic REPL: Read input, parse commands, execute external programs\n2. Built-ins: Implement cd, exit, echo, type\n3. Piping: Support | to chain commands\n4. Redirection: Support > and < for file I/O\n5. Job Control: Background processes with &\n\n## Getting Started\n\nEach stage builds on the previous one. Start with stage 1 and work your way up.\n\nRun tests with cargo test after implementing each stage.".to_string(),
        stages: vec![
            ProjectStage {
                name: "Stage 1: Basic REPL".to_string(),
                description: "Parse user input and execute external commands using std::process::Command.".to_string(),
                files: vec![
                    ProjectFile {
                        path: "Cargo.toml".to_string(),
                        content: "[package]\nname = \"my-shell\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[[bin]]\nname = \"shell\"\npath = \"src/main.rs\"\n".to_string(),
                        is_executable: false,
                    },
                    ProjectFile {
                        path: "src/main.rs".to_string(),
                        content: "use std::io::{self, Write};\n\nfn main() {\n    let stdin = io::stdin();\n    loop {\n        io::stdout().write_all(b\"$ \").unwrap();\n        io::stdout().flush().unwrap();\n        let mut input = String::new();\n        match stdin.read_line(&mut input) {\n            Ok(0) => break,\n            Ok(_) => {\n                let input = input.trim();\n                if input.is_empty() {\n                    continue;\n                }\n                execute_command(input);\n            }\n            Err(e) => {\n                eprintln!(\"Error reading input: {}\", e);\n                break;\n            }\n        }\n    }\n}\n\nfn execute_command(input: &str) {\n    let parts: Vec<&str> = input.split_whitespace().collect();\n    if parts.is_empty() {\n        return;\n    }\n    let program = parts[0];\n    let args = &parts[1..];\n    let _ = std::process::Command::new(program)\n        .args(args)\n        .status();\n}\n".to_string(),
                        is_executable: false,
                    },
                ],
                tests: vec![
                    ProjectFile {
                        path: "tests/test_basic.rs".to_string(),
                        content: "#[test]\nfn test_echo() {\n    assert!(true);\n}\n".to_string(),
                        is_executable: false,
                    },
                ],
                solution_files: vec![
                    ProjectFile {
                        path: "solution.rs".to_string(),
                        content: "// Solution: Execute external commands using std::process::Command\n// Complete the execute_command function to spawn child processes.\n".to_string(),
                        is_executable: false,
                    },
                ],
            },
            ProjectStage {
                name: "Stage 2: Built-ins".to_string(),
                description: "Implement cd, exit, echo, and type built-in commands.".to_string(),
                files: vec![
                    ProjectFile {
                        path: "src/main.rs".to_string(),
                        content: "use std::env;\nuse std::io::{self, Write};\n\nfn main() {\n    let stdin = io::stdin();\n    loop {\n        io::stdout().write_all(b\"$ \").unwrap();\n        io::stdout().flush().unwrap();\n        let mut input = String::new();\n        match stdin.read_line(&mut input) {\n            Ok(0) => break,\n            Ok(_) => {\n                let input = input.trim();\n                if input.is_empty() {\n                    continue;\n                }\n                execute_command(input);\n            }\n            Err(_) => break,\n        }\n    }\n}\n\nfn execute_command(input: &str) {\n    let parts: Vec<&str> = input.split_whitespace().collect();\n    if parts.is_empty() {\n        return;\n    }\n    // TODO: Match on parts[0] for \"exit\", \"echo\", \"cd\", \"type\"\n    // For unknown commands, use std::process::Command\n}\n".to_string(),
                        is_executable: false,
                    },
                ],
                tests: vec![
                    ProjectFile {
                        path: "tests/test_basic.rs".to_string(),
                        content: "#[test]\nfn test_placeholder() {\n    assert!(true);\n}\n".to_string(),
                        is_executable: false,
                    },
                ],
                solution_files: vec![
                    ProjectFile {
                        path: "solution.rs".to_string(),
                        content: "// Solution for Stage 2: Built-ins\n// Implement the TODO items in the skeleton code.\n".to_string(),
                        is_executable: false,
                    },
                ],
            },
            ProjectStage {
                name: "Stage 3: Piping".to_string(),
                description: "Support the pipe operator | to chain commands together.".to_string(),
                files: vec![
                    ProjectFile {
                        path: "src/main.rs".to_string(),
                        content: "use std::io::{self, Write};\nuse std::process::Command;\n\nfn main() {\n    loop {\n        io::stdout().write_all(b\"$ \").unwrap();\n        io::stdout().flush().unwrap();\n        let mut input = String::new();\n        match io::stdin().read_line(&mut input) {\n            Ok(0) => break,\n            Ok(_) => {\n                let input = input.trim();\n                if input.is_empty() {\n                    continue;\n                }\n                if input.contains('|') {\n                    execute_pipe(input);\n                } else {\n                    execute_single(input);\n                }\n            }\n            Err(_) => break,\n        }\n    }\n}\n\nfn execute_single(input: &str) {\n    // TODO: parse and run a single command\n}\n\nfn execute_pipe(input: &str) {\n    // TODO: split on |, pipe stdout of each into stdin of next\n}\n".to_string(),
                        is_executable: false,
                    },
                ],
                tests: vec![
                    ProjectFile {
                        path: "tests/test_basic.rs".to_string(),
                        content: "#[test]\nfn test_placeholder() {\n    assert!(true);\n}\n".to_string(),
                        is_executable: false,
                    },
                ],
                solution_files: vec![
                    ProjectFile {
                        path: "solution.rs".to_string(),
                        content: "// Solution for Stage 3: Piping\n// Implement the TODO items in the skeleton code.\n".to_string(),
                        is_executable: false,
                    },
                ],
            },
            ProjectStage {
                name: "Stage 4: Redirection".to_string(),
                description: "Support > for output redirection and < for input redirection.".to_string(),
                files: vec![
                    ProjectFile {
                        path: "src/main.rs".to_string(),
                        content: "use std::env;\nuse std::fs::File;\nuse std::io::{self, Read, Write};\n\nfn main() {\n    loop {\n        io::stdout().write_all(b\"$ \").unwrap();\n        io::stdout().flush().unwrap();\n        let mut input = String::new();\n        match io::stdin().read_line(&mut input) {\n            Ok(0) => break,\n            Ok(_) => {\n                let input = input.trim();\n                if input.is_empty() {\n                    continue;\n                }\n                let (cmd, redirect_out, redirect_in) = parse_redirection(input);\n                execute_with_redirect(&cmd, redirect_out.as_deref(), redirect_in.as_deref());\n            }\n            Err(_) => break,\n        }\n    }\n}\n\nfn parse_redirection(input: &str) -> (String, Option<String>, Option<String>) {\n    // TODO: parse > and < from the input string\n    (input.to_string(), None, None)\n}\n\nfn execute_with_redirect(cmd: &str, redirect_out: Option<&str>, redirect_in: Option<&str>) {\n    // TODO: execute command with redirected stdin/stdout\n}\n".to_string(),
                        is_executable: false,
                    },
                ],
                tests: vec![
                    ProjectFile {
                        path: "tests/test_basic.rs".to_string(),
                        content: "#[test]\nfn test_placeholder() {\n    assert!(true);\n}\n".to_string(),
                        is_executable: false,
                    },
                ],
                solution_files: vec![
                    ProjectFile {
                        path: "solution.rs".to_string(),
                        content: "// Solution for Stage 4: Redirection\n// Implement the TODO items in the skeleton code.\n".to_string(),
                        is_executable: false,
                    },
                ],
            },
            ProjectStage {
                name: "Stage 5: Job Control".to_string(),
                description: "Support background processes with & operator.".to_string(),
                files: vec![
                    ProjectFile {
                        path: "src/main.rs".to_string(),
                        content: "use std::io::{self, Write};\nuse std::process::Command;\n\nfn main() {\n    let mut background_jobs: Vec<std::process::Child> = Vec::new();\n    loop {\n        io::stdout().write_all(b\"$ \").unwrap();\n        io::stdout().flush().unwrap();\n        let mut input = String::new();\n        match io::stdin().read_line(&mut input) {\n            Ok(0) => break,\n            Ok(_) => {\n                let input = input.trim();\n                if input.is_empty() {\n                    continue;\n                }\n                // TODO: check if input ends with &\n                // If so, spawn as background process and add to background_jobs\n                // Otherwise execute in foreground\n            }\n            Err(_) => break,\n        }\n    }\n}\n".to_string(),
                        is_executable: false,
                    },
                ],
                tests: vec![
                    ProjectFile {
                        path: "tests/test_basic.rs".to_string(),
                        content: "#[test]\nfn test_placeholder() {\n    assert!(true);\n}\n".to_string(),
                        is_executable: false,
                    },
                ],
                solution_files: vec![
                    ProjectFile {
                        path: "solution.rs".to_string(),
                        content: "// Solution for Stage 5: Job Control\n// Implement the TODO items in the skeleton code.\n".to_string(),
                        is_executable: false,
                    },
                ],
            },
        ],
    }
}

fn git_project() -> ProjectPack {
    ProjectPack {
        id: "build-your-own-git".to_string(),
        name: "Build Your Own Git".to_string(),
        description: "Implement core Git functionality: init, add, commit, log, branch, checkout, and diff.".to_string(),
        difficulty: "hard".to_string(),
        language: "rust".to_string(),
        readme: "# Build Your Own Git\n\nImplement a simplified version of Git in Rust.\n\n## Stages\n\n1. Init: Initialize a new repository with a .git directory\n2. Add & Commit: Stage files and create commits with SHA-1 hashing\n3. Log & Status: View commit history and working tree status\n4. Branch & Checkout: Create branches and switch between them\n5. Diff: Show differences between commits and working tree\n\n## Getting Started\n\nEach stage builds on the previous one. Start with stage 1.".to_string(),
        stages: vec![
            ProjectStage {
                name: "Stage 1: Init".to_string(),
                description: "Initialize a new repository with the .git directory structure.".to_string(),
                files: vec![
                    ProjectFile {
                        path: "Cargo.toml".to_string(),
                        content: "[package]\nname = \"my-git\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[[bin]]\nname = \"mygit\"\npath = \"src/main.rs\"\n".to_string(),
                        is_executable: false,
                    },
                    ProjectFile {
                        path: "src/main.rs".to_string(),
                        content: "use std::env;\nuse std::fs;\nuse std::path::Path;\n\nfn main() {\n    let args: Vec<String> = env::args().collect();\n    if args.len() < 2 {\n        eprintln!(\"usage: mygit <command>\");\n        return;\n    }\n    match args[1].as_str() {\n        \"init\" => init_repo(),\n        cmd => eprintln!(\"mygit: '{}' is not a mygit command.\", cmd),\n    }\n}\n\nfn init_repo() {\n    // TODO: create .git/ directory with objects/, refs/, HEAD\n}\n".to_string(),
                        is_executable: false,
                    },
                ],
                tests: vec![
                    ProjectFile {
                        path: "tests/test_basic.rs".to_string(),
                        content: "#[test]\nfn test_placeholder() {\n    assert!(true);\n}\n".to_string(),
                        is_executable: false,
                    },
                ],
                solution_files: vec![
                    ProjectFile {
                        path: "solution.rs".to_string(),
                        content: "// Solution for Stage 1: Init\n// Implement the TODO items in the skeleton code.\n".to_string(),
                        is_executable: false,
                    },
                ],
            },
            ProjectStage {
                name: "Stage 2: Add & Commit".to_string(),
                description: "Stage files and create commits using SHA-1 hashing.".to_string(),
                files: vec![
                    ProjectFile {
                        path: "src/main.rs".to_string(),
                        content: "use std::env;\nuse std::fs;\nuse std::path::Path;\n\nfn main() {\n    let args: Vec<String> = env::args().collect();\n    if args.len() < 2 {\n        eprintln!(\"usage: mygit <command>\");\n        return;\n    }\n    match args[1].as_str() {\n        \"init\" => init_repo(),\n        \"add\" => {\n            // TODO: stage files by writing blob objects\n        }\n        \"commit\" => {\n            // TODO: create tree and commit objects\n        }\n        cmd => eprintln!(\"mygit: '{}' is not a mygit command.\", cmd),\n    }\n}\n\nfn init_repo() {\n    // TODO: create .git/ directory structure\n}\n".to_string(),
                        is_executable: false,
                    },
                ],
                tests: vec![
                    ProjectFile {
                        path: "tests/test_basic.rs".to_string(),
                        content: "#[test]\nfn test_placeholder() {\n    assert!(true);\n}\n".to_string(),
                        is_executable: false,
                    },
                ],
                solution_files: vec![
                    ProjectFile {
                        path: "solution.rs".to_string(),
                        content: "// Solution for Stage 2: Add & Commit\n// Implement the TODO items in the skeleton code.\n".to_string(),
                        is_executable: false,
                    },
                ],
            },
            ProjectStage {
                name: "Stage 3: Log & Status".to_string(),
                description: "View commit history and check working tree status.".to_string(),
                files: vec![
                    ProjectFile {
                        path: "src/main.rs".to_string(),
                        content: "use std::env;\nuse std::fs;\nuse std::path::Path;\n\nfn main() {\n    let args: Vec<String> = env::args().collect();\n    if args.len() < 2 {\n        eprintln!(\"usage: mygit <command>\");\n        return;\n    }\n    match args[1].as_str() {\n        \"init\" => init_repo(),\n        \"add\" => todo!(\"stage files\"),\n        \"commit\" => todo!(\"create commit\"),\n        \"log\" => {\n            // TODO: walk the commit chain from HEAD, printing each commit\n        }\n        \"status\" => {\n            // TODO: compare working tree to index and HEAD\n        }\n        cmd => eprintln!(\"mygit: '{}' is not a mygit command.\", cmd),\n    }\n}\n\nfn init_repo() {\n    // TODO\n}\n".to_string(),
                        is_executable: false,
                    },
                ],
                tests: vec![
                    ProjectFile {
                        path: "tests/test_basic.rs".to_string(),
                        content: "#[test]\nfn test_placeholder() {\n    assert!(true);\n}\n".to_string(),
                        is_executable: false,
                    },
                ],
                solution_files: vec![
                    ProjectFile {
                        path: "solution.rs".to_string(),
                        content: "// Solution for Stage 3: Log & Status\n// Implement the TODO items in the skeleton code.\n".to_string(),
                        is_executable: false,
                    },
                ],
            },
            ProjectStage {
                name: "Stage 4: Branch & Checkout".to_string(),
                description: "Create branches and switch between them.".to_string(),
                files: vec![
                    ProjectFile {
                        path: "src/main.rs".to_string(),
                        content: "use std::env;\nuse std::fs;\nuse std::path::Path;\n\nfn main() {\n    let args: Vec<String> = env::args().collect();\n    if args.len() < 2 {\n        eprintln!(\"usage: mygit <command>\");\n        return;\n    }\n    match args[1].as_str() {\n        \"init\" => init_repo(),\n        \"add\" => todo!(\"stage files\"),\n        \"commit\" => todo!(\"create commit\"),\n        \"log\" => todo!(\"show log\"),\n        \"status\" => todo!(\"show status\"),\n        \"branch\" => {\n            // TODO: create or list branches\n        }\n        \"checkout\" => {\n            // TODO: switch to a branch, update HEAD and working tree\n        }\n        cmd => eprintln!(\"mygit: '{}' is not a mygit command.\", cmd),\n    }\n}\n\nfn init_repo() {\n    // TODO\n}\n".to_string(),
                        is_executable: false,
                    },
                ],
                tests: vec![
                    ProjectFile {
                        path: "tests/test_basic.rs".to_string(),
                        content: "#[test]\nfn test_placeholder() {\n    assert!(true);\n}\n".to_string(),
                        is_executable: false,
                    },
                ],
                solution_files: vec![
                    ProjectFile {
                        path: "solution.rs".to_string(),
                        content: "// Solution for Stage 4: Branch & Checkout\n// Implement the TODO items in the skeleton code.\n".to_string(),
                        is_executable: false,
                    },
                ],
            },
            ProjectStage {
                name: "Stage 5: Diff".to_string(),
                description: "Show differences between commits and the working tree.".to_string(),
                files: vec![
                    ProjectFile {
                        path: "src/main.rs".to_string(),
                        content: "use std::env;\nuse std::fs;\nuse std::path::Path;\n\nfn main() {\n    let args: Vec<String> = env::args().collect();\n    if args.len() < 2 {\n        eprintln!(\"usage: mygit <command>\");\n        return;\n    }\n    match args[1].as_str() {\n        \"init\" => init_repo(),\n        \"add\" => todo!(\"stage files\"),\n        \"commit\" => todo!(\"create commit\"),\n        \"log\" => todo!(\"show log\"),\n        \"status\" => todo!(\"show status\"),\n        \"branch\" => todo!(\"create or list branches\"),\n        \"checkout\" => todo!(\"switch branch\"),\n        \"diff\" => {\n            // TODO: compute and display diffs\n        }\n        cmd => eprintln!(\"mygit: '{}' is not a mygit command.\", cmd),\n    }\n}\n\nfn init_repo() {\n    // TODO\n}\n".to_string(),
                        is_executable: false,
                    },
                ],
                tests: vec![
                    ProjectFile {
                        path: "tests/test_basic.rs".to_string(),
                        content: "#[test]\nfn test_placeholder() {\n    assert!(true);\n}\n".to_string(),
                        is_executable: false,
                    },
                ],
                solution_files: vec![
                    ProjectFile {
                        path: "solution.rs".to_string(),
                        content: "// Solution for Stage 5: Diff\n// Implement the TODO items in the skeleton code.\n".to_string(),
                        is_executable: false,
                    },
                ],
            },
        ],
    }
}

fn http_server_project() -> ProjectPack {
    ProjectPack {
        id: "build-your-own-http-server".to_string(),
        name: "Build Your Own HTTP Server".to_string(),
        description: "Implement an HTTP server from scratch using Rust's TcpListener.".to_string(),
        difficulty: "medium".to_string(),
        language: "rust".to_string(),
        readme: "# Build Your Own HTTP Server\n\nBuild an HTTP/1.1 server using only the standard library.\n\n## Stages\n\n1. TCP Listener: Accept connections and respond with a fixed response\n2. HTTP Parsing: Parse the request method, path, headers, and body\n3. Routing: Serve different responses based on method and path\n4. Static Files: Serve files from a directory\n5. Dynamic Response: Support query parameters and JSON responses\n\n## Getting Started\n\nEach stage builds on the previous one. Start with stage 1.".to_string(),
        stages: vec![
            ProjectStage {
                name: "Stage 1: TCP Listener".to_string(),
                description: "Accept TCP connections and return a fixed HTTP response.".to_string(),
                files: vec![
                    ProjectFile {
                        path: "Cargo.toml".to_string(),
                        content: "[package]\nname = \"my-http-server\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[[bin]]\nname = \"server\"\npath = \"src/main.rs\"\n".to_string(),
                        is_executable: false,
                    },
                    ProjectFile {
                        path: "src/main.rs".to_string(),
                        content: "use std::io::{Read, Write};\nuse std::net::TcpListener;\n\nfn main() {\n    let listener = TcpListener::bind(\"127.0.0.1:8080\").unwrap();\n    println!(\"Listening on port 8080\");\n    for stream in listener.incoming() {\n        match stream {\n            Ok(mut stream) => {\n                let mut buffer = [0; 1024];\n                stream.read(&mut buffer).unwrap();\n                // TODO: parse the request and send a proper HTTP response\n                let response = \"HTTP/1.1 200 OK\\r\\n\\r\\nHello, World!\";\n                stream.write_all(response.as_bytes()).unwrap();\n            }\n            Err(e) => eprintln!(\"Connection failed: {}\", e),\n        }\n    }\n}\n".to_string(),
                        is_executable: false,
                    },
                ],
                tests: vec![
                    ProjectFile {
                        path: "tests/test_basic.rs".to_string(),
                        content: "#[test]\nfn test_placeholder() {\n    assert!(true);\n}\n".to_string(),
                        is_executable: false,
                    },
                ],
                solution_files: vec![
                    ProjectFile {
                        path: "solution.rs".to_string(),
                        content: "// Solution for Stage 1: TCP Listener\n// Implement the TODO items in the skeleton code.\n".to_string(),
                        is_executable: false,
                    },
                ],
            },
            ProjectStage {
                name: "Stage 2: HTTP Parsing".to_string(),
                description: "Parse HTTP request method, path, headers, and body from raw bytes.".to_string(),
                files: vec![
                    ProjectFile {
                        path: "src/main.rs".to_string(),
                        content: "use std::collections::HashMap;\nuse std::io::{Read, Write};\nuse std::net::TcpListener;\n\nstruct HttpRequest {\n    method: String,\n    path: String,\n    headers: HashMap<String, String>,\n    body: String,\n}\n\nfn parse_request(raw: &str) -> Option<HttpRequest> {\n    // TODO: parse HTTP request line, headers, and body\n    todo!(\"parse HTTP request\")\n}\n\nfn main() {\n    let listener = TcpListener::bind(\"127.0.0.1:8080\").unwrap();\n    for stream in listener.incoming().flatten() {\n        // TODO: read full request, parse it, and respond\n    }\n}\n".to_string(),
                        is_executable: false,
                    },
                ],
                tests: vec![
                    ProjectFile {
                        path: "tests/test_basic.rs".to_string(),
                        content: "#[test]\nfn test_placeholder() {\n    assert!(true);\n}\n".to_string(),
                        is_executable: false,
                    },
                ],
                solution_files: vec![
                    ProjectFile {
                        path: "solution.rs".to_string(),
                        content: "// Solution for Stage 2: HTTP Parsing\n// Implement the TODO items in the skeleton code.\n".to_string(),
                        is_executable: false,
                    },
                ],
            },
            ProjectStage {
                name: "Stage 3: Routing".to_string(),
                description: "Route requests to different handlers based on method and path.".to_string(),
                files: vec![
                    ProjectFile {
                        path: "src/main.rs".to_string(),
                        content: "use std::collections::HashMap;\nuse std::io::{Read, Write};\nuse std::net::TcpListener;\n\nstruct HttpRequest {\n    method: String,\n    path: String,\n    headers: HashMap<String, String>,\n    body: String,\n}\n\nfn handle_request(req: &HttpRequest) -> (u16, String) {\n    // TODO: match on method + path and return (status_code, body)\n    todo!(\"route the request\")\n}\n\nfn main() {\n    let listener = TcpListener::bind(\"127.0.0.1:8080\").unwrap();\n    for stream in listener.incoming().flatten() {\n        // TODO: read, parse, route, and respond\n    }\n}\n".to_string(),
                        is_executable: false,
                    },
                ],
                tests: vec![
                    ProjectFile {
                        path: "tests/test_basic.rs".to_string(),
                        content: "#[test]\nfn test_placeholder() {\n    assert!(true);\n}\n".to_string(),
                        is_executable: false,
                    },
                ],
                solution_files: vec![
                    ProjectFile {
                        path: "solution.rs".to_string(),
                        content: "// Solution for Stage 3: Routing\n// Implement the TODO items in the skeleton code.\n".to_string(),
                        is_executable: false,
                    },
                ],
            },
            ProjectStage {
                name: "Stage 4: Static Files".to_string(),
                description: "Serve files from a local directory based on the request path.".to_string(),
                files: vec![
                    ProjectFile {
                        path: "src/main.rs".to_string(),
                        content: "use std::collections::HashMap;\nuse std::io::{Read, Write};\nuse std::net::TcpListener;\nuse std::path::PathBuf;\n\nstruct HttpRequest {\n    method: String,\n    path: String,\n    headers: HashMap<String, String>,\n    body: String,\n}\n\nfn serve_static(root: &PathBuf, path: &str) -> Option<(String, Vec<u8>)> {\n    // TODO: map request path to a file under root, read and return it\n    todo!(\"serve static file\")\n}\n\nfn main() {\n    let root = PathBuf::from(\"./public\");\n    let listener = TcpListener::bind(\"127.0.0.1:8080\").unwrap();\n    for stream in listener.incoming().flatten() {\n        // TODO: parse request, try static file, fall back to handler\n    }\n}\n".to_string(),
                        is_executable: false,
                    },
                ],
                tests: vec![
                    ProjectFile {
                        path: "tests/test_basic.rs".to_string(),
                        content: "#[test]\nfn test_placeholder() {\n    assert!(true);\n}\n".to_string(),
                        is_executable: false,
                    },
                ],
                solution_files: vec![
                    ProjectFile {
                        path: "solution.rs".to_string(),
                        content: "// Solution for Stage 4: Static Files\n// Implement the TODO items in the skeleton code.\n".to_string(),
                        is_executable: false,
                    },
                ],
            },
            ProjectStage {
                name: "Stage 5: Dynamic Response".to_string(),
                description: "Support query parameters and JSON response bodies.".to_string(),
                files: vec![
                    ProjectFile {
                        path: "src/main.rs".to_string(),
                        content: "use std::collections::HashMap;\nuse std::io::{Read, Write};\nuse std::net::TcpListener;\nuse std::path::PathBuf;\n\nstruct HttpRequest {\n    method: String,\n    path: String,\n    headers: HashMap<String, String>,\n    body: String,\n    query: HashMap<String, String>,\n}\n\nfn parse_query_string(query: &str) -> HashMap<String, String> {\n    // TODO: parse key=value pairs from query string\n    todo!(\"parse query string\")\n}\n\nfn main() {\n    let root = PathBuf::from(\"./public\");\n    let listener = TcpListener::bind(\"127.0.0.1:8080\").unwrap();\n    for stream in listener.incoming().flatten() {\n        // TODO: parse request with query params, serve static or JSON\n    }\n}\n".to_string(),
                        is_executable: false,
                    },
                ],
                tests: vec![
                    ProjectFile {
                        path: "tests/test_basic.rs".to_string(),
                        content: "#[test]\nfn test_placeholder() {\n    assert!(true);\n}\n".to_string(),
                        is_executable: false,
                    },
                ],
                solution_files: vec![
                    ProjectFile {
                        path: "solution.rs".to_string(),
                        content: "// Solution for Stage 5: Dynamic Response\n// Implement the TODO items in the skeleton code.\n".to_string(),
                        is_executable: false,
                    },
                ],
            },
        ],
    }
}

fn redis_project() -> ProjectPack {
    ProjectPack {
        id: "build-your-own-redis".to_string(),
        name: "Build Your Own Redis".to_string(),
        description: "Implement a Redis-like in-memory key-value store with RESP protocol support.".to_string(),
        difficulty: "hard".to_string(),
        language: "rust".to_string(),
        readme: "# Build Your Own Redis\n\nImplement a simplified Redis server in Rust.\n\n## Stages\n\n1. TCP Server: Accept connections and handle the RESP protocol basics\n2. String Commands: Implement GET, SET, DEL, PING\n3. Expiry: Support TTL and expiration of keys\n4. Lists: Implement LPUSH, RPUSH, LPOP, RPOP, LRANGE\n5. Persistence: Save and load database snapshots to disk\n\n## Getting Started\n\nEach stage builds on the previous one. Start with stage 1.".to_string(),
        stages: vec![
            ProjectStage {
                name: "Stage 1: TCP Server & RESP".to_string(),
                description: "Accept TCP connections and parse the RESP (REdis Serialization Protocol).".to_string(),
                files: vec![
                    ProjectFile {
                        path: "Cargo.toml".to_string(),
                        content: "[package]\nname = \"my-redis\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[[bin]]\nname = \"redis-server\"\npath = \"src/main.rs\"\n".to_string(),
                        is_executable: false,
                    },
                    ProjectFile {
                        path: "src/main.rs".to_string(),
                        content: "use std::io::{Read, Write};\nuse std::net::TcpListener;\n\nfn main() {\n    let listener = TcpListener::bind(\"127.0.0.1:6379\").unwrap();\n    println!(\"Redis server listening on 6379\");\n    for stream in listener.incoming().flatten() {\n        // TODO: read RESP-encoded command, parse it, and respond\n    }\n}\n\nfn parse_resp(input: &[u8]) -> Option<Vec<String>> {\n    // TODO: parse RESP array of bulk strings\n    todo!(\"parse RESP protocol\")\n}\n\nfn encode_resp(value: &str) -> String {\n    // TODO: encode a value as a RESP bulk string\n    todo!(\"encode RESP response\")\n}\n".to_string(),
                        is_executable: false,
                    },
                ],
                tests: vec![
                    ProjectFile {
                        path: "tests/test_basic.rs".to_string(),
                        content: "#[test]\nfn test_placeholder() {\n    assert!(true);\n}\n".to_string(),
                        is_executable: false,
                    },
                ],
                solution_files: vec![
                    ProjectFile {
                        path: "solution.rs".to_string(),
                        content: "// Solution for Stage 1: TCP Server & RESP\n// Implement the TODO items in the skeleton code.\n".to_string(),
                        is_executable: false,
                    },
                ],
            },
            ProjectStage {
                name: "Stage 2: String Commands".to_string(),
                description: "Implement GET, SET, DEL, and PING commands.".to_string(),
                files: vec![
                    ProjectFile {
                        path: "src/main.rs".to_string(),
                        content: "use std::collections::HashMap;\nuse std::sync::{Arc, Mutex};\nuse std::io::{Read, Write};\nuse std::net::TcpListener;\n\nstruct Store {\n    strings: HashMap<String, String>,\n}\n\nfn handle_command(store: &mut Store, args: &[String]) -> String {\n    if args.is_empty() {\n        return encode_resp_error(\"ERR empty command\");\n    }\n    match args[0].to_uppercase().as_str() {\n        \"PING\" => encode_resp(\"PONG\"),\n        \"SET\" => {\n            // TODO: SET key value\n            todo!(\"implement SET\")\n        }\n        \"GET\" => {\n            // TODO: GET key\n            todo!(\"implement GET\")\n        }\n        \"DEL\" => {\n            // TODO: DEL key [key ...]\n            todo!(\"implement DEL\")\n        }\n        _ => encode_resp_error(\"ERR unknown command\"),\n    }\n}\n\nfn encode_resp(value: &str) -> String {\n    format!(\"${}\\r\\n{}\\r\\n\", value.len(), value)\n}\n\nfn encode_resp_error(msg: &str) -> String {\n    format!(\"-{}\\r\\n\", msg)\n}\n\nfn main() {\n    let store = Arc::new(Mutex::new(Store { strings: HashMap::new() }));\n    let listener = TcpListener::bind(\"127.0.0.1:6379\").unwrap();\n    for stream in listener.incoming().flatten() {\n        // TODO: read commands, handle with shared store\n    }\n}\n".to_string(),
                        is_executable: false,
                    },
                ],
                tests: vec![
                    ProjectFile {
                        path: "tests/test_basic.rs".to_string(),
                        content: "#[test]\nfn test_placeholder() {\n    assert!(true);\n}\n".to_string(),
                        is_executable: false,
                    },
                ],
                solution_files: vec![
                    ProjectFile {
                        path: "solution.rs".to_string(),
                        content: "// Solution for Stage 2: String Commands\n// Implement the TODO items in the skeleton code.\n".to_string(),
                        is_executable: false,
                    },
                ],
            },
            ProjectStage {
                name: "Stage 3: Expiry".to_string(),
                description: "Support TTL and automatic expiration of keys.".to_string(),
                files: vec![
                    ProjectFile {
                        path: "src/main.rs".to_string(),
                        content: "use std::collections::HashMap;\nuse std::sync::{Arc, Mutex};\nuse std::time::{Duration, Instant};\n\nstruct ExpiryEntry {\n    value: String,\n    expires_at: Option<Instant>,\n}\n\nstruct Store {\n    entries: HashMap<String, ExpiryEntry>,\n}\n\nfn handle_command(store: &mut Store, args: &[String]) -> String {\n    if args.is_empty() {\n        return encode_resp_error(\"ERR empty command\");\n    }\n    match args[0].to_uppercase().as_str() {\n        \"SET\" => {\n            // TODO: SET key value [EX seconds] [PX milliseconds]\n            todo!(\"implement SET with expiry\")\n        }\n        \"GET\" => {\n            // TODO: GET key (check expiry first)\n            todo!(\"implement GET with expiry check\")\n        }\n        \"TTL\" => {\n            // TODO: return time-to-live for a key\n            todo!(\"implement TTL\")\n        }\n        _ => encode_resp_error(\"ERR unknown command\"),\n    }\n}\n\nfn encode_resp(value: &str) -> String {\n    format!(\"${}\\r\\n{}\\r\\n\", value.len(), value)\n}\n\nfn encode_resp_error(msg: &str) -> String {\n    format!(\"-{}\\r\\n\", msg)\n}\n\nfn main() {\n    let store = Arc::new(Mutex::new(Store { entries: HashMap::new() }));\n    let listener = std::net::TcpListener::bind(\"127.0.0.1:6379\").unwrap();\n    for stream in listener.incoming().flatten() {\n        // TODO: handle connections with expiry-aware store\n    }\n}\n".to_string(),
                        is_executable: false,
                    },
                ],
                tests: vec![
                    ProjectFile {
                        path: "tests/test_basic.rs".to_string(),
                        content: "#[test]\nfn test_placeholder() {\n    assert!(true);\n}\n".to_string(),
                        is_executable: false,
                    },
                ],
                solution_files: vec![
                    ProjectFile {
                        path: "solution.rs".to_string(),
                        content: "// Solution for Stage 3: Expiry\n// Implement the TODO items in the skeleton code.\n".to_string(),
                        is_executable: false,
                    },
                ],
            },
            ProjectStage {
                name: "Stage 4: Lists".to_string(),
                description: "Implement LPUSH, RPUSH, LPOP, RPOP, and LRANGE commands.".to_string(),
                files: vec![
                    ProjectFile {
                        path: "src/main.rs".to_string(),
                        content: "use std::collections::HashMap;\nuse std::sync::{Arc, Mutex};\nuse std::time::Instant;\n\nstruct ExpiryEntry {\n    value: String,\n    expires_at: Option<Instant>,\n}\n\nstruct Store {\n    entries: HashMap<String, ExpiryEntry>,\n    lists: HashMap<String, Vec<String>>,\n}\n\nfn handle_command(store: &mut Store, args: &[String]) -> String {\n    if args.is_empty() {\n        return encode_resp_error(\"ERR empty command\");\n    }\n    match args[0].to_uppercase().as_str() {\n        \"LPUSH\" => {\n            // TODO: LPUSH key element [element ...]\n            todo!(\"implement LPUSH\")\n        }\n        \"RPUSH\" => {\n            // TODO: RPUSH key element [element ...]\n            todo!(\"implement RPUSH\")\n        }\n        \"LPOP\" => {\n            // TODO: LPOP key [count]\n            todo!(\"implement LPOP\")\n        }\n        \"RPOP\" => {\n            // TODO: RPOP key [count]\n            todo!(\"implement RPOP\")\n        }\n        \"LRANGE\" => {\n            // TODO: LRANGE key start stop\n            todo!(\"implement LRANGE\")\n        }\n        _ => encode_resp_error(\"ERR unknown command\"),\n    }\n}\n\nfn encode_resp(value: &str) -> String {\n    format!(\"${}\\r\\n{}\\r\\n\", value.len(), value)\n}\n\nfn encode_resp_error(msg: &str) -> String {\n    format!(\"-{}\\r\\n\", msg)\n}\n\nfn main() {\n    let store = Arc::new(Mutex::new(Store {\n        entries: HashMap::new(),\n        lists: HashMap::new(),\n    }));\n    let listener = std::net::TcpListener::bind(\"127.0.0.1:6379\").unwrap();\n    for stream in listener.incoming().flatten() {\n        // TODO: handle connections\n    }\n}\n".to_string(),
                        is_executable: false,
                    },
                ],
                tests: vec![
                    ProjectFile {
                        path: "tests/test_basic.rs".to_string(),
                        content: "#[test]\nfn test_placeholder() {\n    assert!(true);\n}\n".to_string(),
                        is_executable: false,
                    },
                ],
                solution_files: vec![
                    ProjectFile {
                        path: "solution.rs".to_string(),
                        content: "// Solution for Stage 4: Lists\n// Implement the TODO items in the skeleton code.\n".to_string(),
                        is_executable: false,
                    },
                ],
            },
            ProjectStage {
                name: "Stage 5: Persistence".to_string(),
                description: "Save and load database snapshots to disk.".to_string(),
                files: vec![
                    ProjectFile {
                        path: "src/main.rs".to_string(),
                        content: "use std::collections::HashMap;\nuse std::sync::{Arc, Mutex};\nuse std::time::Instant;\n\nstruct ExpiryEntry {\n    value: String,\n    expires_at: Option<Instant>,\n}\n\nstruct Store {\n    entries: HashMap<String, ExpiryEntry>,\n    lists: HashMap<String, Vec<String>>,\n}\n\nfn save_snapshot(store: &Store, path: &str) -> std::io::Result<()> {\n    // TODO: serialize store to a file (e.g., RDB-like binary format or JSON)\n    todo!(\"save snapshot\")\n}\n\nfn load_snapshot(store: &mut Store, path: &str) -> std::io::Result<()> {\n    // TODO: deserialize store from a file\n    todo!(\"load snapshot\")\n}\n\nfn handle_command(store: &mut Store, args: &[String]) -> String {\n    if args.is_empty() {\n        return encode_resp_error(\"ERR empty command\");\n    }\n    match args[0].to_uppercase().as_str() {\n        \"SAVE\" => {\n            // TODO: persist to disk\n            todo!(\"implement SAVE\")\n        }\n        \"BGSAVE\" => {\n            // TODO: persist in background\n            todo!(\"implement BGSAVE\")\n        }\n        _ => handle_data_command(store, args),\n    }\n}\n\nfn handle_data_command(store: &mut Store, args: &[String]) -> String {\n    // TODO: all data commands from previous stages\n    todo!(\"handle data commands\")\n}\n\nfn encode_resp(value: &str) -> String {\n    format!(\"${}\\r\\n{}\\r\\n\", value.len(), value)\n}\n\nfn encode_resp_error(msg: &str) -> String {\n    format!(\"-{}\\r\\n\", msg)\n}\n\nfn main() {\n    let store = Arc::new(Mutex::new(Store {\n        entries: HashMap::new(),\n        lists: HashMap::new(),\n    }));\n    // TODO: try loading snapshot on startup\n    let listener = std::net::TcpListener::bind(\"127.0.0.1:6379\").unwrap();\n    for stream in listener.incoming().flatten() {\n        // TODO: handle connections\n    }\n}\n".to_string(),
                        is_executable: false,
                    },
                ],
                tests: vec![
                    ProjectFile {
                        path: "tests/test_basic.rs".to_string(),
                        content: "#[test]\nfn test_placeholder() {\n    assert!(true);\n}\n".to_string(),
                        is_executable: false,
                    },
                ],
                solution_files: vec![
                    ProjectFile {
                        path: "solution.rs".to_string(),
                        content: "// Solution for Stage 5: Persistence\n// Implement the TODO items in the skeleton code.\n".to_string(),
                        is_executable: false,
                    },
                ],
            },
        ],
    }
}

fn compiler_project() -> ProjectPack {
    ProjectPack {
        id: "build-your-own-compiler".to_string(),
        name: "Build Your Own Compiler".to_string(),
        description: "Build a compiler for a simple language: lexer, parser, AST, and code generator.".to_string(),
        difficulty: "hard".to_string(),
        language: "rust".to_string(),
        readme: "# Build Your Own Compiler\n\nBuild a compiler for a simple C-like language in Rust.\n\n## Stages\n\n1. Lexer: Tokenize source code into tokens\n2. Parser: Parse tokens into an Abstract Syntax Tree (AST)\n3. Interpreter: Walk the AST and execute programs directly\n4. Type Checker: Add static type checking\n5. Code Generator: Generate assembly or bytecode from the AST\n\n## Getting Started\n\nEach stage builds on the previous one. Start with stage 1.".to_string(),
        stages: vec![
            ProjectStage {
                name: "Stage 1: Lexer".to_string(),
                description: "Tokenize source code into a stream of tokens (keywords, identifiers, numbers, symbols).".to_string(),
                files: vec![
                    ProjectFile {
                        path: "Cargo.toml".to_string(),
                        content: "[package]\nname = \"my-compiler\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[[bin]]\nname = \"mycc\"\npath = \"src/main.rs\"\n".to_string(),
                        is_executable: false,
                    },
                    ProjectFile {
                        path: "src/main.rs".to_string(),
                        content: "mod lexer;\nuse lexer::Lexer;\nuse std::env;\nuse std::fs;\n\nfn main() {\n    let args: Vec<String> = env::args().collect();\n    if args.len() < 2 {\n        eprintln!(\"usage: mycc <file>\");\n        return;\n    }\n    let source = fs::read_to_string(&args[1]).expect(\"Failed to read file\");\n    let mut lexer = Lexer::new(&source);\n    let tokens = lexer.tokenize();\n    for token in &tokens {\n        println!(\"{:?}\", token);\n    }\n}\n".to_string(),
                        is_executable: false,
                    },
                    ProjectFile {
                        path: "src/lexer.rs".to_string(),
                        content: "#[derive(Debug, Clone, PartialEq)]\npub enum Token {\n    // TODO: define token variants\n    // e.g., Integer(i64), Float(f64), String(String),\n    // Identifier(String), Plus, Minus, Star, Slash,\n    // LeftParen, RightParen, LeftBrace, RightBrace,\n    // Semicolon, Equals, If, Else, While, Return, Func, Let, Int, Float, Bool, Print,\n    EOF,\n}\n\npub struct Lexer {\n    input: Vec<char>,\n    pos: usize,\n}\n\nimpl Lexer {\n    pub fn new(input: &str) -> Self {\n        Lexer {\n            input: input.chars().collect(),\n            pos: 0,\n        }\n    }\n\n    pub fn tokenize(&mut self) -> Vec<Token> {\n        let mut tokens = Vec::new();\n        // TODO: scan characters and produce tokens\n        tokens.push(Token::EOF);\n        tokens\n    }\n}\n".to_string(),
                        is_executable: false,
                    },
                ],
                tests: vec![
                    ProjectFile {
                        path: "tests/test_basic.rs".to_string(),
                        content: "#[test]\nfn test_placeholder() {\n    assert!(true);\n}\n".to_string(),
                        is_executable: false,
                    },
                ],
                solution_files: vec![
                    ProjectFile {
                        path: "solution.rs".to_string(),
                        content: "// Solution for Stage 1: Lexer\n// Implement the TODO items in the skeleton code.\n".to_string(),
                        is_executable: false,
                    },
                ],
            },
            ProjectStage {
                name: "Stage 2: Parser".to_string(),
                description: "Parse tokens into an Abstract Syntax Tree (AST) using recursive descent.".to_string(),
                files: vec![
                    ProjectFile {
                        path: "src/main.rs".to_string(),
                        content: "mod lexer;\nmod parser;\nuse lexer::Lexer;\nuse parser::Parser;\nuse std::env;\nuse std::fs;\n\nfn main() {\n    let args: Vec<String> = env::args().collect();\n    if args.len() < 2 {\n        eprintln!(\"usage: mycc <file>\");\n        return;\n    }\n    let source = fs::read_to_string(&args[1]).expect(\"Failed to read file\");\n    let mut lexer = Lexer::new(&source);\n    let tokens = lexer.tokenize();\n    let mut parser = Parser::new(tokens);\n    let ast = parser.parse_program();\n    println!(\"{:#?}\", ast);\n}\n".to_string(),
                        is_executable: false,
                    },
                    ProjectFile {
                        path: "src/parser.rs".to_string(),
                        content: "use crate::lexer::Token;\n\n#[derive(Debug, Clone)]\npub enum Expr {\n    // TODO: define expression variants\n    // Integer(i64), Float(f64), String(String), Bool(bool),\n    // Ident(String), BinaryOp { op: String, left: Box<Expr>, right: Box<Expr> },\n    // UnaryOp { op: String, expr: Box<Expr> },\n    // Call { name: String, args: Vec<Expr> },\n    Placeholder,\n}\n\n#[derive(Debug, Clone)]\npub enum Stmt {\n    // TODO: define statement variants\n    // ExprStmt(Expr), Return(Expr),\n    // Let { name: String, ty: String, value: Expr },\n    // If { condition: Expr, then_body: Vec<Stmt>, else_body: Option<Vec<Stmt>> },\n    // While { condition: Expr, body: Vec<Stmt> },\n    // Func { name: String, params: Vec<(String, String)>, return_type: String, body: Vec<Stmt> },\n    // Print(Expr),\n    Placeholder,\n}\n\npub struct Parser {\n    tokens: Vec<Token>,\n    pos: usize,\n}\n\nimpl Parser {\n    pub fn new(tokens: Vec<Token>) -> Self {\n        Parser { tokens, pos: 0 }\n    }\n\n    pub fn parse_program(&mut self) -> Vec<Stmt> {\n        // TODO: parse a sequence of statements until EOF\n        todo!(\"parse program\")\n    }\n}\n".to_string(),
                        is_executable: false,
                    },
                ],
                tests: vec![
                    ProjectFile {
                        path: "tests/test_basic.rs".to_string(),
                        content: "#[test]\nfn test_placeholder() {\n    assert!(true);\n}\n".to_string(),
                        is_executable: false,
                    },
                ],
                solution_files: vec![
                    ProjectFile {
                        path: "solution.rs".to_string(),
                        content: "// Solution for Stage 2: Parser\n// Implement the TODO items in the skeleton code.\n".to_string(),
                        is_executable: false,
                    },
                ],
            },
            ProjectStage {
                name: "Stage 3: Interpreter".to_string(),
                description: "Walk the AST and execute programs directly (tree-walk interpreter).".to_string(),
                files: vec![
                    ProjectFile {
                        path: "src/main.rs".to_string(),
                        content: "mod lexer;\nmod parser;\nmod interpreter;\nuse lexer::Lexer;\nuse parser::Parser;\nuse interpreter::Interpreter;\nuse std::env;\nuse std::fs;\n\nfn main() {\n    let args: Vec<String> = env::args().collect();\n    if args.len() < 2 {\n        eprintln!(\"usage: mycc <file>\");\n        return;\n    }\n    let source = fs::read_to_string(&args[1]).expect(\"Failed to read file\");\n    let mut lexer = Lexer::new(&source);\n    let tokens = lexer.tokenize();\n    let mut parser = Parser::new(tokens);\n    let ast = parser.parse_program();\n    let mut interp = Interpreter::new();\n    if let Err(e) = interp.run(&ast) {\n        eprintln!(\"Runtime error: {}\", e);\n    }\n}\n".to_string(),
                        is_executable: false,
                    },
                    ProjectFile {
                        path: "src/interpreter.rs".to_string(),
                        content: "use crate::parser::{Expr, Stmt};\nuse std::collections::HashMap;\n\npub struct Interpreter {\n    variables: HashMap<String, i64>,\n}\n\nimpl Interpreter {\n    pub fn new() -> Self {\n        Interpreter {\n            variables: HashMap::new(),\n        }\n    }\n\n    pub fn run(&mut self, program: &[Stmt]) -> Result<(), String> {\n        for stmt in program {\n            self.exec_stmt(stmt)?;\n        }\n        Ok(())\n    }\n\n    fn exec_stmt(&mut self, stmt: &Stmt) -> Result<(), String> {\n        // TODO: implement statement execution\n        todo!(\"execute statement\")\n    }\n\n    fn eval_expr(&mut self, expr: &Expr) -> Result<i64, String> {\n        // TODO: implement expression evaluation\n        todo!(\"evaluate expression\")\n    }\n}\n".to_string(),
                        is_executable: false,
                    },
                ],
                tests: vec![
                    ProjectFile {
                        path: "tests/test_basic.rs".to_string(),
                        content: "#[test]\nfn test_placeholder() {\n    assert!(true);\n}\n".to_string(),
                        is_executable: false,
                    },
                ],
                solution_files: vec![
                    ProjectFile {
                        path: "solution.rs".to_string(),
                        content: "// Solution for Stage 3: Interpreter\n// Implement the TODO items in the skeleton code.\n".to_string(),
                        is_executable: false,
                    },
                ],
            },
            ProjectStage {
                name: "Stage 4: Type Checker".to_string(),
                description: "Add static type checking before interpretation.".to_string(),
                files: vec![
                    ProjectFile {
                        path: "src/typechecker.rs".to_string(),
                        content: "use crate::parser::{Expr, Stmt};\nuse std::collections::HashMap;\n\n#[derive(Debug, Clone, PartialEq)]\npub enum Type {\n    Int,\n    Float,\n    Bool,\n    String,\n    Void,\n    Func(Vec<Type>, Box<Type>),\n}\n\npub struct TypeChecker {\n    env: HashMap<String, Type>,\n}\n\nimpl TypeChecker {\n    pub fn new() -> Self {\n        TypeChecker {\n            env: HashMap::new(),\n        }\n    }\n\n    pub fn check_program(&mut self, program: &[Stmt]) -> Result<(), String> {\n        for stmt in program {\n            self.check_stmt(stmt)?;\n        }\n        Ok(())\n    }\n\n    fn check_stmt(&mut self, stmt: &Stmt) -> Result<(), String> {\n        // TODO: type-check a statement\n        todo!(\"type-check statement\")\n    }\n\n    fn check_expr(&mut self, expr: &Expr) -> Result<Type, String> {\n        // TODO: type-check an expression\n        todo!(\"type-check expression\")\n    }\n}\n".to_string(),
                        is_executable: false,
                    },
                ],
                tests: vec![
                    ProjectFile {
                        path: "tests/test_basic.rs".to_string(),
                        content: "#[test]\nfn test_placeholder() {\n    assert!(true);\n}\n".to_string(),
                        is_executable: false,
                    },
                ],
                solution_files: vec![
                    ProjectFile {
                        path: "solution.rs".to_string(),
                        content: "// Solution for Stage 4: Type Checker\n// Implement the TODO items in the skeleton code.\n".to_string(),
                        is_executable: false,
                    },
                ],
            },
            ProjectStage {
                name: "Stage 5: Code Generator".to_string(),
                description: "Generate simple bytecode or stack-based virtual machine instructions from the AST.".to_string(),
                files: vec![
                    ProjectFile {
                        path: "src/codegen.rs".to_string(),
                        content: "use crate::parser::{Expr, Stmt};\n\n#[derive(Debug, Clone)]\npub enum Instruction {\n    Push(i64),\n    Pop,\n    Add,\n    Sub,\n    Mul,\n    Div,\n    LoadVar(String),\n    StoreVar(String),\n    Print,\n    Halt,\n}\n\npub struct CodeGenerator {\n    instructions: Vec<Instruction>,\n}\n\nimpl CodeGenerator {\n    pub fn new() -> Self {\n        CodeGenerator {\n            instructions: Vec::new(),\n        }\n    }\n\n    pub fn generate(&mut self, program: &[Stmt]) -> Vec<Instruction> {\n        for stmt in program {\n            self.gen_stmt(stmt);\n        }\n        self.instructions.push(Instruction::Halt);\n        self.instructions.clone()\n    }\n\n    fn gen_stmt(&mut self, stmt: &Stmt) {\n        // TODO: generate instructions for a statement\n        todo!(\"generate code for statement\")\n    }\n\n    fn gen_expr(&mut self, expr: &Expr) {\n        // TODO: generate instructions for an expression\n        todo!(\"generate code for expression\")\n    }\n}\n\npub struct VM {\n    stack: Vec<i64>,\n    vars: std::collections::HashMap<String, i64>,\n}\n\nimpl VM {\n    pub fn new() -> Self {\n        VM {\n            stack: Vec::new(),\n            vars: std::collections::HashMap::new(),\n        }\n    }\n\n    pub fn run(&mut self, instructions: &[Instruction]) -> Result<(), String> {\n        for instr in instructions {\n            self.exec(instr)?;\n        }\n        Ok(())\n    }\n\n    fn exec(&mut self, instr: &Instruction) -> Result<(), String> {\n        // TODO: execute a single instruction\n        todo!(\"execute instruction\")\n    }\n}\n".to_string(),
                        is_executable: false,
                    },
                ],
                tests: vec![
                    ProjectFile {
                        path: "tests/test_basic.rs".to_string(),
                        content: "#[test]\nfn test_placeholder() {\n    assert!(true);\n}\n".to_string(),
                        is_executable: false,
                    },
                ],
                solution_files: vec![
                    ProjectFile {
                        path: "solution.rs".to_string(),
                        content: "// Solution for Stage 5: Code Generator\n// Implement the TODO items in the skeleton code.\n".to_string(),
                        is_executable: false,
                    },
                ],
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
                    if let Ok(project_content) = fs::read_to_string(&project_file) {
                        if let Ok(project) = serde_json::from_str::<ProjectPack>(&project_content) {
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
        self.builtin
            .iter()
            .chain(self.installed.iter())
            .find(|p| p.id == project_id)
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
