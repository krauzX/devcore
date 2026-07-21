use anyhow::Result;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

/// A research paper tracked in the reading list.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Paper {
    /// Unique paper identifier
    pub id: String,
    /// Paper title
    pub title: String,
    /// Comma-separated list of authors
    pub authors: Option<String>,
    /// Publication venue or conference
    pub venue: Option<String>,
    /// Publication year
    pub year: Option<u32>,
    /// Digital Object Identifier
    pub doi: Option<String>,
    /// arXiv identifier
    pub arxiv_id: Option<String>,
    /// Current reading status
    pub status: PaperStatus,
    /// Tags for categorization
    pub tags: Option<String>,
    /// Personal notes
    pub notes: Option<String>,
    /// ISO 8601 timestamp when the paper was added
    pub added_at: String,
}

/// Reading status of a research paper.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PaperStatus {
    /// Not yet read
    ToRead,
    /// Currently being read
    Reading,
    /// Read and reviewed
    Read,
    /// Cited in work
    Cited,
    /// Archived for reference
    Archived,
}

impl std::fmt::Display for PaperStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ToRead => write!(f, "To Read"),
            Self::Reading => write!(f, "Reading"),
            Self::Read => write!(f, "Read"),
            Self::Cited => write!(f, "Cited"),
            Self::Archived => write!(f, "Archived"),
        }
    }
}

impl Paper {
    /// Lists papers, optionally filtered by status.
    pub fn list(conn: &Connection, status_filter: Option<&str>) -> Result<Vec<Paper>> {
        let sql = if status_filter.is_some() {
            "SELECT id, title, authors, venue, year, doi, arxiv_id, status, tags, notes, added_at
             FROM papers WHERE status = ?1 ORDER BY added_at DESC"
        } else {
            "SELECT id, title, authors, venue, year, doi, arxiv_id, status, tags, notes, added_at
             FROM papers ORDER BY added_at DESC"
        };

        let mut stmt = conn.prepare(sql)?;

        let papers = if let Some(s) = status_filter {
            let rows = stmt.query_map(params![s], Self::row_to_paper)?;
            rows.filter_map(|r| r.ok()).collect()
        } else {
            let rows = stmt.query_map([], Self::row_to_paper)?;
            rows.filter_map(|r| r.ok()).collect()
        };

        Ok(papers)
    }

    fn row_to_paper(row: &rusqlite::Row) -> rusqlite::Result<Paper> {
        let status_str: String = row.get(7)?;
        let status = match status_str.as_str() {
            "reading" => PaperStatus::Reading,
            "read" => PaperStatus::Read,
            "cited" => PaperStatus::Cited,
            "archived" => PaperStatus::Archived,
            _ => PaperStatus::ToRead,
        };

        Ok(Paper {
            id: row.get(0)?,
            title: row.get(1)?,
            authors: row.get(2)?,
            venue: row.get(3)?,
            year: row.get(4)?,
            doi: row.get(5)?,
            arxiv_id: row.get(6)?,
            status,
            tags: row.get(8)?,
            notes: row.get(9)?,
            added_at: row.get(10)?,
        })
    }

    /// Returns aggregate counts of papers by reading status.
    pub fn stats(conn: &Connection) -> Result<PaperStats> {
        let total: u32 = conn.query_row("SELECT COUNT(*) FROM papers", [], |r| r.get(0))?;
        let to_read: u32 = conn.query_row(
            "SELECT COUNT(*) FROM papers WHERE status = 'to_read'",
            [],
            |r| r.get(0),
        )?;
        let reading: u32 = conn.query_row(
            "SELECT COUNT(*) FROM papers WHERE status = 'reading'",
            [],
            |r| r.get(0),
        )?;
        let read: u32 = conn.query_row(
            "SELECT COUNT(*) FROM papers WHERE status = 'read'",
            [],
            |r| r.get(0),
        )?;
        let cited: u32 = conn.query_row(
            "SELECT COUNT(*) FROM papers WHERE status = 'cited'",
            [],
            |r| r.get(0),
        )?;

        Ok(PaperStats {
            total,
            to_read,
            reading,
            read,
            cited,
        })
    }
}

/// Aggregate counts of papers by reading status.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaperStats {
    /// Total number of tracked papers
    pub total: u32,
    /// Papers not yet read
    pub to_read: u32,
    /// Papers currently being read
    pub reading: u32,
    /// Papers that have been read
    pub read: u32,
    /// Papers that have been cited
    pub cited: u32,
}
