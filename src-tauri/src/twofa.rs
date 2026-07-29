use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha1::Sha1;
use uuid::Uuid;
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
