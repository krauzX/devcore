use crate::models::*;
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use git2::{Delta, DiffOptions, Repository, Sort};
use std::path::Path;

pub struct GitAnalyzer {
    repo: Repository,
}

impl GitAnalyzer {
    pub fn open(path: &Path) -> Result<Self> {
        let repo = Repository::open(path)
            .with_context(|| format!("Failed to open git repo at {}", path.display()))?;
        Ok(Self { repo })
    }

    pub fn head_oid(&self) -> Result<String> {
        let head = self.repo.head().context("No HEAD")?;
        Ok(head.target().context("HEAD is not a direct reference")?.to_string())
    }

    pub fn commits_since(
        &self,
        since: DateTime<Utc>,
        limit: usize,
    ) -> Result<Vec<CommitInfo>> {
        let mut revwalk = self.repo.revwalk()?;
        revwalk.set_sorting(Sort::TIME)?;
        revwalk.push_head()?;

        let since_ts = git2::Time::new(since.timestamp(), 0);
        let mut commits = Vec::new();

        for oid_result in revwalk {
            let oid = oid_result?;
            let commit = self.repo.find_commit(oid)?;

            if commit.time().seconds() < since_ts.seconds() {
                break;
            }

            let info = self.commit_info(&commit)?;
            commits.push(info);

            if commits.len() >= limit {
                break;
            }
        }

        Ok(commits)
    }

    pub fn commit_info(&self, commit: &git2::Commit) -> Result<CommitInfo> {
        let oid = commit.id().to_string();
        let message = commit.message().unwrap_or("").to_string();
        let author = commit.author().name().unwrap_or("unknown").to_string();
        let timestamp = DateTime::from_timestamp(commit.time().seconds(), 0)
            .unwrap_or_default();

        let ai_source = AiSource::from_commit_message(&message);
        let is_ai = ai_source.is_some();

        let files_changed = self.commit_files(commit)?;
        let diff_stats = self.commit_diff_stats(commit)?;

        Ok(CommitInfo {
            oid,
            message,
            author,
            timestamp,
            is_ai_generated: is_ai,
            ai_source,
            files_changed,
            insertions: diff_stats.0,
            deletions: diff_stats.1,
        })
    }

    fn commit_files(&self, commit: &git2::Commit) -> Result<Vec<FileChange>> {
        let tree = commit.tree()?;
        let parent_tree = commit.parent(0).ok().map(|p| p.tree()).transpose()?;

        let mut diff_opts = DiffOptions::new();
        let diff = self.repo.diff_tree_to_tree(
            parent_tree.as_ref(),
            Some(&tree),
            Some(&mut diff_opts),
        )?;

        let mut files = Vec::new();
        for delta_idx in 0..diff.deltas().len() {
            let delta = diff.deltas().nth(delta_idx).unwrap();
            let path = delta
                .new_file()
                .path()
                .or_else(|| delta.old_file().path())
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_default();

            let status = match delta.status() {
                Delta::Added => ChangeStatus::Added,
                Delta::Modified => ChangeStatus::Modified,
                Delta::Deleted => ChangeStatus::Deleted,
                Delta::Renamed => ChangeStatus::Renamed,
                _ => ChangeStatus::Modified,
            };

            files.push(FileChange {
                path,
                status,
                insertions: 0,
                deletions: 0,
            });
        }

        Ok(files)
    }

    fn commit_diff_stats(&self, commit: &git2::Commit) -> Result<(u32, u32)> {
        let tree = commit.tree()?;
        let parent_tree = commit.parent(0).ok().map(|p| p.tree()).transpose()?;

        let mut diff_opts = DiffOptions::new();
        let diff = self.repo.diff_tree_to_tree(
            parent_tree.as_ref(),
            Some(&tree),
            Some(&mut diff_opts),
        )?;

        let stats = diff.stats()?;
        Ok((stats.insertions() as u32, stats.deletions() as u32))
    }

    pub fn file_content(&self, path: &str) -> Result<Option<String>> {
        let head = self.repo.head()?;
        let tree = head.peel_to_tree()?;
        let entry = tree.get_path(std::path::Path::new(path))?;
        let obj = entry.to_object(&self.repo)?;
        let blob = obj.into_blob().map_err(|e| anyhow::anyhow!("Failed to convert to blob: {:?}", e))?;
        Ok(Some(String::from_utf8_lossy(blob.content()).to_string()))
    }

    pub fn list_files(&self) -> Result<Vec<String>> {
        let head = self.repo.head()?;
        let tree = head.peel_to_tree()?;
        let mut files = Vec::new();

        tree.walk(git2::TreeWalkMode::PreOrder, |_root, entry| {
            if let Some(name) = entry.name() {
                if entry.kind() == Some(git2::ObjectType::Blob) {
                    files.push(name.to_string());
                }
            }
            git2::TreeWalkResult::Ok
        })?;

        Ok(files)
    }

    pub fn blame_file(&self, path: &str) -> Result<Vec<BlameLine>> {
        let _head = self.repo.head()?;
        let blame = self.repo.blame_file(
            std::path::Path::new(path),
            Some(&mut git2::BlameOptions::new()),
        )?;

        let mut lines = Vec::new();
        for hunk in blame.iter() {
            lines.push(BlameLine {
                line: hunk.final_start_line() as u32,
                commit_oid: hunk.final_commit_id().to_string(),
                author: hunk.final_signature().name().unwrap_or("unknown").to_string(),
            });
        }

        Ok(lines)
    }
}

#[derive(Debug, Clone)]
pub struct CommitInfo {
    pub oid: String,
    pub message: String,
    pub author: String,
    pub timestamp: DateTime<Utc>,
    pub is_ai_generated: bool,
    pub ai_source: Option<AiSource>,
    pub files_changed: Vec<FileChange>,
    pub insertions: u32,
    pub deletions: u32,
}

#[derive(Debug, Clone)]
pub struct BlameLine {
    pub line: u32,
    pub commit_oid: String,
    pub author: String,
}
