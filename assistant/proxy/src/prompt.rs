//! Loads the system prompt, knowledge base and tool list from the assistant folder.

use anyhow::{Context, Result, bail};
use serde_json::Value;
use std::collections::HashMap;
use std::path::Path;

pub struct Assets {
    pub system_prompt: String,
    /// Tools in DeepSeek's format, with the `x_jobex` blocks removed.
    pub tools: Value,
    /// Tool name -> (English label, Polish label) shown while a tool runs.
    pub labels: HashMap<String, (String, String)>,
}

impl Assets {
    pub fn load(dir: &Path) -> Result<Self> {
        let read = |name: &str| std::fs::read_to_string(dir.join(name)).with_context(|| format!("reading {}", dir.join(name).display()));
        let template = read("SYSTEM_PROMPT.md")?;
        let knowledge = read("KNOWLEDGE.md")?;
        if !template.contains("{{KNOWLEDGE}}") {
            bail!("SYSTEM_PROMPT.md has no {{{{KNOWLEDGE}}}} placeholder");
        }
        let system_prompt = template.replace("{{KNOWLEDGE}}", knowledge.trim());
        let raw: Value = serde_json::from_str(&read("tools.json")?).context("tools.json is not valid JSON")?;
        Self::from_parts(system_prompt, &raw)
    }

    pub fn from_parts(system_prompt: String, raw_tools: &Value) -> Result<Self> {
        let list = raw_tools.get("tools").and_then(Value::as_array).context("tools.json has no `tools` array")?;
        let mut tools = Vec::with_capacity(list.len());
        let mut labels = HashMap::new();
        for t in list {
            let name = t.pointer("/function/name").and_then(Value::as_str).context("a tool has no function.name")?.to_string();
            let meta = t.get("x_jobex");
            let en = meta.and_then(|m| m.get("label")).and_then(Value::as_str).unwrap_or("Working on it").to_string();
            let pl = meta.and_then(|m| m.get("label_pl")).and_then(Value::as_str).unwrap_or("Pracuję nad tym").to_string();
            labels.insert(name, (en, pl));
            let mut clean = t.clone();
            if let Some(obj) = clean.as_object_mut() {
                obj.retain(|k, _| !k.starts_with("x_"));
            }
            tools.push(clean);
        }
        Ok(Self { system_prompt, tools: Value::Array(tools), labels })
    }

    pub fn label(&self, tool: &str, polish: bool) -> String {
        match self.labels.get(tool) {
            Some((en, pl)) => {
                if polish {
                    pl.clone()
                } else {
                    en.clone()
                }
            }
            None => {
                if polish {
                    "Pracuję nad tym".into()
                } else {
                    "Working on it".into()
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loads_the_real_assistant_folder() {
        let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
        let a = Assets::load(&dir).unwrap();
        assert!(a.system_prompt.contains("# Knowledge base"));
        assert!(a.system_prompt.contains("What Jobex is"));
        assert!(!a.system_prompt.contains("{{KNOWLEDGE}}"));
        let tools = a.tools.as_array().unwrap();
        assert_eq!(tools.len(), 22);
        assert!(tools.iter().all(|t| t.get("x_jobex").is_none()));
        assert_eq!(a.label("search_shifts", false), "Finding suitable shifts");
        assert_eq!(a.label("draft_shift", true), "Tworzę formularz zmiany");
    }
}
