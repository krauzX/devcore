use std::collections::{HashMap, HashSet};
use std::path::Path;

use anyhow::Result;
use chrono::DateTime;
use git2::Repository;
use walkdir::WalkDir;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RepoAnalysis {
    pub total_commits: usize,
    pub total_insertions: usize,
    pub total_deletions: usize,
    pub unique_files: usize,
    pub unique_authors: usize,
    pub first_commit: Option<DateTime<chrono::Utc>>,
    pub last_commit: Option<DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LanguageStat {
    pub name: String,
    pub files: usize,
    pub lines: usize,
}

fn ext_to_language(ext: &str) -> Option<&'static str> {
    match ext {
        "rs" => Some("Rust"),
        "py" => Some("Python"),
        "js" => Some("JavaScript"),
        "ts" => Some("TypeScript"),
        "jsx" => Some("JavaScript (JSX)"),
        "tsx" => Some("TypeScript (TSX)"),
        "go" => Some("Go"),
        "java" => Some("Java"),
        "kt" => Some("Kotlin"),
        "c" => Some("C"),
        "cpp" | "cc" | "cxx" | "h" | "hpp" => Some("C++"),
        "cs" => Some("C#"),
        "rb" => Some("Ruby"),
        "php" => Some("PHP"),
        "swift" => Some("Swift"),
        "scala" => Some("Scala"),
        "sh" | "bash" | "zsh" => Some("Shell"),
        "html" | "htm" => Some("HTML"),
        "css" => Some("CSS"),
        "scss" | "sass" => Some("SASS"),
        "json" => Some("JSON"),
        "yaml" | "yml" => Some("YAML"),
        "toml" => Some("TOML"),
        "xml" => Some("XML"),
        "sql" => Some("SQL"),
        "md" | "markdown" => Some("Markdown"),
        "txt" => Some("Text"),
        "lua" => Some("Lua"),
        "r" | "R" => Some("R"),
        "dart" => Some("Dart"),
        "ex" | "exs" => Some("Elixir"),
        "erl" | "hrl" => Some("Erlang"),
        "hs" => Some("Haskell"),
        "ml" | "mli" => Some("OCaml"),
        "nim" => Some("Nim"),
        "zig" => Some("Zig"),
        "v" => Some("V"),
        "sol" => Some("Solidity"),
        _ => None,
    }
}

pub fn analyze_repo(path: &Path) -> Result<RepoAnalysis> {
    let repo = Repository::open(path)?;
    let mut revwalk = repo.revwalk()?;
    revwalk.set_sorting(git2::Sort::TIME)?;
    revwalk.push_head()?;

    let mut total_commits = 0usize;
    let mut total_insertions = 0usize;
    let mut total_deletions = 0usize;
    let mut files_touched: HashSet<String> = HashSet::new();
    let mut authors: HashSet<String> = HashSet::new();
    let mut timestamps: Vec<i64> = Vec::new();

    for oid_result in revwalk {
        let oid = oid_result?;
        let commit = repo.find_commit(oid)?;

        total_commits += 1;

        let author_name = commit.author().name().unwrap_or("unknown").to_string();
        authors.insert(author_name);

        let ts = commit.time().seconds();
        timestamps.push(ts);

        if commit.parent_count() > 0 {
            let parent = commit.parent(0)?;
            let diff = repo.diff_tree_to_tree(
                Some(&parent.tree()?),
                Some(&commit.tree()?),
                None,
            )?;
            let stats = diff.stats()?;
            total_insertions += stats.insertions();
            total_deletions += stats.deletions();

            for delta in diff.deltas() {
                if let Some(new_file) = delta.new_file().path() {
                    files_touched.insert(new_file.to_string_lossy().into_owned());
                }
            }
        } else {
            let tree = commit.tree()?;
            collect_files_from_tree(&repo, &tree, &mut files_touched);
        }
    }

    let first_commit = timestamps.iter().min().and_then(|&ts| {
        DateTime::from_timestamp(ts, 0)
    });
    let last_commit = timestamps.iter().max().and_then(|&ts| {
        DateTime::from_timestamp(ts, 0)
    });

    Ok(RepoAnalysis {
        total_commits,
        total_insertions,
        total_deletions,
        unique_files: files_touched.len(),
        unique_authors: authors.len(),
        first_commit,
        last_commit,
    })
}

fn collect_files_from_tree(repo: &Repository, tree: &git2::Tree, files: &mut HashSet<String>) {
    for entry in tree.iter() {
        if let Ok(name) = entry.name() {
            match entry.kind() {
                Some(git2::ObjectType::Tree) => {
                    if let Ok(subtree) = repo.find_tree(entry.id()) {
                        collect_files_from_tree(repo, &subtree, files);
                    }
                }
                Some(git2::ObjectType::Blob) => {
                    files.insert(name.to_string());
                }
                _ => {}
            }
        }
    }
}

pub fn detect_languages(path: &Path) -> Vec<LanguageStat> {
    let mut stats: HashMap<String, (usize, usize)> = HashMap::new();

    for entry in WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        let file_path = entry.path();
        let ext = file_path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");

        if let Some(lang) = ext_to_language(ext) {
            let entry = stats
                .entry(lang.to_string())
                .or_insert((0, 0));
            entry.0 += 1;
            if let Ok(content) = std::fs::read_to_string(file_path) {
                entry.1 += content.lines().count();
            }
        }
    }

    let mut result: Vec<LanguageStat> = stats
        .into_iter()
        .map(|(name, (files, lines))| LanguageStat { name, files, lines })
        .collect();

    result.sort_by(|a, b| b.lines.cmp(&a.lines));
    result
}
