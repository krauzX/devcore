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

static BUILTIN_PROJECTS_JSON: &[&str] = &[
    include_str!("../data/projects/shell.json"),
    include_str!("../data/projects/git.json"),
    include_str!("../data/projects/http-server.json"),
    include_str!("../data/projects/redis.json"),
    include_str!("../data/projects/compiler.json"),
    include_str!("../data/projects/text-editor.json"),
    include_str!("../data/projects/interpreter.json"),
    include_str!("../data/projects/database.json"),
    include_str!("../data/projects/regex.json"),
    include_str!("../data/projects/rustlings.json"),
];

pub fn load_builtin_projects() -> Vec<ProjectPack> {
    BUILTIN_PROJECTS_JSON
        .iter()
        .enumerate()
        .filter_map(|(i, json_str)| {
            serde_json::from_str(json_str)
                .map_err(|e| eprintln!("Failed to parse project JSON at index {}: {}", i, e))
                .ok()
        })
        .collect()
}

pub struct ProjectEngine {
    builtin: Vec<ProjectPack>,
    installed: Vec<ProjectPack>,
    projects_dir: PathBuf,
}

impl ProjectEngine {
    pub fn new(data_dir: &Path) -> Self {
        let projects_dir = data_dir.join("projects");
        let _ = fs::create_dir_all(&projects_dir).ok();
        let installed = Self::load_installed_projects(&projects_dir);
        Self {
            builtin: load_builtin_projects(),
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
