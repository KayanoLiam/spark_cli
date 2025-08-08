#[derive(Debug, Clone)]
pub enum Provider {
    OpenAI,
    Anthropic,
    Google,
    Ollama,
    OpenRouter,
}

impl Provider {
    pub fn from_str(value: &str) -> Self {
        match value.to_lowercase().as_str() {
            "openai" => Self::OpenAI,
            "anthropic" => Self::Anthropic,
            "google" => Self::Google,
            "ollama" => Self::Ollama,
            "openrouter" => Self::OpenRouter,
            _ => Self::OpenRouter,
        }
    }
}
