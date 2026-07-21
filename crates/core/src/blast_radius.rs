use crate::config::DevCoreConfig;
use crate::error::DevCoreError;
use crate::models::BlastRadius;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

/// Analyzes the downstream impact of file changes by building an import/dependency graph.
pub struct BlastRadiusAnalyzer {
    root: PathBuf,
    config: DevCoreConfig,
    import_graph: HashMap<String, HashSet<String>>,
    reverse_index: HashMap<String, HashSet<String>>,
}

impl BlastRadiusAnalyzer {
    /// Creates a new analyzer rooted at the given project directory.
    /// Call `build_graph` before `analyze` to populate the dependency graph.
    pub fn new(root: &Path) -> Self {
        let config = DevCoreConfig::load(root).unwrap_or_default();
        Self {
            root: root.to_path_buf(),
            config,
            import_graph: HashMap::new(),
            reverse_index: HashMap::new(),
        }
    }

    /// Scans the project for source files and builds the import dependency graph.
    /// Supports TypeScript, JavaScript, Rust, Go, and Python import syntax.
    pub fn build_graph(&mut self) -> Result<(), DevCoreError> {
        let files = self.collect_source_files()?;
        self.import_graph.clear();
        self.reverse_index.clear();

        for file in &files {
            if let Ok(content) = std::fs::read_to_string(file) {
                let imports = extract_imports(&content);
                let rel = self.relative_path(file);
                self.import_graph.insert(rel, imports);
            }
        }

        for (file, imports) in &self.import_graph {
            for import in imports {
                self.reverse_index
                    .entry(import.clone())
                    .or_default()
                    .insert(file.clone());
            }
        }

        Ok(())
    }

    /// Analyzes the blast radius of a file change, returning direct and indirect dependents.
    /// Traverses up to 3 levels of transitive dependencies.
    pub fn analyze(&self, file_path: &str) -> BlastRadius {
        let direct = self.find_direct_dependents(file_path);
        let mut indirect = HashSet::new();
        let mut depth = 0u32;

        let mut frontier: Vec<String> = direct.to_vec();
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

    pub fn list_all_files(&self) -> Vec<String> {
        let mut files: Vec<String> = self.import_graph.keys().cloned().collect();
        files.sort();
        files
    }

    fn find_direct_dependents(&self, target: &str) -> Vec<String> {
        let mut dependents: Vec<String> = self
            .reverse_index
            .get(target)
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .collect();

        let target_stem = std::path::Path::new(target)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("");

        if !target_stem.is_empty() {
            if let Some(stem_deps) = self.reverse_index.get(target_stem) {
                for dep in stem_deps {
                    if !dependents.contains(dep) {
                        dependents.push(dep.clone());
                    }
                }
            }
        }

        dependents.sort();
        dependents
    }

    fn collect_source_files(&self) -> Result<Vec<PathBuf>, DevCoreError> {
        let mut files = Vec::new();

        for entry in walkdir::WalkDir::new(&self.root)
            .follow_links(false)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry.file_type().is_file() {
                if entry.file_type().is_symlink() {
                    continue;
                }

                if let Some(ext) = entry.path().extension() {
                    let ext_str = ext.to_string_lossy();
                    if self.config.source_extensions.contains(&ext_str.to_string()) {
                        let path = entry.path().to_path_buf();

                        if let Ok(metadata) = std::fs::metadata(&path) {
                            if metadata.len() > self.config.max_file_size_bytes {
                                continue;
                            }
                        }

                        let rel = self.relative_path(&path);
                        let excluded = self
                            .config
                            .exclude_dirs
                            .iter()
                            .any(|d| rel.starts_with(d.as_str()));
                        if !excluded {
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

fn extract_imports(content: &str) -> HashSet<String> {
    let mut imports = HashSet::new();

    for line in content.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with("import ") || trimmed.starts_with("from ") {
            if let Some(path) = extract_import_path(trimmed) {
                imports.insert(path);
            }
        }

        if trimmed.starts_with("use ") {
            if let Some(path) = extract_use_path(trimmed) {
                imports.insert(path);
            }
        }

        if trimmed.starts_with("import ") {
            if let Some(path) = extract_go_import(trimmed) {
                imports.insert(path);
            }
        }

        if trimmed.starts_with("from ") && trimmed.contains(" import ") {
            if let Some(path) = extract_py_import(trimmed) {
                imports.insert(path);
            }
        }
    }

    imports
}

fn extract_import_path(line: &str) -> Option<String> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() >= 2 {
        let path = parts[1].trim_matches(|c| c == '\'' || c == '"' || c == '`');
        if path.starts_with('.') || path.starts_with('/') {
            return Some(normalize_import_path(path));
        }
    }
    None
}

fn extract_use_path(line: &str) -> Option<String> {
    let rest = line.strip_prefix("use ")?;
    let path = rest.trim_end_matches(';').trim();

    if let Some(inner) = path.strip_prefix("crate::") {
        let parts: Vec<&str> = inner.split("::").collect();
        if !parts.is_empty() && parts[0] != "*" {
            return Some(parts[0].to_string());
        }
    }

    if let Some(inner) = path.strip_prefix("super::") {
        let parts: Vec<&str> = inner.split("::").collect();
        if !parts.is_empty() && parts[0] != "*" {
            return Some(parts[0].to_string());
        }
    }

    if let Some(inner) = path.strip_prefix("self::") {
        let parts: Vec<&str> = inner.split("::").collect();
        if !parts.is_empty() && parts[0] != "*" {
            return Some(parts[0].to_string());
        }
    }

    None
}

fn extract_go_import(line: &str) -> Option<String> {
    if line.contains('"') {
        let start = line.find('"')? + 1;
        let end = line.rfind('"')?;
        if start < end {
            return Some(line[start..end].to_string());
        }
    }
    None
}

fn extract_py_import(line: &str) -> Option<String> {
    let rest = line.strip_prefix("from ")?;
    let parts: Vec<&str> = rest.split(" import ").collect();
    if !parts.is_empty() {
        return Some(parts[0].trim().replace('.', "/"));
    }
    None
}

fn normalize_import_path(path: &str) -> String {
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
