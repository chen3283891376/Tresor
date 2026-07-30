mod password_generator;
mod storage;
mod utils;
mod twofa;

use std::fs;
use std::sync::Mutex;
use enigo::{Enigo, Keyboard, Settings};
use tauri::{AppHandle, Emitter, Listener, State};
use tauri_plugin_dialog::{DialogExt, FilePath};
use tauri_plugin_global_shortcut::{Code, Modifiers, Shortcut, ShortcutState};
use zeroize::Zeroize;

use crate::password_generator::generate_strong_password;
use crate::twofa::{scan_qr_from_image, scan_qr_from_screenshot};
use crate::storage::{check_all_password_leaks, PasswordLeakCheckResult};
use storage::{DecryptedEntry, EntryMetaPreview};
use twofa::{DecryptedTwoFAEntry, TwoFAEntryPreview};
use utils::{clear_active_master_key, get_active_master_key, set_active_master_key, VaultError};
use utils::{clear_need_paste_pwd, get_need_paste_pwd};

struct KeyFilePathState(Mutex<Option<String>>);
struct VaultSaltState(Mutex<Option<[u8;16]>>);

#[tauri::command]
async fn open_key_file_picker(
    app: AppHandle,
    state: State<'_, KeyFilePathState>
) -> Result<bool, String> {
    let file_opt: Option<FilePath> = app
        .dialog()
        .file()
        .add_filter("密钥分片文件", &["key"])
        .blocking_pick_file();

    match file_opt {
        Some(file_path) => {
            let path_ref = file_path.as_path();
            let full_path = path_ref.expect("REASON").to_string_lossy().to_string();

            let mut guard = state.0.lock().unwrap();
            *guard = Some(full_path);

            Ok(true)
        }
        None => Ok(false),
    }
}

#[tauri::command]
fn clear_stored_key_path(state: State<KeyFilePathState>) {
    let mut guard = state.0.lock().unwrap();
    guard.take();
}

#[tauri::command]
async fn register_vault(
    app: AppHandle,
    user_pwd: String,
    salt_state: State<'_, VaultSaltState>,
) -> Result<(), String> {
    let mut pwd_buf = user_pwd.into_bytes();

    let mut new_key = [0u8; 32];
    getrandom::fill(&mut new_key).map_err(|e| e.to_string())?;

    let mut vault_salt = [0u8;16];
    getrandom::fill(&mut vault_salt).map_err(|e| e.to_string())?;

    let save_opt: Option<FilePath> = app
        .dialog()
        .file()
        .add_filter("密钥分片文件", &["key"])
        .set_file_name("vault.key")
        .blocking_save_file();

    match save_opt {
        Some(save_path) => {
            let path = save_path.as_path().ok_or(VaultError::UserCancelSave.to_string())?;
            fs::write(path.to_string_lossy().to_string(), &new_key).map_err(|e| e.to_string())?;
        }
        None => {
            return Err(VaultError::UserCancelSave.to_string());
        }
    };

    set_active_master_key(&pwd_buf, &new_key, &vault_salt).map_err(|e| e.to_string())?;

    let mut salt_guard = salt_state.0.lock().unwrap();
    *salt_guard = Some(vault_salt);

    pwd_buf.zeroize();
    new_key.zeroize();
    Ok(())
}

#[tauri::command]
async fn unlock_vault(
    user_pwd: String,
    state: State<'_, KeyFilePathState>,
    salt_state: State<'_, VaultSaltState>,
) -> Result<(), String> {
    let mut pwd_buf = user_pwd.into_bytes();
    let path_guard = state.0.lock().unwrap();
    let key_path = path_guard
        .as_ref()
        .ok_or(VaultError::NoKeyFile.to_string())?;
    let key_file_data = fs::read(key_path).map_err(|e| e.to_string())?;
    drop(path_guard);

    let vault_path = storage::get_vault_storage_path().map_err(|e|e.to_string())?;
    let raw_file = fs::read(&vault_path).map_err(|e|e.to_string())?;
    if raw_file.len() < 44 {
        return Err("金库文件格式非法".to_string());
    }
    let mut vault_salt = [0u8;16];
    vault_salt.copy_from_slice(&raw_file[0..16]);

    set_active_master_key(&pwd_buf, &key_file_data, &vault_salt).map_err(|e| e.to_string())?;

    let mut salt_guard = salt_state.0.lock().unwrap();
    *salt_guard = Some(vault_salt);

    pwd_buf.zeroize();
    Ok(())
}

