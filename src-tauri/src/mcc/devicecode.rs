use crate::domain::DeviceCodePrompt;
use once_cell::sync::Lazy;
use regex::Regex;

/// Parse a Microsoft device-code prompt from an MCC stdout line.
///
/// MCC's device-code flow prints something like:
///   "To sign in, use a web browser to open the page https://microsoft.com/devicelogin
///    and enter the code ABCD1234 to authenticate."
static DEVICE_CODE_RE: Lazy<Regex> = Lazy::new(|| {
    // Capture the device code: a 6-12 char token of uppercase letters/digits that has
    // at least one digit (codes like F8BQX9KJ). We require the surrounding context to
    // mention the device-login flow (checked by the caller) to avoid false positives.
    Regex::new(r"\b([A-Z0-9]{6,12})\b").expect("device code regex")
});

static URL_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"(?i)(https?://[^\s,\)"]+)"#).expect("url regex")
});

/// If the line looks like a device-code prompt, return the parsed prompt.
pub fn parse_device_code(line: &str) -> Option<DeviceCodePrompt> {
    let lower = line.to_lowercase();
    if !lower.contains("devicelogin")
        && !lower.contains("device code")
        && !lower.contains("enter the code")
        && !lower.contains("use a web browser")
    {
        return None;
    }

    let url = URL_RE
        .captures(line)
        .and_then(|c| c.get(1).map(|m| m.as_str().to_string()))
        .unwrap_or_else(|| "https://microsoft.com/devicelogin".to_string());

    // Find a code-like token that is not part of the URL.
    for cap in DEVICE_CODE_RE.captures_iter(line) {
        if let Some(m) = cap.get(1) {
            let v = m.as_str();
            if v.chars().any(|c| c.is_ascii_digit())
                && v.chars()
                    .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit())
                && !url.contains(v)
            {
                return Some(DeviceCodePrompt {
                    code: v.to_string(),
                    url,
                });
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_typical_line() {
        let line = "To sign in, use a web browser to open the page https://microsoft.com/devicelogin and enter the code ABCD1234 to authenticate.";
        let p = parse_device_code(line).expect("should parse");
        assert_eq!(p.url, "https://microsoft.com/devicelogin");
        assert_eq!(p.code, "ABCD1234");
    }

    #[test]
    fn ignores_normal_chat() {
        assert!(parse_device_code("<Steve> hello world").is_none());
    }
}
