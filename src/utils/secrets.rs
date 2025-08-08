/// Normalize API keys passed by users.
/// - Trims whitespace
/// - Strips surrounding ASCII or unicode quotes (" ' “ ” ‘ ’)
pub fn normalize_api_key(raw: &str) -> String {
    let mut s = raw.trim().to_string();

    fn is_quote_char(c: char) -> bool {
        matches!(c, '"' | '\'' | '“' | '”' | '‘' | '’')
    }

    while s.starts_with(is_quote_char) && s.len() > 1 {
        s.remove(0);
    }
    while s.ends_with(is_quote_char) && s.len() > 1 {
        s.pop();
    }

    s.trim().to_string()
}
