use serde::Deserialize;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Default, Deserialize)]
pub struct TextlintRc {
    #[serde(default)]
    pub plugins: serde_json::Value,
    #[serde(default)]
    pub rules: BTreeMap<String, serde_json::Value>,
}

impl TextlintRc {
    pub fn from_path(path: &Path) -> anyhow::Result<Self> {
        let s = std::fs::read_to_string(path)?;
        let rc: TextlintRc = serde_json::from_str(&s)?;
        Ok(rc)
    }

    pub fn rule_enabled(&self, name: &str) -> bool {
        match self.rules.get(name) {
            None => false,
            Some(v) => match v {
                serde_json::Value::Bool(b) => *b,
                serde_json::Value::Null => false,
                _ => true,
            },
        }
    }

    pub fn preset_child_enabled(&self, preset: &str, child: &str) -> bool {
        let v = match self.rules.get(preset) {
            None => return false,
            Some(v) => v,
        };
        match v {
            serde_json::Value::Bool(b) => *b,
            serde_json::Value::Object(obj) => match obj.get(child) {
                Some(serde_json::Value::Bool(false)) => false,
                _ => true,
            },
            _ => true,
        }
    }

    pub fn prh_rule_paths(&self, base_dir: &Path) -> Vec<PathBuf> {
        let prh = match self.rules.get("prh") {
            Some(serde_json::Value::Object(o)) => o,
            _ => return Vec::new(),
        };
        let paths = match prh.get("rulePaths") {
            Some(serde_json::Value::Array(a)) => a,
            _ => return Vec::new(),
        };
        paths
            .iter()
            .filter_map(|v| v.as_str())
            .map(|s| {
                let p = PathBuf::from(s);
                if p.is_absolute() {
                    p
                } else {
                    base_dir.join(p)
                }
            })
            .collect()
    }
}
