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
            serde_json::Value::Object(obj) => {
                !matches!(obj.get(child), Some(serde_json::Value::Bool(false)))
            }
            _ => true,
        }
    }

    /// preset 配下の child rule の option value を取り出す。
    pub fn preset_child_option(
        &self,
        preset: &str,
        child: &str,
        key: &str,
    ) -> Option<&serde_json::Value> {
        let preset_v = self.rules.get(preset)?;
        let obj = preset_v.as_object()?;
        let child_v = obj.get(child)?;
        child_v.as_object()?.get(key)
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
                if p.is_absolute() { p } else { base_dir.join(p) }
            })
            .collect()
    }
}
