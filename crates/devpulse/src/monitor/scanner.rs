use anyhow::Result;
use chrono::{DateTime, Utc};
use devcore_core::*;
use std::path::PathBuf;
use walkdir::WalkDir;

/// Scans the system for git repositories and collects activity data.
pub struct SystemScanner {
    repos: Vec<PathBuf>,
    scan_dirs: Vec<PathBuf>,
}

impl SystemScanner {
    /// Create scanner with default search directories.
    /// Searches: home dir subdirectories, common dev locations.
    pub fn new() -> Self {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        let scan_dirs = vec![
            home.join("projects"),
            home.join("code"),
            home.join("dev"),
            home.join("src"),
            home.join("repos"),
            home.join("work"),
            home.join("oss"),
            home.join("github"),
            home.join("gitlab"),
        ];

        Self {
            repos: Vec::new(),
            scan_dirs,
        }
    }

    pub fn with_dirs(dirs: Vec<PathBuf>) -> Self {
        Self {
            repos: Vec::new(),
            scan_dirs: dirs,
        }
    }

    pub fn scan(&mut self) -> &[PathBuf] {
        self.repos.clear();

        for dir in &self.scan_dirs {
            if !dir.exists() {
                continue;
            }

            for entry in WalkDir::new(dir)
                .max_depth(4)
                .into_iter()
                .filter_map(|e| e.ok())
            {
                if entry.file_type().is_dir() && entry.file_name() == ".git" {
                    if let Some(parent) = entry.path().parent() {
                        let repo_path = parent.to_path_buf();
                        if !self.repos.contains(&repo_path) {
                            self.repos.push(repo_path);
                        }
                    }
                }
            }
        }

        &self.repos
    }

    pub fn collect_activity(&self, since: DateTime<Utc>) -> Result<Vec<RepoActivity>> {
        let mut activities = Vec::new();

        for repo_path in &self.repos {
            if let Ok(git) = GitAnalyzer::open(repo_path) {
                if let Ok(commits) = git.commits_since(since, 500) {
                    let total_insertions: u32 = commits.iter().map(|c| c.insertions).sum();
                    let total_deletions: u32 = commits.iter().map(|c| c.deletions).sum();
                    let ai_commits = commits.iter().filter(|c| c.is_ai_generated).count();
                    let human_commits = commits.len() - ai_commits;

                    let unique_files: std::collections::HashSet<String> = commits
                        .iter()
                        .flat_map(|c| c.files_changed.iter().map(|f| f.path.clone()))
                        .collect();

                    let unique_authors: std::collections::HashSet<String> =
                        commits.iter().map(|c| c.author.clone()).collect();

                    let repo_name = repo_path
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_else(|| "unknown".to_string());

                    activities.push(RepoActivity {
                        path: repo_path.clone(),
                        name: repo_name,
                        total_commits: commits.len(),
                        ai_commits,
                        human_commits,
                        total_insertions,
                        total_deletions,
                        unique_files: unique_files.len(),
                        unique_authors: unique_authors.len(),
                        first_commit: commits.last().map(|c| c.timestamp),
                        last_commit: commits.first().map(|c| c.timestamp),
                    });
                }
            }
        }

        activities.sort_by(|a, b| b.total_commits.cmp(&a.total_commits));
        Ok(activities)
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct RepoActivity {
    pub path: PathBuf,
    pub name: String,
    pub total_commits: usize,
    pub ai_commits: usize,
    pub human_commits: usize,
    pub total_insertions: u32,
    pub total_deletions: u32,
    pub unique_files: usize,
    pub unique_authors: usize,
    pub first_commit: Option<DateTime<Utc>>,
    pub last_commit: Option<DateTime<Utc>>,
}
