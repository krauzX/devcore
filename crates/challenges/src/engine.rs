use crate::pack::{builtin_packs, Difficulty, Problem, ProblemPack};
use std::fs;
use std::path::{Path, PathBuf};

pub struct ChallengeEngine {
    builtin: Vec<ProblemPack>,
    installed: Vec<ProblemPack>,
    packs_dir: PathBuf,
}

impl ChallengeEngine {
    pub fn new(data_dir: &Path) -> Self {
        let packs_dir = data_dir.join("challenges").join("packs");
        let _ = fs::create_dir_all(&packs_dir);
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
                    if let Ok(data) = fs::read_to_string(&pack_file) {
                        if let Ok(mut pack) = serde_json::from_str::<ProblemPack>(&data) {
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
}
