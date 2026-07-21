mod utils;

use std::fs;
use std::sync::Mutex;
use tauri::{AppHandle, State};
use tauri_plugin_dialog::{DialogExt, FilePath};
use zeroize::{Zeroize};

use utils::{
    ACTIVE_VAULT_KEY, VaultMasterKey, VaultError,
    set_active_master_key, get_active_master_key, clear_active_master_key
};

struct KeyFilePathState(Mutex<Option<String>>);

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
) -> Result<(), String> {
    let mut pwd_buf = user_pwd.into_bytes();

    let mut new_key = [0u8; 32];
    getrandom::fill(&mut new_key).map_err(|e| e.to_string())?;

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

    set_active_master_key(&pwd_buf, &new_key).map_err(|e| e.to_string())?;

    pwd_buf.zeroize();
    new_key.zeroize();
    Ok(())
}

#[tauri::command]
async fn unlock_vault(
    user_pwd: String,
    state: State<'_, KeyFilePathState>
) -> Result<(), String> {
    let mut pwd_buf = user_pwd.into_bytes();
    let path_guard = state.0.lock().unwrap();
    let key_path = path_guard
        .as_ref()
        .ok_or(VaultError::NoKeyFile.to_string())?;
    let key_file_data = fs::read(key_path).map_err(|e| e.to_string())?;

    set_active_master_key(&pwd_buf, &key_file_data).map_err(|e| e.to_string())?;

    pwd_buf.zeroize();
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
        .invoke_handler(tauri::generate_handler![
            open_key_file_picker,
            clear_stored_key_path,
            register_vault,
            unlock_vault,
        ])
        .run(tauri::generate_context!())
        .expect("Tauri应用启动失败");
}