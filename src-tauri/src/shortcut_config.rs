use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use tauri::AppHandle;
use tauri::Emitter;
use tauri::Manager;
use tauri_plugin_global_shortcut::{
    Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState,
};

pub type ShortcutConfig = HashMap<String, String>;

fn config_path(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("无法获取应用数据目录: {}", e))?;
    Ok(dir.join("shortcuts.json"))
}

pub fn load(app: &AppHandle) -> ShortcutConfig {
    let path = match config_path(app) {
        Ok(p) => p,
        Err(_) => return ShortcutConfig::new(),
    };
    fs::read_to_string(&path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

pub fn save(app: &AppHandle, config: &ShortcutConfig) -> Result<(), String> {
    let path = config_path(app)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("无法创建配置目录: {}", e))?;
    }
    let json = serde_json::to_string_pretty(config).map_err(|e| format!("序列化失败: {}", e))?;
    fs::write(&path, &json).map_err(|e| format!("写入配置文件失败: {}", e))?;
    Ok(())
}

fn action_event_name(action: &str) -> Option<&'static str> {
    match action {
        "paste_password" => Some("trigger_password_paste"),
        "scan_qr" => Some("trigger_qr_scan"),
        _ => None,
    }
}

fn convert_modifiers_to_winapi(mods: Modifiers) -> u32 {
    let mut flags = 0u32;
    if mods.contains(Modifiers::ALT) {
        flags |= 0x0001; // MOD_ALT
    }
    if mods.contains(Modifiers::CONTROL) {
        flags |= 0x0002; // MOD_CONTROL
    }
    if mods.contains(Modifiers::SHIFT) {
        flags |= 0x0004; // MOD_SHIFT
    }
    if mods.contains(Modifiers::SUPER) {
        flags |= 0x0008; // MOD_WIN
    }
    flags
}

fn code_to_vk(code: Code) -> Option<u32> {
    let k = code as u32;
    let key_a = Code::KeyA as u32;
    let key_z = Code::KeyZ as u32;
    let digit0 = Code::Digit0 as u32;
    let digit9 = Code::Digit9 as u32;
    let f1 = Code::F1 as u32;
    let f24 = Code::F24 as u32;

    if (key_a..=key_z).contains(&k) {
        return Some(k - key_a + 0x41);
    }
    if (digit0..=digit9).contains(&k) {
        return Some(k - digit0 + 0x30);
    }
    if (f1..=f24).contains(&k) {
        return Some(k - f1 + 0x70);
    }

    match code {
        Code::Space => Some(0x20),
        Code::Enter => Some(0x0D),
        Code::Tab => Some(0x09),
        Code::Escape => Some(0x1B),
        Code::Backspace => Some(0x08),
        Code::Delete => Some(0x2E),
        Code::Insert => Some(0x2D),
        Code::Home => Some(0x24),
        Code::End => Some(0x23),
        Code::PageUp => Some(0x21),
        Code::PageDown => Some(0x22),
        Code::ArrowUp => Some(0x26),
        Code::ArrowDown => Some(0x28),
        Code::ArrowLeft => Some(0x25),
        Code::ArrowRight => Some(0x27),
        Code::Comma => Some(0xBC),
        Code::Period => Some(0xBE),
        Code::Minus => Some(0xBD),
        Code::Equal => Some(0xBB),
        Code::Semicolon => Some(0xBA),
        Code::Quote => Some(0xDE),
        Code::Backslash => Some(0xDC),
        Code::Slash => Some(0xBF),
        Code::BracketLeft => Some(0xDB),
        Code::BracketRight => Some(0xDD),
        Code::Backquote => Some(0xC0),
        _ => None,
    }
}

fn default_for(action: &str) -> Option<&'static str> {
    match action {
        "paste_password" => Some("Ctrl+Alt+V"),
        "scan_qr" => Some("Ctrl+Alt+S"),
        _ => None,
    }
}

pub fn register_all(app: &AppHandle, config: &ShortcutConfig) -> Result<(), String> {
    let known_actions = ["paste_password", "scan_qr"];
    for action in known_actions {
        let Some(event_name) = action_event_name(action) else {
            continue;
        };
        let shortcut_str = config
            .get(action)
            .map(String::as_str)
            .or_else(|| default_for(action))
            .ok_or_else(|| format!("未知操作: {}", action))?;
        let shortcut: Shortcut = shortcut_str
            .parse()
            .map_err(|e| format!("无效快捷键 '{}': {}", shortcut_str, e))?;
        app.global_shortcut()
            .on_shortcut(shortcut, move |app, _shortcut, event| {
                if event.state != ShortcutState::Pressed {
                    return;
                }
                let _ = app.emit(event_name, ());
            })
            .map_err(|e| format!("注册快捷键失败: {}", e))?;
    }
    Ok(())
}

pub fn load_effective(app: &AppHandle) -> ShortcutConfig {
    let mut config = load(app);
    for action in ["paste_password", "scan_qr"] {
        if !config.contains_key(action) {
            if let Some(default) = default_for(action) {
                config.insert(action.to_string(), default.to_string());
            }
        }
    }
    config
}

#[tauri::command]
pub fn get_shortcut_config(app: AppHandle) -> ShortcutConfig {
    load_effective(&app)
}

#[tauri::command]
pub fn save_and_apply_shortcut_config(
    app: AppHandle,
    config: ShortcutConfig,
) -> Result<(), String> {
    save(&app, &config)?;
    let _ = app.global_shortcut().unregister_all();
    register_all(&app, &config)?;
    Ok(())
}

#[tauri::command]
pub fn check_shortcut_available(shortcut_str: String) -> Result<bool, String> {
    let shortcut: Shortcut = shortcut_str
        .parse()
        .map_err(|e| format!("无效快捷键: {}", e))?;

    let mods = shortcut.mods;
    let code = shortcut.key;

    if cfg!(windows) {
        use winapi::shared::minwindef::FALSE;
        use winapi::um::winuser::{RegisterHotKey, UnregisterHotKey};
        let mod_flags = convert_modifiers_to_winapi(mods);
        let vk = match code_to_vk(code) {
            Some(vk) => vk,
            None => return Ok(true),
        };
        let result = unsafe { RegisterHotKey(std::ptr::null_mut(), 0xFFFF, mod_flags, vk) };
        let occupied = result == FALSE;
        unsafe { UnregisterHotKey(std::ptr::null_mut(), 0xFFFF) };
        Ok(!occupied)
    } else {
        Ok(true)
    }
}
