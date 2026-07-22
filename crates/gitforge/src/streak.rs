use std::collections::BTreeSet;
use std::path::Path;

use anyhow::Result;
use chrono::{DateTime, NaiveDate, Utc};
use git2::Repository;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Streak {
    pub current: u32,
    pub longest: u32,
    pub total_days: u32,
    pub last_commit_date: Option<NaiveDate>,
}

pub fn compute_streak(repo_path: &Path) -> Result<Streak> {
    let repo = Repository::open(repo_path)?;
    let mut revwalk = repo.revwalk()?;
    revwalk.set_sorting(git2::Sort::TIME)?;
    revwalk.push_head()?;

    let mut dates: BTreeSet<NaiveDate> = BTreeSet::new();
    let mut last_ts: Option<i64> = None;

    for oid_result in revwalk {
        let oid = oid_result?;
        let commit = repo.find_commit(oid)?;
        let ts = commit.time().seconds();

        if last_ts.is_none() {
            last_ts = Some(ts);
        }

        if let Some(dt) = DateTime::from_timestamp(ts, 0) {
            dates.insert(dt.date_naive());
        }
    }

    if dates.is_empty() {
        return Ok(Streak {
            current: 0,
            longest: 0,
            total_days: 0,
            last_commit_date: None,
        });
    }

    let last_commit_date = last_ts
        .and_then(|ts| DateTime::from_timestamp(ts, 0))
        .map(|dt| dt.date_naive());

    let today = Utc::now().date_naive();
    let dates_vec: Vec<NaiveDate> = dates.into_iter().collect();
    let total_days = dates_vec.len() as u32;

    let mut longest = 0u32;
    let mut current_streak = 0u32;

    for i in 0..dates_vec.len() {
        if i == 0 {
            current_streak = 1;
        } else {
            let prev = dates_vec[i - 1];
            let curr = dates_vec[i];
            if curr - prev == chrono::Duration::days(1) {
                current_streak += 1;
            } else {
                current_streak = 1;
            }
        }
        if current_streak > longest {
            longest = current_streak;
        }
    }

    let mut current = 0u32;
    if let Some(&last_date) = dates_vec.last() {
        if last_date == today || last_date == today - chrono::Duration::days(1) {
            let streak_end = dates_vec.len();
            for i in (0..dates_vec.len()).rev() {
                let is_first = i + 1 == streak_end;
                let is_consecutive = i + 1 < streak_end
                    && dates_vec[i + 1] - dates_vec[i] == chrono::Duration::days(1);
                if is_first || is_consecutive {
                    current += 1;
                } else {
                    break;
                }
                if i == 0 && current > 0 {
                    break;
                }
            }
        }
    }

    Ok(Streak {
        current,
        longest,
        total_days,
        last_commit_date,
    })
}
