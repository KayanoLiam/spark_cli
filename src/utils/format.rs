use console::style;

pub fn success(msg: &str) -> String { style(msg).green().to_string() }
pub fn warn(msg: &str) -> String { style(msg).yellow().to_string() }
pub fn error(msg: &str) -> String { style(msg).red().to_string() }
