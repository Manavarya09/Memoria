use regex::Regex;

pub fn clean_text(text: &str) -> String {
    let mut result = text.to_string();
    result = result.replace("\r\n", "\n");
    result = result.replace('\r', "\n");
    result.trim().to_string()
}

pub fn extract_urls(text: &str) -> Vec<String> {
    let re = match Regex::new("https?://[^\\s<>\\\"]+") {
        Ok(r) => r,
        Err(_) => return Vec::new(),
    };
    re.find_iter(text).map(|m| m.as_str().to_string()).collect()
}

pub fn extract_file_paths(text: &str) -> Vec<String> {
    let re = match Regex::new("(?:/[\\w.-]+)+") {
        Ok(r) => r,
        Err(_) => return Vec::new(),
    };
    re.find_iter(text)
        .map(|m| m.as_str().to_string())
        .filter(|p| {
            let pl = p.to_lowercase();
            pl.contains("/users/")
                || pl.contains("/home/")
                || pl.contains("/var/")
                || pl.ends_with(".txt")
                || pl.ends_with(".pdf")
                || pl.ends_with(".md")
                || pl.ends_with(".js")
                || pl.ends_with(".rs")
                || pl.ends_with(".py")
                || pl.ends_with(".go")
        })
        .collect()
}

pub fn detect_content_type(text: &str, url: Option<&str>, file_path: Option<&str>) -> String {
    if let Some(u) = url {
        if u.contains("github.com") || u.contains("gitlab.com") {
            return "code".to_string();
        }
        if u.contains("stackoverflow.com") {
            return "qa".to_string();
        }
        if u.contains("youtube.com") || u.contains("vimeo.com") {
            return "video".to_string();
        }
        if u.contains("docs.") || u.contains("wikipedia.org") {
            return "documentation".to_string();
        }
    }

    if let Some(path) = file_path {
        let pl = path.to_lowercase();
        let code_exts = ["rs", "js", "ts", "py", "go", "java"];
        for ext in code_exts {
            if pl.ends_with(&format!(".{}", ext)) {
                return "code".to_string();
            }
        }
        if pl.ends_with(".pdf") {
            return "document".to_string();
        }
        if pl.ends_with(".md") || pl.ends_with(".txt") {
            return "text".to_string();
        }
    }

    let code_indicators = [
        "function", "const ", "let ", "var ", "def ", "fn ", "pub ", "import ",
    ];
    let code_count = code_indicators
        .iter()
        .filter(|ind| text.contains(*ind))
        .count();

    if code_count >= 2 {
        return "code".to_string();
    }

    "general".to_string()
}

pub fn tokenize(text: &str) -> Vec<String> {
    text.split_whitespace()
        .map(|s| s.to_lowercase())
        .filter(|w| w.len() > 1)
        .collect()
}

pub fn truncate_for_embedding(text: &str, max_chars: usize) -> String {
    if text.len() <= max_chars {
        return text.to_string();
    }

    let mut truncated = text.chars().take(max_chars).collect::<String>();

    if let Some(pos) = truncated.rfind(' ') {
        truncated.truncate(pos);
    }

    truncated
}
