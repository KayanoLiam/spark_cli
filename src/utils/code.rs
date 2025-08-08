use regex::Regex;

#[derive(Debug, Clone)]
pub struct CodeBlock {
    pub language: Option<String>,
    pub filename: Option<String>,
    pub content: String,
}

/// Extract triple-backtick fenced code blocks.
/// Supports forms like:
/// ```lang\n...```
/// ```filename.ext\n...```
pub fn extract_code_blocks(text: &str) -> Vec<CodeBlock> {
    // This regex captures the fence info line and body; non-greedy body until next ```
    // (?s) enables dot to match newline
    let re = Regex::new(r"(?s)```\s*([^\n`]*)\n(.*?)```\s*").expect("valid regex");
    let mut results = Vec::new();
    for caps in re.captures_iter(text) {
        let info = caps.get(1).map(|m| m.as_str().trim()).unwrap_or("");
        let body = caps.get(2).map(|m| m.as_str()).unwrap_or("");
        let (language, filename) = if info.contains('.') && !info.contains(' ') {
            (None, Some(info.to_string()))
        } else if info.is_empty() {
            (None, None)
        } else {
            (Some(info.to_string()), None)
        };
        results.push(CodeBlock {
            language,
            filename,
            content: body.to_string(),
        });
    }
    results
}

pub fn choose_best_block<'a>(blocks: &'a [CodeBlock], preferred_langs: &[&str]) -> Option<&'a CodeBlock> {
    if blocks.is_empty() { return None; }
    // exact language match first
    for &lang in preferred_langs {
        if let Some(b) = blocks.iter().find(|b| b.language.as_deref().map(|s| s.eq_ignore_ascii_case(lang)).unwrap_or(false)) {
            return Some(b);
        }
    }
    // filename extension match
    for &lang in preferred_langs {
        let ext = guess_ext_from_lang(lang);
        if let Some(b) = blocks.iter().find(|b| b.filename.as_deref().map(|f| f.ends_with(&format!(".{}", ext))).unwrap_or(false)) {
            return Some(b);
        }
    }
    // otherwise, first block
    blocks.get(0)
}

pub fn guess_ext_from_lang(lang: &str) -> &str {
    match lang.to_ascii_lowercase().as_str() {
        "cpp" | "c++" => "cpp",
        "c" => "c",
        "rust" | "rs" => "rs",
        "python" | "py" => "py",
        "typescript" | "ts" => "ts",
        "javascript" | "js" => "js",
        "go" => "go",
        "java" => "java",
        _ => "txt",
    }
}