#[tauri::command]
async fn set_vault_storage_path(path: String) -> Result<(), String> {
    storage::set_vault_storage_path(&path).map_err(|e| e.to_string())
}

#[tauri::command]
async fn open_vault_file_picker(app: AppHandle) -> Result<String, String> {
    let file_opt: Option<FilePath> = app
        .dialog()
        .file()
        .add_filter("金库文件", &["dat", "bin", "vault"])
        .blocking_pick_file();

    match file_opt {
        Some(file_path) => {
            let path_ref = file_path.as_path();
            let full_path = path_ref.ok_or("无法获取文件路径".to_string())?.to_string_lossy().to_string();
            storage::set_vault_storage_path(&full_path).map_err(|e| e.to_string())?;
            Ok(full_path)
        }
        None => Err("未选择文件".to_string()),
    }
}

#[tauri::command]
async fn save_vault_file_picker(app: AppHandle) -> Result<String, String> {
    let save_opt: Option<FilePath> = app
        .dialog()
        .file()
        .add_filter("金库文件", &["dat", "bin", "vault"])
        .set_file_name("passwords.vault")
        .blocking_save_file();

    match save_opt {
        Some(save_path) => {
            let path = save_path.as_path().ok_or("无法获取保存路径".to_string())?;
            let full_path = path.to_string_lossy().to_string();
            storage::set_vault_storage_path(&full_path).map_err(|e| e.to_string())?;
            Ok(full_path)
        }
        None => Err("用户取消保存".to_string()),
    }
}

#[tauri::command]
async fn load_vault_store() -> Result<Vec<EntryMetaPreview>, String> {
    let master_key = match get_active_master_key() {
        Ok(key) => key,
        Err(_) => return Err("金库未解锁".to_string()),
    };
    let vault_path = storage::get_vault_storage_path().map_err(|e| e.to_string())?;
    let preview = storage::load_vault_store(&master_key, vault_path.to_str().unwrap_or_default())
        .map_err(|e| e.to_string())?;
    Ok(preview)
}

#[tauri::command]
async fn load_password_leaks() -> Result<Vec<PasswordLeakCheckResult>, String> {
    let master_key = match get_active_master_key() {
        Ok(key) => key,
        Err(_) => return Err("金库未解锁".to_string()),
    };
    let vault_path = storage::get_vault_storage_path().map_err(|e| e.to_string())?;
    let path_str = vault_path.to_str().ok_or("无效金库路径")?;
    let root = storage::load_or_create_store(&master_key, path_str)
        .map_err(|e| e.to_string())?;

    let results = check_all_password_leaks(&root, &master_key.inner, 4).await;
    Ok(results)
}

