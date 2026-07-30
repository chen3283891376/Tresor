use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha1::Sha1;
use tauri_plugin_dialog::DialogExt;
use uuid::Uuid;
use xcap::Monitor;
use zeroize::Zeroize;

use crate::storage;
use crate::utils::{unlock_memory, VaultError};

/// 单条2FA记录的加密容器。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedTwoFAEntry {
    pub entry_id: String,
    pub encrypted_issuer: storage::CipherHexWrap,
    pub encrypted_account: storage::CipherHexWrap,
    pub encrypted_secret: storage::CipherHexWrap,
    pub created_at: u64,
    pub updated_at: u64,
}

/// 2FA列表预览。
#[derive(Debug, Serialize)]
pub struct TwoFAEntryPreview {
    pub entry_id: String,
    pub issuer: String,
    pub account: String,
    pub created_at: u64,
}

/// 完整解密后的2FA记录。
#[derive(Debug, Serialize)]
pub struct DecryptedTwoFAEntry {
    pub entry_id: String,
    pub issuer: String,
    pub account: String,
    pub secret_base32: String,
    pub created_at: u64,
    pub updated_at: u64,
}

/// 创建一条2FA记录并持久化。
pub fn create_two_fa_entry(
    root: &mut storage::VaultStoreRoot,
    master_key: &[u8],
    issuer: &str,
    account: &str,
    secret: &str,
) -> Result<(), VaultError> {
    let entry_id = Uuid::new_v4().to_string();
    let mut sub_key = storage::derive_entry_sub_key(master_key, &entry_id)?;

    let encrypted_issuer = storage::encrypt_single_plaintext(issuer.as_bytes(), &sub_key)?;
    let encrypted_account = storage::encrypt_single_plaintext(account.as_bytes(), &sub_key)?;
    let encrypted_secret = storage::encrypt_single_plaintext(secret.as_bytes(), &sub_key)?;

    let now = storage::unix_timestamp();
    root.two_fa_entries.push(EncryptedTwoFAEntry {
        entry_id: entry_id.clone(),
        encrypted_issuer,
        encrypted_account,
        encrypted_secret,
        created_at: now,
        updated_at: now,
    });
    root.meta.last_modified = now;

    unlock_memory(&mut sub_key);
    sub_key.zeroize();

    Ok(())
}

/// 返回2FA列表预览（解密 issuer + account）。
pub fn list_two_fa_entries(root: &storage::VaultStoreRoot, master_key: &[u8]) -> Result<Vec<TwoFAEntryPreview>, VaultError> {
    let mut list = Vec::new();
    for entry in &root.two_fa_entries {
        let mut sub_key = storage::derive_entry_sub_key(master_key, &entry.entry_id)?;

        let mut issuer_buf = storage::decrypt_single_cipher(&entry.encrypted_issuer, &sub_key)?;
        let mut account_buf = storage::decrypt_single_cipher(&entry.encrypted_account, &sub_key)?;
        let issuer = storage::utf8_string_from_bytes(&mut issuer_buf)?;
        let account = storage::utf8_string_from_bytes(&mut account_buf)?;

        unlock_memory(&mut sub_key);
        sub_key.zeroize();

        list.push(TwoFAEntryPreview {
            entry_id: entry.entry_id.clone(),
            issuer,
            account,
            created_at: entry.created_at,
        });
    }
    Ok(list)
}

/// 根据 EntryID 查询并解密一条完整2FA记录。
pub fn get_two_fa_entry_by_id(
    root: &storage::VaultStoreRoot,
    master_key: &[u8],
    entry_id: &str,
) -> Result<DecryptedTwoFAEntry, VaultError> {
    let entry = root
        .two_fa_entries
        .iter()
        .find(|item| item.entry_id == entry_id)
        .ok_or_else(|| {
            VaultError::FileIo(std::io::Error::new(std::io::ErrorKind::NotFound, "指定2FA记录不存在"))
        })?;

    let mut sub_key = storage::derive_entry_sub_key(master_key, entry_id)?;

    let mut issuer_buf = storage::decrypt_single_cipher(&entry.encrypted_issuer, &sub_key)?;
    let mut account_buf = storage::decrypt_single_cipher(&entry.encrypted_account, &sub_key)?;
    let mut secret_buf = storage::decrypt_single_cipher(&entry.encrypted_secret, &sub_key)?;

    let issuer = storage::utf8_string_from_bytes(&mut issuer_buf)?;
    let account = storage::utf8_string_from_bytes(&mut account_buf)?;
    let secret = storage::utf8_string_from_bytes(&mut secret_buf)?;

    unlock_memory(&mut sub_key);
    sub_key.zeroize();

    Ok(DecryptedTwoFAEntry {
        entry_id: entry.entry_id.clone(),
        issuer,
        account,
        secret_base32: secret,
        created_at: entry.created_at,
        updated_at: entry.updated_at,
    })
}

