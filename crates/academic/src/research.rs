use anyhow::Result;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Paper {
    pub id: String,
    pub title: String,
    pub authors: Option<String>,
    pub venue: Option<String>,
    pub year: Option<u32>,
    pub doi: Option<String>,
    pub arxiv_id: Option<String>,
    pub status: PaperStatus,
    pub tags: Option<String>,
    pub notes: Option<String>,
    pub added_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PaperStatus {
    ToRead,
    Reading,
    Read,
    Cited,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaperStats {
    pub total: u32,
    pub to_read: u32,
    pub reading: u32,
    pub read: u32,
    pub cited: u32,
}
