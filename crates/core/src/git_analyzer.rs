use crate::error::DevCoreError;
use crate::models::*;
use chrono::{DateTime, Utc};
use git2::{Delta, DiffOptions, Repository, Sort};
use std::path::Path;

/// Analyzes a git repository to extract commit history, file changes, and blame information.
pub struct GitAnalyzer {
    repo: Repository,
}

impl GitAnalyzer {
    /// Opens a git repository at the given path.
    /// Returns an error if the path is not a valid git repository.
    pub fn open(path: &Path) -> Result<Self, DevCoreError> {
        let repo = Repository::open(path).map_err(|e| {
            DevCoreError::Config(format!(
                "Failed to open git repo at {}: {}",
                path.display(),
                e
            ))
        })?;
        Ok(Self { repo })
    }

    /// Returns the OID of the current HEAD commit as a hex string.
    pub fn head_oid(&self) -> Result<String, DevCoreError> {
        let head = self
            .repo
            .head()
            .map_err(|_| DevCoreError::NotFound("No HEAD reference found".into()))?;
        Ok(head
            .target()
            .ok_or_else(|| DevCoreError::NotFound("HEAD is not a direct reference".into()))?
            .to_string())
    }

    /// Returns commit information for all commits since the given timestamp, up to `limit`.
    pub fn commits_since(
        &self,
        since: DateTime<Utc>,
        limit: usize,
    ) -> Result<Vec<CommitInfo>, DevCoreError> {
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

    /// Extracts structured information from a single git commit.
    pub fn commit_info(&self, commit: &git2::Commit) -> Result<CommitInfo, DevCoreError> {
        let oid = commit.id().to_string();
        let message = commit.message().unwrap_or("").to_string();
        let author = commit.author().name().unwrap_or("unknown").to_string();
        let timestamp = DateTime::from_timestamp(commit.time().seconds(), 0).unwrap_or_default();

        let ai_source = AiSource::from_commit_message(&message);
        let is_ai = ai_source.is_some();

        let tree = commit.tree()?;
        let parent_tree = commit.parent(0).ok().map(|p| p.tree()).transpose()?;
        let mut diff_opts = DiffOptions::new();
        let diff =
            self.repo
                .diff_tree_to_tree(parent_tree.as_ref(), Some(&tree), Some(&mut diff_opts))?;

        let mut files_changed = Vec::new();
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

            files_changed.push(FileChange {
                path,
                status,
                insertions: 0,
                deletions: 0,
            });
        }

        let stats = diff.stats()?;
        let insertions = stats.insertions() as u32;
        let deletions = stats.deletions() as u32;

        Ok(CommitInfo {
            oid,
            message,
            author,
            timestamp,
            is_ai_generated: is_ai,
            ai_source,
            files_changed,
            insertions,
            deletions,
        })
    }

    /// Returns the text content of a file at HEAD, or `None` if the file does not exist.
    pub fn file_content(&self, path: &str) -> Result<Option<String>, DevCoreError> {
        let head = self.repo.head()?;
        let tree = head.peel_to_tree()?;
        let entry = match tree.get_path(std::path::Path::new(path)) {
            Ok(e) => e,
            Err(_) => return Ok(None),
        };
        let obj = entry.to_object(&self.repo)?;
        let blob = obj.into_blob().map_err(|e| {
            DevCoreError::Git(git2::Error::from_str(&format!(
                "Failed to convert to blob: {:?}",
                e
            )))
        })?;
        Ok(Some(String::from_utf8_lossy(blob.content()).to_string()))
    }

    /// Lists all file paths tracked at HEAD.
    pub fn list_files(&self) -> Result<Vec<String>, DevCoreError> {
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

    /// Returns blame information for a file, listing each line's last-modifying commit and author.
    pub fn blame_file(&self, path: &str) -> Result<Vec<BlameLine>, DevCoreError> {
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
                author: hunk
                    .final_signature()
                    .name()
                    .unwrap_or("unknown")
                    .to_string(),
            });
        }

        Ok(lines)
    }
}

/// Summary information about a single git commit.
#[derive(Debug, Clone)]
pub struct CommitInfo {
    /// Git commit OID (hex string)
    pub oid: String,
    /// Full commit message
    pub message: String,
    /// Commit author name
    pub author: String,
    /// Commit timestamp
    pub timestamp: DateTime<Utc>,
    /// Whether the commit appears to be AI-generated
    pub is_ai_generated: bool,
    /// Detected AI source, if any
    pub ai_source: Option<AiSource>,
    /// Files modified in this commit
    pub files_changed: Vec<FileChange>,
    /// Total lines inserted
    pub insertions: u32,
    /// Total lines deleted
    pub deletions: u32,
}

/// A single line from `git blame`, recording which commit last touched it.
#[derive(Debug, Clone)]
pub struct BlameLine {
    /// 1-based line number in the file
    pub line: u32,
    /// OID of the commit that last modified this line
    pub commit_oid: String,
    /// Author of the last-modifying commit
    pub author: String,
}
