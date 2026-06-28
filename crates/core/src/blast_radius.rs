use crate::models::BlastRadius;
use anyhow::Result;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

pub struct BlastRadiusAnalyzer {
    root: PathBuf,
    import_graph: HashMap<String, HashSet<String>>,
}

impl BlastRadiusAnalyzer {
    pub fn new(root: &Path) -> Self {
        Self {
            root: root.to_path_buf(),
            import_graph: HashMap::new(),
        }
    }

    pub fn build_graph(&mut self) -> Result<()> {
        let files = self.collect_source_files()?;
        self.import_graph.clear();

        for file in &files {
            if let Ok(content) = std::fs::read_to_string(file) {
                let imports = self.extract_imports(&content);
                let rel = self.relative_path(file);
                self.import_graph.insert(rel, imports);
            }
        }

        Ok(())
    }

    pub fn analyze(&self, file_path: &str) -> BlastRadius {
        let direct = self.find_direct_dependents(file_path);
        let mut indirect = HashSet::new();
        let mut depth = 0u32;

        let mut frontier: Vec<String> = direct.iter().cloned().collect();
        let mut visited: HashSet<String> = direct.iter().cloned().collect();

        while !frontier.is_empty() {
            depth += 1;
            let mut next_frontier = Vec::new();

            for dep in &frontier {
                let deps_of_dep = self.find_direct_dependents(dep);
                for d in deps_of_dep {
                    if !visited.contains(&d) {
                        visited.insert(d.clone());
                        indirect.insert(d.clone());
                        next_frontier.push(d);
                    }
                }
            }

            frontier = next_frontier;
            if depth >= 3 {
                break;
            }
        }

        BlastRadius {
            direct_dependents: direct,
            indirect_dependents: indirect.into_iter().collect(),
            depth,
        }
    }

    fn find_direct_dependents(&self, target: &str) -> Vec<String> {
        let mut dependents = Vec::new();

        for (file, imports) in &self.import_graph {
            if file == target {
                continue;
            }
            if imports.contains(target) {
                dependents.push(file.clone());
            }
        }

        dependents.sort();
        dependents
    }

    fn extract_imports(&self, content: &str) -> HashSet<String> {
        let mut imports = HashSet::new();

        for line in content.lines() {
            let trimmed = line.trim();

            // TypeScript/JavaScript imports
            if trimmed.starts_with("import ") || trimmed.starts_with("from ") {
                if let Some(path) = self.extract_import_path(trimmed) {
                    imports.insert(path);
                }
            }

            // Rust use statements
            if trimmed.starts_with("use ") {
                if let Some(path) = self.extract_use_path(trimmed) {
                    imports.insert(path);
                }
            }

            // Go imports
            if trimmed.starts_with("import ") {
                if let Some(path) = self.extract_go_import(trimmed) {
                    imports.insert(path);
                }
            }

            // Python imports
            if trimmed.starts_with("from ") && trimmed.contains(" import ") {
                if let Some(path) = self.extract_py_import(trimmed) {
                    imports.insert(path);
                }
            }
        }

        imports
    }

    fn extract_import_path(&self, line: &str) -> Option<String> {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 {
            let path = parts[1].trim_matches(|c| c == '\'' || c == '"' || c == '`');
            if path.starts_with('.') || path.starts_with('/') {
                return Some(self.normalize_import_path(path));
            }
        }
        None
    }

    fn extract_use_path(&self, line: &str) -> Option<String> {
        let rest = line.strip_prefix("use ")?;
        let path = rest.trim_end_matches(';').trim();
        if path.starts_with("crate::") || path.starts_with("super::") || path.starts_with("self::") {
            Some(path.replace("crate::", "").replace("super::", "../").replace("self::", ""))
        } else {
            None
        }
    }

    fn extract_go_import(&self, line: &str) -> Option<String> {
        if line.contains("\"") {
            let start = line.find('"')? + 1;
            let end = line.rfind('"')?;
            if start < end {
                return Some(line[start..end].to_string());
            }
        }
        None
    }

    fn extract_py_import(&self, line: &str) -> Option<String> {
        let rest = line.strip_prefix("from ")?;
        let parts: Vec<&str> = rest.split(" import ").collect();
        if !parts.is_empty() {
            return Some(parts[0].trim().replace('.', "/"));
        }
        None
    }

    fn normalize_import_path(&self, path: &str) -> String {
        let parts: Vec<&str> = path.split('/').collect();
        let mut normalized = Vec::new();

        for part in parts {
            match part {
                "." | "" => {}
                ".." => {
                    normalized.pop();
                }
                _ => normalized.push(part),
            }
        }

        normalized.join("/")
    }

    fn collect_source_files(&self) -> Result<Vec<PathBuf>> {
        let extensions = ["ts", "tsx", "js", "jsx", "rs", "go", "py"];
        let mut files = Vec::new();

        for entry in walkdir::WalkDir::new(&self.root)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry.file_type().is_file() {
                if let Some(ext) = entry.path().extension() {
                    if extensions.contains(&ext.to_string_lossy().as_ref()) {
                        let path = entry.path().to_path_buf();
                        let rel = self.relative_path(&path);
                        if !rel.starts_with("node_modules")
                            && !rel.starts_with("target")
                            && !rel.starts_with(".git")
                        {
                            files.push(path);
                        }
                    }
                }
            }
        }

        Ok(files)
    }

    fn relative_path(&self, path: &Path) -> String {
        path.strip_prefix(&self.root)
            .unwrap_or(path)
            .to_string_lossy()
            .replace('\\', "/")
    }
}
