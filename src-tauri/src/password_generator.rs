use serde::Deserialize;

const UPPERCASE: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ";
const LOWERCASE: &[u8] = b"abcdefghijklmnopqrstuvwxyz";
const DIGITS: &[u8] = b"0123456789";
const SYMBOLS: &[u8] = b"!@#$%^&*()_+-=[]{}|;:,.<>?";
const AMBIGUOUS: &[u8] = b"0O1lI|";

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct PasswordGeneratorConfig {
    pub length: u8,
    pub include_uppercase: bool,
    pub include_lowercase: bool,
    pub include_digits: bool,
    pub include_symbols: bool,
    pub exclude_ambiguous: bool,
    pub custom_symbols: Option<String>,
}

impl Default for PasswordGeneratorConfig {
    fn default() -> Self {
        Self {
            length: 24,
            include_uppercase: true,
            include_lowercase: true,
            include_digits: true,
            include_symbols: true,
            exclude_ambiguous: false,
            custom_symbols: None,
        }
    }
}

#[tauri::command]
pub fn generate_strong_password(config: PasswordGeneratorConfig) -> Result<String, String> {
    let length = config.length.max(4).min(128) as usize;

    let mut groups: Vec<&[u8]> = Vec::new();
    if config.include_uppercase {
        groups.push(UPPERCASE);
    }
    if config.include_lowercase {
        groups.push(LOWERCASE);
    }
    if config.include_digits {
        groups.push(DIGITS);
    }
    if config.include_symbols {
        let symbol_set: &[u8] = match &config.custom_symbols {
            Some(cs) if !cs.is_empty() => cs.as_bytes(),
            _ => SYMBOLS,
        };
        groups.push(symbol_set);
    }
    if groups.is_empty() {
        groups.push(LOWERCASE);
    }

    let mut charset: Vec<u8> = Vec::new();
    for g in &groups {
        charset.extend_from_slice(g);
    }
    if config.exclude_ambiguous {
        charset.retain(|c| !AMBIGUOUS.contains(c));
    }
    if charset.is_empty() {
        return Err("字符集为空，请调整配置".to_string());
    }

    let mut pwd = Vec::with_capacity(length);

    for &g in &groups {
        let filtered: Vec<u8> = if config.exclude_ambiguous {
            g.iter().filter(|c| !AMBIGUOUS.contains(c)).copied().collect()
        } else {
            g.to_vec()
        };
        if filtered.is_empty() {
            continue;
        }
        let mut buf = [0u8; 1];
        getrandom::fill(&mut buf).map_err(|e| e.to_string())?;
        pwd.push(filtered[buf[0] as usize % filtered.len()]);
    }

    while pwd.len() < length {
        let mut buf = [0u8; 1];
        getrandom::fill(&mut buf).map_err(|e| e.to_string())?;
        pwd.push(charset[buf[0] as usize % charset.len()]);
    }

    for i in (1..pwd.len()).rev() {
        let mut buf = [0u8; 1];
        getrandom::fill(&mut buf).map_err(|e| e.to_string())?;
        let j = buf[0] as usize % (i + 1);
        pwd.swap(i, j);
    }

    String::from_utf8(pwd).map_err(|_| "生成的密码包含无效字符".to_string())
}
