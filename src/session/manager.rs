use anyhow::Result;

pub struct SessionManager;

impl SessionManager {
    pub fn new() -> Self { Self }

    pub fn create_session(&self, _name: &str) -> Result<()> { Ok(()) }

    pub fn list_sessions(&self) -> Result<()> { Ok(()) }

    pub fn load_session(&self, _id: &str) -> Result<()> { Ok(()) }

    pub fn delete_session(&self, _id: &str) -> Result<()> { Ok(()) }
}
