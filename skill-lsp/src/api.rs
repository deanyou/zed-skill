//! Embedded Cadence SKILL API index generated from official `.fnd` reference docs.
//!
//! Data file: `data/skill_api.json` (~9.6k functions extracted from IC23.1 docs).
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::OnceLock;

#[derive(Debug, Clone, Deserialize)]
pub struct ApiFunction {
    pub name: String,
    pub signature: String,
    #[serde(default)]
    pub description: String,
    pub module: String,
    pub category: String,
}

pub struct ApiIndex {
    by_name: HashMap<String, ApiFunction>,
    /// Lowercased names, sorted — for case-insensitive prefix matching.
    names_lower: Vec<String>,
}

static API: OnceLock<ApiIndex> = OnceLock::new();
const SKILL_API_JSON: &str = include_str!("../data/skill_api.json");

/// Initialize the API index. Safe to call multiple times; only the first call loads data.
pub fn init() {
    let _ = index();
}

pub fn index() -> &'static ApiIndex {
    API.get_or_init(|| {
        #[derive(Deserialize)]
        struct ApiData {
            functions: Vec<ApiFunction>,
        }
        let data: ApiData =
            serde_json::from_str(SKILL_API_JSON).expect("embedded skill_api.json is valid");
        let mut by_name = HashMap::with_capacity(data.functions.len());
        let mut names_lower = Vec::with_capacity(data.functions.len());
        for f in data.functions {
            names_lower.push(f.name.to_lowercase());
            by_name.insert(f.name.to_lowercase(), f);
        }
        names_lower.sort();
        names_lower.dedup();
        ApiIndex { by_name, names_lower }
    })
}

impl ApiIndex {
    /// Exact (case-insensitive) lookup by function name.
    pub fn get(&self, name: &str) -> Option<&ApiFunction> {
        self.by_name.get(&name.to_lowercase())
    }

    /// Case-insensitive prefix matches, ordered by name, capped at `limit`.
    pub fn completions(&self, prefix: &str, limit: usize) -> Vec<&ApiFunction> {
        let needle = prefix.to_lowercase();
        if needle.is_empty() {
            return Vec::new();
        }
        self.names_lower
            .iter()
            .filter(|n| n.starts_with(&needle))
            .take(limit)
            .filter_map(|n| self.by_name.get(n))
            .collect()
    }

    pub fn len(&self) -> usize {
        self.by_name.len()
    }
}
