use std::{fs, io::Write, path::PathBuf, time::{SystemTime, UNIX_EPOCH}};

use anyhow::{Context, Result};
use dirs::home_dir;
use serde::{Deserialize, Serialize};

use super::history::MessageRecord;

const APP_DIR_NAME: &str = ".spark_cli";
const SESSIONS_DIR: &str = "sessions";
const CURRENT_FILE: &str = "CURRENT";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMeta {
    pub id: String,
    pub name: String,
    pub created_ms: i64,
}

pub struct SessionManager {
    root: PathBuf,
}

impl SessionManager {
    pub fn new() -> Self {
        let root = home_dir().unwrap_or_else(|| PathBuf::from("."));
        let root = root.join(APP_DIR_NAME).join(SESSIONS_DIR);
        Self { root }
    }

    fn now_ms() -> i64 {
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as i64
    }

    pub fn create_session(&self, name: &str) -> Result<String> {
        let id = format!("{}", Self::now_ms());
        let dir = self.root.join(&id);
        fs::create_dir_all(&dir)?;
        // save meta
        let meta = SessionMeta { id: id.clone(), name: name.to_string(), created_ms: Self::now_ms() };
        fs::write(dir.join("meta.json"), serde_json::to_vec_pretty(&meta)?)?;
        // init history file
        fs::write(dir.join("history.jsonl"), b"")?;
        Ok(id)
    }

    pub fn list_sessions(&self) -> Result<Vec<SessionMeta>> {
        let mut results = Vec::new();
        if !self.root.exists() { return Ok(results); }
        for entry in fs::read_dir(&self.root)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                let meta_path = entry.path().join("meta.json");
                if meta_path.exists() {
                    if let Ok(bytes) = fs::read(&meta_path) {
                        if let Ok(meta) = serde_json::from_slice::<SessionMeta>(&bytes) {
                            results.push(meta);
                        }
                    }
                }
            }
        }
        // sort by created_ms desc
        results.sort_by(|a, b| b.created_ms.cmp(&a.created_ms));
        Ok(results)
    }

    pub fn append_message(&self, id: &str, record: &MessageRecord) -> Result<()> {
        let path = self.root.join(id).join("history.jsonl");
        let mut line = serde_json::to_string(record)?;
        line.push('\n');
        fs::OpenOptions::new().create(true).append(true).open(&path)
            .with_context(|| format!("open history failed: {}", path.display()))?
            .write_all(line.as_bytes())?;
        Ok(())
    }

    pub fn delete_session(&self, id: &str) -> Result<()> {
        let dir = self.root.join(id);
        if dir.exists() { fs::remove_dir_all(dir)?; }
        // clear current if it was pointing to this id
        if self.current_session_id().as_deref() == Some(id) {
            let _ = fs::remove_file(self.root.join(CURRENT_FILE));
        }
        Ok(())
    }

    pub fn set_current_session_id(&self, id: &str) -> Result<()> {
        if !self.root.exists() { fs::create_dir_all(&self.root)?; }
        fs::write(self.root.join(CURRENT_FILE), id.as_bytes())?;
        Ok(())
    }

    pub fn current_session_id(&self) -> Option<String> {
        let path = self.root.join(CURRENT_FILE);
        if !path.exists() { return None; }
        let s = fs::read_to_string(path).ok()?;
        let id = s.trim().to_string();
        if id.is_empty() { None } else { Some(id) }
    }
}