#[tauri::command]
async fn create_password_entry(account: String, pwd: String, url: Option<String>, note: Option<String>, salt_state: State<'_, VaultSaltState>) -> Result<(), String> {
    let salt_guard = salt_state.0.lock().unwrap();
    let vault_salt = salt_guard.as_ref().ok_or("未设置盐".to_string())?;
    
    let master_key = match get_active_master_key() {
        Ok(key) => key,
        Err(_) => return Err("金库未解锁".to_string()),
    };
    let vault_path = storage::get_vault_storage_path().map_err(|e| e.to_string())?;
    let mut root = storage::load_or_create_store(&master_key, vault_path.to_str().unwrap_or_default())
        .map_err(|e| e.to_string())?;
    storage::create_new_entry(
        &mut root,
        master_key.inner.as_slice(),
        account.as_str(),
        pwd.as_str(),
        url.as_deref(),
        note.as_deref(),
    )
    .map_err(|e| e.to_string())?;
    storage::save_vault_store(&root, &master_key, vault_salt, vault_path.to_str().unwrap_or_default()).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
async fn get_decrypted_entry(entry_id: String) -> Result<DecryptedEntry, String> {
    let master_key = match get_active_master_key() {
        Ok(key) => key,
        Err(_) => return Err("金库未解锁".to_string()),
    };
    let vault_path = storage::get_vault_storage_path().map_err(|e| e.to_string())?;
    let root = storage::load_or_create_store(&master_key, vault_path.to_str().unwrap_or_default())
        .map_err(|e| e.to_string())?;
    storage::get_entry_by_id(&root, master_key.inner.as_slice(), entry_id.as_str()).map_err(|e| e.to_string())
}

#[tauri::command]
async fn update_password_entry(
    entry_id: String,
    new_account: Option<String>,
    new_pwd: Option<String>,
    new_url: Option<String>,
    new_note: Option<String>,
    salt_state: State<'_, VaultSaltState>,
) -> Result<(), String> {
    let salt_guard = salt_state.0.lock().unwrap();
    let vault_salt = salt_guard.as_ref().ok_or("未设置盐".to_string())?;
    
    let master_key = match get_active_master_key() {
        Ok(key) => key,
        Err(_) => return Err("金库未解锁".to_string()),
    };
    let vault_path = storage::get_vault_storage_path().map_err(|e| e.to_string())?;
    let mut root = storage::load_or_create_store(&master_key, vault_path.to_str().unwrap_or_default())
        .map_err(|e| e.to_string())?;
    storage::update_entry(
        &mut root,
        master_key.inner.as_slice(),
        entry_id.as_str(),
        new_account.as_deref(),
        new_pwd.as_deref(),
        new_url.as_deref(),
        new_note.as_deref(),
    )
    .map_err(|e| e.to_string())?;
    storage::save_vault_store(&root, &master_key, vault_salt, vault_path.to_str().unwrap_or_default()).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
async fn delete_password_entry(entry_id: String, salt_state: State<'_, VaultSaltState>) -> Result<(), String> {
    let salt_guard = salt_state.0.lock().unwrap();
    let vault_salt = salt_guard.as_ref().ok_or("未设置盐".to_string())?;
    
    let master_key = match get_active_master_key() {
        Ok(key) => key,
        Err(_) => return Err("金库未解锁".to_string()),
    };
    let vault_path = storage::get_vault_storage_path().map_err(|e| e.to_string())?;
    let mut root = storage::load_or_create_store(&master_key, vault_path.to_str().unwrap_or_default())
        .map_err(|e| e.to_string())?;
    storage::delete_entry(&mut root, entry_id.as_str()).map_err(|e| e.to_string())?;
    storage::save_vault_store(&root, &master_key, vault_salt, vault_path.to_str().unwrap_or_default()).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
async fn save_vault_store(salt_state: State<'_, VaultSaltState>) -> Result<(), String> {
    let salt_guard = salt_state.0.lock().unwrap();
    let vault_salt = salt_guard.as_ref().ok_or("未设置盐".to_string())?;
    
    let master_key = match get_active_master_key() {
        Ok(key) => key,
        Err(_) => return Err("金库未解锁".to_string()),
    };
    let vault_path = storage::get_vault_storage_path().map_err(|e| e.to_string())?;
    let root = storage::load_or_create_store(&master_key, vault_path.to_str().unwrap_or_default())
        .map_err(|e| e.to_string())?;
    storage::save_vault_store(&root, &master_key, vault_salt, vault_path.to_str().unwrap_or_default()).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn logout_vault(salt_state: State<'_, VaultSaltState>) {
    clear_active_master_key();
    // 登出清空缓存盐
    let mut guard = salt_state.0.lock().unwrap();
    guard.take();
}

#[tauri::command]
fn set_paste_pwd(entry_id: String) -> Result<(), String> {
    let master_key = match get_active_master_key() {
        Ok(key) => key,
        Err(_) => return Err("金库未解锁".to_string()),
    };
    let vault_path = storage::get_vault_storage_path().map_err(|e| e.to_string())?;
    let path_str = vault_path.to_str().ok_or("无效金库路径")?;
    let root = storage::load_or_create_store(&master_key, path_str)
        .map_err(|e| e.to_string())?;
    let current_entry = storage::get_entry_by_id(&root, master_key.inner.as_slice(), entry_id.as_str()).map_err(|e| e.to_string())?;
    let pwd = current_entry.password;
    utils::set_need_paste_pwd(pwd).map_err(|e| e.to_string())?;

    Ok(())
}

// ── 2FA 命令 ──────────────────────────────────────────────────

#[tauri::command]
async fn compute_totp_code(entry_id: String) -> Result<(String, u64), String> {
    let master_key = match get_active_master_key() {
        Ok(key) => key,
        Err(_) => return Err("金库未解锁".to_string()),
    };
    let vault_path = storage::get_vault_storage_path().map_err(|e| e.to_string())?;
    let root = storage::load_or_create_store(&master_key, vault_path.to_str().unwrap_or_default())
        .map_err(|e| e.to_string())?;
    twofa::get_totp_for_entry(&root, master_key.inner.as_slice(), entry_id.as_str()).map_err(|e| e.to_string())
}

#[tauri::command]
async fn create_two_fa_entry(
    issuer: String,
    account: String,
    secret: String,
    salt_state: State<'_, VaultSaltState>,
) -> Result<(), String> {
    let salt_guard = salt_state.0.lock().unwrap();
    let vault_salt = salt_guard.as_ref().ok_or("未设置盐".to_string())?;

    let master_key = match get_active_master_key() {
        Ok(key) => key,
        Err(_) => return Err("金库未解锁".to_string()),
    };
    let vault_path = storage::get_vault_storage_path().map_err(|e| e.to_string())?;
    let mut root = storage::load_or_create_store(&master_key, vault_path.to_str().unwrap_or_default())
        .map_err(|e| e.to_string())?;
    twofa::create_two_fa_entry(
        &mut root,
        master_key.inner.as_slice(),
        issuer.as_str(),
        account.as_str(),
        secret.as_str(),
    )
    .map_err(|e| e.to_string())?;
    storage::save_vault_store(&root, &master_key, vault_salt, vault_path.to_str().unwrap_or_default()).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
async fn load_two_fa_store() -> Result<Vec<TwoFAEntryPreview>, String> {
    let master_key = match get_active_master_key() {
        Ok(key) => key,
        Err(_) => return Err("金库未解锁".to_string()),
    };
    let vault_path = storage::get_vault_storage_path().map_err(|e| e.to_string())?;
    let root = storage::load_or_create_store(&master_key, vault_path.to_str().unwrap_or_default())
        .map_err(|e| e.to_string())?;
    let preview = twofa::list_two_fa_entries(&root, master_key.inner.as_slice())
        .map_err(|e| e.to_string())?;
    Ok(preview)
}

#[tauri::command]
async fn get_decrypted_two_fa_entry(entry_id: String) -> Result<DecryptedTwoFAEntry, String> {
    let master_key = match get_active_master_key() {
        Ok(key) => key,
        Err(_) => return Err("金库未解锁".to_string()),
    };
    let vault_path = storage::get_vault_storage_path().map_err(|e| e.to_string())?;
    let root = storage::load_or_create_store(&master_key, vault_path.to_str().unwrap_or_default())
        .map_err(|e| e.to_string())?;
    twofa::get_two_fa_entry_by_id(&root, master_key.inner.as_slice(), entry_id.as_str()).map_err(|e| e.to_string())
}

#[tauri::command]
async fn update_two_fa_entry(
    entry_id: String,
    new_issuer: Option<String>,
    new_account: Option<String>,
    new_secret: Option<String>,
    salt_state: State<'_, VaultSaltState>,
) -> Result<(), String> {
    let salt_guard = salt_state.0.lock().unwrap();
    let vault_salt = salt_guard.as_ref().ok_or("未设置盐".to_string())?;

    let master_key = match get_active_master_key() {
        Ok(key) => key,
        Err(_) => return Err("金库未解锁".to_string()),
    };
    let vault_path = storage::get_vault_storage_path().map_err(|e| e.to_string())?;
    let mut root = storage::load_or_create_store(&master_key, vault_path.to_str().unwrap_or_default())
        .map_err(|e| e.to_string())?;
    twofa::update_two_fa_entry(
        &mut root,
        master_key.inner.as_slice(),
        entry_id.as_str(),
        new_issuer.as_deref(),
        new_account.as_deref(),
        new_secret.as_deref(),
    )
    .map_err(|e| e.to_string())?;
    storage::save_vault_store(&root, &master_key, vault_salt, vault_path.to_str().unwrap_or_default()).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
async fn delete_two_fa_entry(entry_id: String, salt_state: State<'_, VaultSaltState>) -> Result<(), String> {
    let salt_guard = salt_state.0.lock().unwrap();
    let vault_salt = salt_guard.as_ref().ok_or("未设置盐".to_string())?;

    let master_key = match get_active_master_key() {
        Ok(key) => key,
        Err(_) => return Err("金库未解锁".to_string()),
    };
    let vault_path = storage::get_vault_storage_path().map_err(|e| e.to_string())?;
    let mut root = storage::load_or_create_store(&master_key, vault_path.to_str().unwrap_or_default())
        .map_err(|e| e.to_string())?;
    twofa::delete_two_fa_entry(&mut root, entry_id.as_str()).map_err(|e| e.to_string())?;
    storage::save_vault_store(&root, &master_key, vault_salt, vault_path.to_str().unwrap_or_default()).map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    #[cfg(unix)]
    unsafe {
        libc::prctl(libc::PR_SET_DUMPABLE, 0);
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(KeyFilePathState(Mutex::new(None)))
        .manage(VaultSaltState(Mutex::new(None)))
        .setup(|app| {
            let app_handle = app.handle().clone();
            #[cfg(desktop)]
            {
                app.handle().plugin(
                    tauri_plugin_global_shortcut::Builder::new()
                        .with_shortcut(Shortcut::new(Some(Modifiers::CONTROL | Modifiers::ALT), Code::KeyV))?
                        .with_shortcut(Shortcut::new(Some(Modifiers::CONTROL | Modifiers::ALT), Code::KeyS))?
                        .with_handler(move |app, shortcut, event| {
                            if event.state != ShortcutState::Pressed {
                                return;
                            }
                            if shortcut.matches(Modifiers::CONTROL | Modifiers::ALT, Code::KeyV) {
                                let _ = app.emit("trigger_password_paste", ());
                            } else if shortcut.matches(Modifiers::CONTROL | Modifiers::ALT, Code::KeyS) {
                                let _ = app.emit("trigger_qr_scan", ());
                            }
                        })
                        .build()
                )?;

                app_handle.once("trigger_password_paste", |_| {
                    std::thread::spawn(move || {
                        let mut pwd = match get_need_paste_pwd() {
                            Ok(s) => s,
                            Err(_) => return,
                        };
                        if pwd.trim().is_empty() {
                            pwd.zeroize();
                            return;
                        }
                        let mut enigo = match Enigo::new(&Settings::default()) {
                            Ok(e) => e,
                            Err(_) => {
                                pwd.zeroize();
                                clear_need_paste_pwd();
                                return;
                            }
                        };
                        if enigo.text(&pwd).is_err() {
                            pwd.zeroize();
                            clear_need_paste_pwd();
                            return;
                        }
                        pwd.zeroize();
                        clear_need_paste_pwd();
                        drop(enigo)
                    });
                });
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            open_key_file_picker,
            clear_stored_key_path,
            register_vault,
            unlock_vault,
            set_vault_storage_path,
            open_vault_file_picker,
            save_vault_file_picker,
            load_vault_store,
            load_password_leaks,
            generate_strong_password,
            create_password_entry,
            get_decrypted_entry,
            update_password_entry,
            delete_password_entry,
            save_vault_store,
            clear_active_master_key,
            logout_vault,
            set_paste_pwd,
            create_two_fa_entry,
            load_two_fa_store,
            get_decrypted_two_fa_entry,
            update_two_fa_entry,
            delete_two_fa_entry,
            compute_totp_code,
            scan_qr_from_screenshot,
            scan_qr_from_image
        ])
        .run(tauri::generate_context!())
        .expect("Tauri应用启动失败");
}