/// 更新指定2FA记录。
pub fn update_two_fa_entry(
    root: &mut storage::VaultStoreRoot,
    master_key: &[u8],
    entry_id: &str,
    new_issuer: Option<&str>,
    new_account: Option<&str>,
    new_secret: Option<&str>,
) -> Result<(), VaultError> {
    let entry = root
        .two_fa_entries
        .iter_mut()
        .find(|item| item.entry_id == entry_id)
        .ok_or_else(|| {
            VaultError::FileIo(std::io::Error::new(std::io::ErrorKind::NotFound, "指定2FA记录不存在"))
        })?;

    let mut sub_key = storage::derive_entry_sub_key(master_key, entry_id)?;
    let now = storage::unix_timestamp();

    if let Some(issuer) = new_issuer {
        entry.encrypted_issuer = storage::encrypt_single_plaintext(issuer.as_bytes(), &sub_key)?;
    }
    if let Some(account) = new_account {
        entry.encrypted_account = storage::encrypt_single_plaintext(account.as_bytes(), &sub_key)?;
    }
    if let Some(secret) = new_secret {
        entry.encrypted_secret = storage::encrypt_single_plaintext(secret.as_bytes(), &sub_key)?;
    }

    entry.updated_at = now;
    root.meta.last_modified = now;

    unlock_memory(&mut sub_key);
    sub_key.zeroize();

    Ok(())
}

/// 删除指定2FA记录。
pub fn delete_two_fa_entry(root: &mut storage::VaultStoreRoot, entry_id: &str) -> Result<(), VaultError> {
    let index = root
        .two_fa_entries
        .iter()
        .position(|item| item.entry_id == entry_id)
        .ok_or_else(|| {
            VaultError::FileIo(std::io::Error::new(std::io::ErrorKind::NotFound, "指定2FA记录不存在"))
        })?;
    root.two_fa_entries.remove(index);
    root.meta.last_modified = storage::unix_timestamp();
    Ok(())
}

const BASE32_ALPHABET: &[u8; 32] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";

fn base32_decode(encoded: &str) -> Result<Vec<u8>, VaultError> {
    let cleaned: String = encoded.trim_end_matches('=').to_uppercase();
    let mut bits = Vec::with_capacity(cleaned.len() * 5);
    for ch in cleaned.bytes() {
        let val = BASE32_ALPHABET.iter().position(|&c| c == ch);
        match val {
            Some(v) => {
                for i in (0..5).rev() {
                    bits.push(((v >> i) & 1) as u8);
                }
            }
            None => continue,
        }
    }
    let mut bytes = Vec::with_capacity(bits.len() / 8);
    for chunk in bits.chunks(8) {
        if chunk.len() < 8 {
            break;
        }
        let mut byte = 0u8;
        for &b in chunk {
            byte = (byte << 1) | b;
        }
        bytes.push(byte);
    }
    Ok(bytes)
}

type HmacSha1 = Hmac<Sha1>;

fn compute_totp_code(secret_base32: &str, timestamp_secs: u64, digits: u32, period: u64) -> Result<String, VaultError> {
    let secret = base32_decode(secret_base32)?;
    if secret.is_empty() {
        return Err(VaultError::FileIo(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "无法解码Base32密钥",
        )));
    }

    let time_step = timestamp_secs / period;
    let time_bytes = time_step.to_be_bytes();

    let mut mac = HmacSha1::new_from_slice(&secret)
        .map_err(|e| VaultError::FileIo(std::io::Error::new(std::io::ErrorKind::InvalidInput, e)))?;
    mac.update(&time_bytes);
    let result = mac.finalize().into_bytes();

    let offset = (result[19] & 0xf) as usize;
    let code = ((result[offset] & 0x7f) as u32) << 24
        | (result[offset + 1] as u32) << 16
        | (result[offset + 2] as u32) << 8
        | (result[offset + 3] as u32);

    let totp = code % 10u32.pow(digits);
    Ok(format!("{:0width$}", totp, width = digits as usize))
}

/// 计算指定2FA条目的当前TOTP验证码。
pub fn get_totp_for_entry(root: &storage::VaultStoreRoot, master_key: &[u8], entry_id: &str) -> Result<(String, u64), VaultError> {
    let entry = root
        .two_fa_entries
        .iter()
        .find(|item| item.entry_id == entry_id)
        .ok_or_else(|| {
            VaultError::FileIo(std::io::Error::new(std::io::ErrorKind::NotFound, "指定2FA记录不存在"))
        })?;

    let mut sub_key = storage::derive_entry_sub_key(master_key, entry_id)?;
    let mut secret_buf = storage::decrypt_single_cipher(&entry.encrypted_secret, &sub_key)?;
    let secret = storage::utf8_string_from_bytes(&mut secret_buf)?;

    unlock_memory(&mut sub_key);
    sub_key.zeroize();

    let now = storage::unix_timestamp();

    let period = 30u64;
    let time_remaining = period - (now % period);
    let code = compute_totp_code(&secret, now, 6, period)?;

    Ok((code, time_remaining))
}

