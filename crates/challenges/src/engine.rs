use crate::pack::{builtin_packs, Difficulty, Problem, ProblemPack};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnlineProblem {
    pub fid: u32,
    pub title: String,
    pub slug: String,
    pub difficulty: String,
    pub acceptance: f64,
    pub frequency: f64,
    pub url: String,
    #[serde(default)]
    pub is_premium: bool,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub hints: Vec<String>,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub solution: Option<String>,
    #[serde(default)]
    pub company: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnlineProblemListResult {
    pub problems: Vec<OnlineProblem>,
    pub total: usize,
    pub page: usize,
    pub per_page: usize,
    pub total_pages: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(non_snake_case)]
struct OfflineStat {
    #[serde(default)]
    frontend_question_id: Option<u32>,
    #[serde(default)]
    question__title: Option<String>,
    #[serde(default)]
    question__title_slug: Option<String>,
    #[serde(default)]
    total_acs: Option<u64>,
    #[serde(default)]
    total_submitted: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct OfflineDifficulty {
    #[serde(default)]
    level: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct OfflineStatStatusPair {
    #[serde(default)]
    stat: Option<OfflineStat>,
    #[serde(default)]
    difficulty: Option<OfflineDifficulty>,
    #[serde(default)]
    paid_only: Option<bool>,
    #[serde(default)]
    frequency: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct OfflineWrapper {
    #[serde(default)]
    stat_status_pairs: Option<Vec<OfflineStatStatusPair>>,
}

#[derive(Debug, Clone)]
pub struct OfflineProblem {
    pub fid: u32,
    pub title: String,
    pub slug: String,
    pub difficulty: Difficulty,
    pub acceptance: f64,
    pub is_premium: bool,
}

#[derive(Debug, Clone)]
pub struct OfflineProblemListResult {
    pub problems: Vec<OfflineProblem>,
    pub total: usize,
    pub page: usize,
    pub per_page: usize,
    pub total_pages: usize,
}

pub struct ChallengeEngine {
    builtin: Vec<ProblemPack>,
    installed: Vec<ProblemPack>,
    packs_dir: PathBuf,
}

static ONLINE_PROBLEMS: Lazy<Vec<OnlineProblem>> = Lazy::new(|| {
    let data = include_str!("../data/leetcode_clean.json");
    serde_json::from_str(data).unwrap_or_default()
});

static OFFLINE_PROBLEMS: Lazy<Vec<OfflineProblem>> = Lazy::new(|| {
    let data = include_str!("../data/leetcode_official.json");
    let wrapper: OfflineWrapper = serde_json::from_str(data).unwrap_or(OfflineWrapper { stat_status_pairs: None });
    let pairs = wrapper.stat_status_pairs.unwrap_or_default();
    let mut problems: Vec<OfflineProblem> = Vec::with_capacity(pairs.len());
    for pair in pairs {
        let stat = match pair.stat {
            Some(s) => s,
            None => continue,
        };
        let diff = pair.difficulty.as_ref().and_then(|d| d.level);
        let difficulty = diff
            .and_then(Difficulty::from_level)
            .unwrap_or(Difficulty::Easy);
        let submitted = stat.total_submitted.unwrap_or(0);
        let acs = stat.total_acs.unwrap_or(0);
        let acceptance = if submitted > 0 {
            (acs as f64 / submitted as f64) * 100.0
        } else {
            0.0
        };
        problems.push(OfflineProblem {
            fid: stat.frontend_question_id.unwrap_or(0),
            title: stat.question__title.unwrap_or_default(),
            slug: stat.question__title_slug.unwrap_or_default(),
            difficulty,
            acceptance,
            is_premium: pair.paid_only.unwrap_or(false),
        });
    }
    problems
});

impl ChallengeEngine {
    pub fn new(data_dir: &Path) -> Self {
        let packs_dir = data_dir.join("challenges").join("packs");
        let _ = fs::create_dir_all(&packs_dir).ok();
        let installed = Self::load_installed_packs(&packs_dir);
        Self {
            builtin: builtin_packs(),
            installed,
            packs_dir,
        }
    }

    fn load_installed_packs(packs_dir: &Path) -> Vec<ProblemPack> {
        let mut packs = Vec::new();
        if let Ok(entries) = fs::read_dir(packs_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let pack_file = path.join("pack.json");
                    if let Ok(pack_content) = fs::read_to_string(&pack_file) {
                        if let Ok(mut pack) = serde_json::from_str::<ProblemPack>(&pack_content) {
                            pack.installed = true;
                            packs.push(pack);
                        }
                    }
                }
            }
        }
        packs
    }

    pub fn list_available(&self) -> &[ProblemPack] {
        &self.builtin
    }

    pub fn list_installed(&self) -> Vec<&ProblemPack> {
        self.installed.iter().collect()
    }

    pub fn install_pack(&mut self, pack_id: &str) -> Result<(), String> {
        if self.installed.iter().any(|p| p.id == pack_id) {
            return Err(format!("Pack '{}' is already installed", pack_id));
        }
        let pack = self
            .builtin
            .iter()
            .find(|p| p.id == pack_id)
            .ok_or_else(|| format!("Pack '{}' not found", pack_id))?;
        let pack_dir = self.packs_dir.join(pack_id);
        fs::create_dir_all(&pack_dir).map_err(|e| e.to_string())?;
        let mut installed_pack = pack.clone();
        installed_pack.installed = true;
        let json = serde_json::to_string_pretty(&installed_pack).map_err(|e| e.to_string())?;
        fs::write(pack_dir.join("pack.json"), json).map_err(|e| e.to_string())?;
        for problem in &installed_pack.problems {
            let problem_dir = pack_dir.join(&problem.id);
            fs::create_dir_all(&problem_dir).map_err(|e| e.to_string())?;
            fs::write(
                problem_dir.join("skeleton.rs"),
                &problem.skeleton,
            )
            .map_err(|e| e.to_string())?;
            fs::write(
                problem_dir.join("solution.rs"),
                &problem.solution,
            )
            .map_err(|e| e.to_string())?;
        }
        self.installed.push(installed_pack);
        Ok(())
    }

    pub fn remove_pack(&mut self, pack_id: &str) -> Result<(), String> {
        let idx = self
            .installed
            .iter()
            .position(|p| p.id == pack_id)
            .ok_or_else(|| format!("Pack '{}' is not installed", pack_id))?;
        let pack_dir = self.packs_dir.join(pack_id);
        fs::remove_dir_all(&pack_dir).map_err(|e| e.to_string())?;
        self.installed.remove(idx);
        Ok(())
    }

    pub fn get_problem(&self, pack_id: &str, problem_id: &str) -> Option<&Problem> {
        let all_packs = self.builtin.iter().chain(self.installed.iter());
        for pack in all_packs {
            if pack.id == pack_id {
                return pack.problems.iter().find(|p| p.id == problem_id);
            }
        }
        None
    }

    pub fn list_problems(&self, pack_id: &str) -> Vec<&Problem> {
        let all_packs = self.builtin.iter().chain(self.installed.iter());
        for pack in all_packs {
            if pack.id == pack_id {
                return pack.problems.iter().collect();
            }
        }
        Vec::new()
    }

    pub fn problems_by_difficulty(&self, difficulty: Difficulty) -> Vec<(&str, &Problem)> {
        let all_packs = self.builtin.iter().chain(self.installed.iter());
        all_packs
            .filter(|p| p.difficulty == difficulty)
            .flat_map(|pack| {
                pack.problems
                    .iter()
                    .map(move |problem| (pack.id.as_str(), problem))
            })
            .collect()
    }

    pub fn load_online_problems(&self) -> &Vec<OnlineProblem> {
        &ONLINE_PROBLEMS
    }

    pub fn list_online(
        &self,
        difficulty: Option<&str>,
        page: usize,
        per_page: usize,
    ) -> OnlineProblemListResult {
        let all = self.load_online_problems();
        let filtered: Vec<OnlineProblem> = all
            .iter()
            .filter(|p| {
                if let Some(diff) = difficulty {
                    p.difficulty.to_string().to_lowercase() == diff.to_lowercase()
                } else {
                    true
                }
            })
            .cloned()
            .collect();
        let total = filtered.len();
        let total_pages = if per_page > 0 {
            total.div_ceil(per_page)
        } else {
            1
        };
        let page = page.max(1).min(total_pages.max(1));
        let start = ((page - 1) * per_page).min(total);
        let end = (start + per_page).min(total);
        let problems = filtered[start..end].to_vec();

        OnlineProblemListResult {
            problems,
            total,
            page,
            per_page,
            total_pages,
        }
    }

    pub fn load_offline_problems(&self) -> &Vec<OfflineProblem> {
        &OFFLINE_PROBLEMS
    }

    pub fn list_offline(
        &self,
        difficulty: Option<&str>,
        page: usize,
        per_page: usize,
    ) -> OfflineProblemListResult {
        let all = self.load_offline_problems();
        let filtered: Vec<&OfflineProblem> = all
            .iter()
            .filter(|p| {
                if let Some(diff) = difficulty {
                    p.difficulty.to_string().to_lowercase() == diff.to_lowercase()
                } else {
                    true
                }
            })
            .collect();
        let total = filtered.len();
        let total_pages = if per_page > 0 {
            total.div_ceil(per_page)
        } else {
            1
        };
        let page = page.max(1).min(total_pages.max(1));
        let start = ((page - 1) * per_page).min(total);
        let end = (start + per_page).min(total);
        let problems = filtered[start..end].iter().map(|p| (*p).clone()).collect();

        OfflineProblemListResult {
            problems,
            total,
            page,
            per_page,
            total_pages,
        }
    }

    pub fn get_offline_problem(&self, slug: &str) -> Option<OfflineProblem> {
        let all = self.load_offline_problems();
        all.iter().find(|p| p.slug == slug).cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_engine_new() {
        let dir = tempdir().unwrap();
        let engine = ChallengeEngine::new(dir.path());
        assert!(dir.path().join("challenges/packs").exists());
        assert!(engine.installed.is_empty());
    }

    #[test]
    fn test_list_available() {
        let dir = tempdir().unwrap();
        let engine = ChallengeEngine::new(dir.path());
        let packs = engine.list_available();
        assert_eq!(packs.len(), 5);
    }
}