// ── 二维码扫描 ──────────────────────────────────────────────────

/// 二维码扫描结果
#[derive(Debug, Serialize)]
pub struct QrScanResult {
    pub secret: String,
    pub issuer: String,
    pub account: String,
    pub algorithm: Option<String>,
    pub digits: Option<u32>,
    pub period: Option<u32>,
}

/// 从灰度图像解码二维码，核心解码逻辑
fn decode_qr_from_gray(gray: &image::GrayImage) -> Result<String, String> {
    let (w, h) = gray.dimensions();
    let gray_data: &[u8] = gray.as_ref();
    let wu = w as usize;
    let mut prepared = rqrr::PreparedImage::prepare_from_greyscale(wu, h as usize, |x, y| {
        gray_data[y * wu + x]
    });
    let grids = prepared.detect_grids();

    if grids.is_empty() {
        return Err("未检测到二维码".to_string());
    }

    for grid in &grids {
        if let Ok((_, content)) = grid.decode() {
            return Ok(content);
        }
    }

    Err("二维码解码失败，无法读取内容".to_string())
}

/// 从 RGBA 图像解码二维码
fn decode_qr_from_rgba(img: &image::RgbaImage) -> Result<String, String> {
    let gray = image::imageops::grayscale(img);
    decode_qr_from_gray(&gray)
}

/// 从图片文件解码二维码，返回原始文本内容
fn decode_qr_from_file(path: &str) -> Result<String, String> {
    let img = image::open(path).map_err(|e| format!("无法读取图片: {}", e))?;
    let gray = img.to_luma8();
    decode_qr_from_gray(&gray)
}

/// 截屏并解码二维码，返回原始文本内容
fn decode_qr_from_screen() -> Result<String, String> {
    let monitors = Monitor::all().map_err(|e| format!("无法获取屏幕信息: {}", e))?;
    let monitor = monitors.first().ok_or("未检测到显示器")?;
    let img = monitor
        .capture_image()
        .map_err(|e| format!("截图失败: {}", e))?;
    decode_qr_from_rgba(&img)
}

/// 解析 otpauth:// URI，提取密钥和账户信息
fn parse_otpauth_uri(uri_str: &str) -> Result<QrScanResult, String> {
    let url = url::Url::parse(uri_str).map_err(|e| format!("无效URI: {}", e))?;

    if url.scheme() != "otpauth" {
        return Err("不是有效的 otpauth:// URI".to_string());
    }

    let host = url.host_str().ok_or("缺少协议类型")?;
    if host != "totp" && host != "hotp" {
        return Err(format!("不支持的2FA类型: {}", host));
    }

    let path = url.path().trim_start_matches('/');

    let query: std::collections::HashMap<_, _> = url.query_pairs().into_owned().collect();

    let secret = query
        .get("secret")
        .cloned()
        .ok_or("缺少 secret 参数")?;

    let issuer_param = query.get("issuer").cloned();

    let algorithm = query.get("algorithm").cloned();
    let digits = query
        .get("digits")
        .and_then(|v| v.parse::<u32>().ok());
    let period = query
        .get("period")
        .and_then(|v| v.parse::<u32>().ok());

    let (issuer, account) = if let Some(colon_idx) = path.find(':') {
        let i = &path[..colon_idx];
        let a = &path[colon_idx + 1..];
        (Some(i.to_string()), a.to_string())
    } else {
        (None, path.to_string())
    };

    let issuer = issuer.or(issuer_param).unwrap_or_default();

    Ok(QrScanResult {
        secret,
        issuer,
        account,
        algorithm,
        digits,
        period,
    })
}

/// Tauri 命令：截取当前屏幕 → 解码 QR → 解析 otpauth URI → 返回结果
#[tauri::command]
pub async fn scan_qr_from_screenshot() -> Result<QrScanResult, String> {
    let raw_text = decode_qr_from_screen()?;
    parse_otpauth_uri(&raw_text)
}

/// Tauri 命令：弹出文件选择器选取二维码图片 → 解码 → 解析 otpauth URI → 返回结果
#[tauri::command]
pub async fn scan_qr_from_image(app: tauri::AppHandle) -> Result<QrScanResult, String> {
    let file_opt: Option<tauri_plugin_dialog::FilePath> = app
        .dialog()
        .file()
        .add_filter("二维码图片", &["png", "jpg", "jpeg"])
        .blocking_pick_file();

    let file_path = match file_opt {
        Some(fp) => fp,
        None => return Err("未选择图片".to_string()),
    };

    let full_path = file_path
        .as_path()
        .ok_or("无法获取文件路径".to_string())?
        .to_string_lossy()
        .to_string();

    let raw_text = decode_qr_from_file(&full_path)?;
    parse_otpauth_uri(&raw_text)
}
