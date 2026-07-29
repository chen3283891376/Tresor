use aes_gcm::aead::Aead;
use aes_gcm::{Aes256Gcm, KeyInit, Nonce};
use hex;
use hkdf::Hkdf;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use hex::ToHex;
use reqwest::Client;
use sha1::{Digest, Sha1};
use uuid::Uuid;
use zeroize::Zeroize;

use crate::utils::{lock_memory, unlock_memory, VaultError, VaultMasterKey};

static VAULT_STORAGE_PATH: Lazy<Mutex<Option<PathBuf>>> = Lazy::new(|| Mutex::new(None));

const SCHEMA_VERSION: u32 = 1;

/// 全局缓存的金库文件路径。所有存储命令都从这里读取/写入。
pub fn set_vault_storage_path(path: &str) -> Result<(), VaultError> {
    let mut guard = VAULT_STORAGE_PATH.lock().unwrap();
    *guard = Some(PathBuf::from(path));
    Ok(())
}

/// 获取当前配置的金库文件路径。
pub fn get_vault_storage_path() -> Result<PathBuf, VaultError> {
    let guard = VAULT_STORAGE_PATH.lock().unwrap();
    guard.clone().ok_or_else(|| {
        VaultError::FileIo(std::io::Error::new(std::io::ErrorKind::NotFound, "金库存储路径未设置"))
    })
}

/// 解密后的金库顶层结构体：主密钥只用于加密整个 JSON 容器。
#[derive(Debug, Serialize, Deserialize)]
pub struct VaultStoreRoot {
    pub meta: VaultMeta,
    pub entries: Vec<EncryptedPasswordEntry>,
    #[serde(default)]
    pub two_fa_entries: Vec<EncryptedTwoFAEntry>,
}

/// 单条2FA记录的加密容器。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedTwoFAEntry {
    pub entry_id: String,
    pub encrypted_issuer: CipherHexWrap,
    pub encrypted_account: CipherHexWrap,
    pub encrypted_secret: CipherHexWrap,
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

/// 金库元数据：公开明文，用于版本和时间戳校验。
#[derive(Debug, Serialize, Deserialize)]
pub struct VaultMeta {
    pub schema_version: u32,
    pub created_at: u64,
    pub last_modified: u64,
    pub argon2_salt: [u8; 16],
}

/// 单条密码记录的加密容器：不再持久化子密钥，只持久化 EntryID。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedPasswordEntry {
    pub entry_id: String,
    pub encrypted_account: CipherHexWrap,
    pub encrypted_password: CipherHexWrap,
    pub encrypted_url: Option<CipherHexWrap>,
    pub encrypted_note: Option<CipherHexWrap>,
    pub created_at: u64,
    pub updated_at: u64,
}

/// 通用密文封装，十六进制序列化存储。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CipherHexWrap {
    pub nonce_hex: String,
    pub cipher_hex: String,
    pub tag_hex: String,
}

/// 供前端预览列表使用的无敏感字段结构。
#[derive(Debug, Serialize)]
pub struct EntryMetaPreview {
    pub entry_id: String,
    pub url: Option<String>,
    pub created_at: u64,
}

/// 完整解密后的记录对象。
#[derive(Debug, Serialize)]
pub struct DecryptedEntry {
    pub entry_id: String,
    pub account: String,
    pub password: String,
    pub url: Option<String>,
    pub note: Option<String>,
    pub created_at: u64,
    pub updated_at: u64,
}

/// 初始化一个空金库。
pub fn init_empty_vault(salt: &[u8; 16]) -> VaultStoreRoot {
    let now = unix_timestamp();
    VaultStoreRoot {
        meta: VaultMeta {
            schema_version: SCHEMA_VERSION,
            created_at: now,
            last_modified: now,
            argon2_salt: *salt,
        },
        entries: Vec::new(),
        two_fa_entries: Vec::new(),
    }
}

/// 通过主密钥 + EntryID 实时派生本条 AES-256 子密钥。
pub fn derive_entry_sub_key(master_key: &[u8], entry_id: &str) -> Result<[u8; 32], VaultError> {
    let mut sub_key = [0u8; 32];
    lock_memory(&mut sub_key)?;

    let hkdf: Hkdf<Sha256> = Hkdf::new(None, master_key);
    let info = entry_id.as_bytes();
    hkdf.expand(info, &mut sub_key)?;

    Ok(sub_key)
}

/// 将整个金库 JSON 结构体用主密钥加密并原子写入磁盘。
pub fn encrypt_store_to_file(root: &VaultStoreRoot, master_key: &VaultMasterKey, salt: &[u8;16], path: &str) -> Result<(), VaultError> {
    let mut plaintext = serde_json::to_vec(root).map_err(map_json_error)?;
    lock_memory(&mut plaintext)?;

    let mut nonce = [0u8; 12];
    getrandom::fill(&mut nonce).map_err(map_rand_error)?;

    let mut master_key_bytes = [0u8; 32];
    master_key_bytes.copy_from_slice(master_key.inner.as_slice());
    let cipher = Aes256Gcm::new((&master_key_bytes).into());
    let nonce_obj = Nonce::try_from(&nonce[..]).unwrap();
    let mut encrypted_blob = cipher
        .encrypt(&nonce_obj, plaintext.as_ref())
        .map_err(map_aead_error)?;

    let tag_bytes = encrypted_blob.split_off(plaintext.len());
    // 格式：[16 salt][12 nonce][16 tag][cipher]
    let mut output = Vec::with_capacity(16 + 12 + 16 + encrypted_blob.len());
    output.extend_from_slice(salt);
    output.extend_from_slice(&nonce);
    output.extend_from_slice(&tag_bytes);
    output.extend_from_slice(&encrypted_blob);

    lock_memory(&mut output)?;

    write_atomic(&output, path)?;

    unlock_memory(&mut output);
    output.zeroize();
    unlock_memory(&mut plaintext);
    plaintext.zeroize();
    Ok(())
}

/// 读取并解密金库文件。
pub fn decrypt_store_from_file(master_key: &VaultMasterKey, path: &str) -> Result<VaultStoreRoot, VaultError> {
    let data = fs::read(path)?;
    // 新文件最小长度：16+12+16 = 44
    if data.len() < 44 {
        return Err(VaultError::FileIo(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "金库文件格式不完整",
        )));
    }

    // 读取头部明文盐
    let mut salt = [0u8;16];
    salt.copy_from_slice(&data[0..16]);
    let mut nonce = [0u8; 12];
    nonce.copy_from_slice(&data[16..28]);
    let tag_bytes = &data[28..44];
    let ciphertext_bytes = &data[44..];

    let mut combined = ciphertext_bytes.to_vec();
    combined.extend_from_slice(tag_bytes);

    let mut master_key_bytes = [0u8; 32];
    master_key_bytes.copy_from_slice(master_key.inner.as_slice());
    let cipher = Aes256Gcm::new((&master_key_bytes).into());
    let nonce_obj = Nonce::try_from(&nonce[..]).unwrap();
    let mut plaintext = cipher
        .decrypt(&nonce_obj, combined.as_ref())
        .map_err(|_| VaultError::FileIo(std::io::Error::new(std::io::ErrorKind::InvalidData, "金库文件已被篡改或损坏")))?;

    lock_memory(&mut plaintext)?;
    let root = serde_json::from_slice::<VaultStoreRoot>(&plaintext).map_err(map_json_error)?;
    unlock_memory(&mut plaintext);
    plaintext.zeroize();

    Ok(root)
}

/// 加密单条明文字段。
pub fn encrypt_single_plaintext(plain: &[u8], sub_key: &[u8; 32]) -> Result<CipherHexWrap, VaultError> {
    let mut plaintext = plain.to_vec();
    lock_memory(&mut plaintext)?;

    let mut nonce = [0u8; 12];
    getrandom::fill(&mut nonce).map_err(map_rand_error)?;

    let cipher = Aes256Gcm::new(sub_key.into());
    let nonce_obj = Nonce::try_from(&nonce[..]).unwrap();
    let mut encrypted_blob = cipher
        .encrypt(&nonce_obj, plaintext.as_ref())
        .map_err(map_aead_error)?;
    let tag_bytes = encrypted_blob.split_off(plaintext.len());

    let wrap = CipherHexWrap {
        nonce_hex: hex::encode(nonce),
        cipher_hex: hex::encode(encrypted_blob),
        tag_hex: hex::encode(tag_bytes),
    };

    unlock_memory(&mut plaintext);
    plaintext.zeroize();

    Ok(wrap)
}

/// 解密单条字段。
pub fn decrypt_single_cipher(wrap: &CipherHexWrap, sub_key: &[u8; 32]) -> Result<Vec<u8>, VaultError> {
    let nonce_bytes = hex::decode(&wrap.nonce_hex).map_err(map_hex_error)?;
    let cipher_bytes = hex::decode(&wrap.cipher_hex).map_err(map_hex_error)?;
    let tag_bytes = hex::decode(&wrap.tag_hex).map_err(map_hex_error)?;

    if nonce_bytes.len() != 12 {
        return Err(VaultError::FileIo(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "AES-GCM nonce 长度非法",
        )));
    }

    let mut combined = cipher_bytes;
    combined.extend_from_slice(&tag_bytes);

    let cipher = Aes256Gcm::new(sub_key.into());
    let nonce_obj = Nonce::try_from(&nonce_bytes[..]).unwrap();
    let mut plaintext = cipher
        .decrypt(&nonce_obj, combined.as_ref())
        .map_err(|_| VaultError::FileIo(std::io::Error::new(std::io::ErrorKind::InvalidData, "字段密文已被篡改")))?;

    lock_memory(&mut plaintext)?;
    Ok(plaintext)
}

/// 创建一条新记录并持久化到金库。
pub fn create_new_entry(
    root: &mut VaultStoreRoot,
    master_key: &[u8],
    account: &str,
    pwd: &str,
    url: Option<&str>,
    note: Option<&str>,
) -> Result<(), VaultError> {
    let entry_id = Uuid::new_v4().to_string();
    let mut sub_key = derive_entry_sub_key(master_key, &entry_id)?;

    let encrypted_account = encrypt_single_plaintext(account.as_bytes(), &sub_key)?;
    let encrypted_password = encrypt_single_plaintext(pwd.as_bytes(), &sub_key)?;
    let encrypted_url = url
        .map(|value| encrypt_single_plaintext(value.as_bytes(), &sub_key))
        .transpose()?;
    let encrypted_note = note
        .map(|value| encrypt_single_plaintext(value.as_bytes(), &sub_key))
        .transpose()?;

    let now = unix_timestamp();
    root.entries.push(EncryptedPasswordEntry {
        entry_id: entry_id.clone(),
        encrypted_account,
        encrypted_password,
        encrypted_url,
        encrypted_note,
        created_at: now,
        updated_at: now,
    });
    root.meta.last_modified = now;

    unlock_memory(&mut sub_key);
    sub_key.zeroize();

    Ok(())
}

/// 根据 EntryID 查询并解密一条完整记录。
pub fn get_entry_by_id(root: &VaultStoreRoot, master_key: &[u8], entry_id: &str) -> Result<DecryptedEntry, VaultError> {
    let entry = root.entries.iter().find(|item| item.entry_id == entry_id).ok_or_else(|| {
        VaultError::FileIo(std::io::Error::new(std::io::ErrorKind::NotFound, "指定记录不存在"))
    })?;

    let mut sub_key = derive_entry_sub_key(master_key, entry_id)?;
    let mut account_bytes = decrypt_single_cipher(&entry.encrypted_account, &sub_key)?;
    let mut password_bytes = decrypt_single_cipher(&entry.encrypted_password, &sub_key)?;
    let mut url_bytes = match &entry.encrypted_url {
        Some(cipher) => Some(decrypt_single_cipher(cipher, &sub_key)?),
        None => None,
    };
    let mut note_bytes = match &entry.encrypted_note {
        Some(cipher) => Some(decrypt_single_cipher(cipher, &sub_key)?),
        None => None,
    };

    let account = utf8_string_from_bytes(&mut account_bytes)?;
    let password = utf8_string_from_bytes(&mut password_bytes)?;
    let url = url_bytes.as_mut().map(|buf| utf8_string_from_bytes(buf)).transpose()?;
    let note = note_bytes.as_mut().map(|buf| utf8_string_from_bytes(buf)).transpose()?;

    unlock_memory(&mut sub_key);
    sub_key.zeroize();

    Ok(DecryptedEntry {
        entry_id: entry.entry_id.clone(),
        account,
        password,
        url,
        note,
        created_at: entry.created_at,
        updated_at: entry.updated_at,
    })
}

/// 更新指定记录，保留未修改字段的原密文。
pub fn update_entry(
    root: &mut VaultStoreRoot,
    master_key: &[u8],
    entry_id: &str,
    new_account: Option<&str>,
    new_pwd: Option<&str>,
    new_url: Option<&str>,
    new_note: Option<&str>,
) -> Result<(), VaultError> {
    let entry = root.entries.iter_mut().find(|item| item.entry_id == entry_id).ok_or_else(|| {
        VaultError::FileIo(std::io::Error::new(std::io::ErrorKind::NotFound, "指定记录不存在"))
    })?;

    let mut sub_key = derive_entry_sub_key(master_key, entry_id)?;
    let now = unix_timestamp();

    if let Some(account) = new_account {
        entry.encrypted_account = encrypt_single_plaintext(account.as_bytes(), &sub_key)?;
    }
    if let Some(pwd) = new_pwd {
        entry.encrypted_password = encrypt_single_plaintext(pwd.as_bytes(), &sub_key)?;
    }
    if let Some(url) = new_url {
        entry.encrypted_url = Some(encrypt_single_plaintext(url.as_bytes(), &sub_key)?);
    }
    if let Some(note) = new_note {
        entry.encrypted_note = Some(encrypt_single_plaintext(note.as_bytes(), &sub_key)?);
    }

    entry.updated_at = now;
    root.meta.last_modified = now;

    unlock_memory(&mut sub_key);
    sub_key.zeroize();

    Ok(())
}

/// 删除指定记录。
pub fn delete_entry(root: &mut VaultStoreRoot, entry_id: &str) -> Result<(), VaultError> {
    let index = root.entries.iter().position(|item| item.entry_id == entry_id).ok_or_else(|| {
        VaultError::FileIo(std::io::Error::new(std::io::ErrorKind::NotFound, "指定记录不存在"))
    })?;
    root.entries.remove(index);
    root.meta.last_modified = unix_timestamp();
    Ok(())
}

/// 仅返回列表预览信息，不触发子密钥派生。
pub fn list_all_entry_meta(root: &VaultStoreRoot, master_key: &[u8]) -> Result<Vec<EntryMetaPreview>, VaultError> {
    let mut list = Vec::new();
    for entry in &root.entries {
        let mut url_text: Option<String> = None;
        if let Some(cipher) = &entry.encrypted_url {
            let mut sub_key = derive_entry_sub_key(master_key, &entry.entry_id)?;
            let mut buf = decrypt_single_cipher(cipher, &sub_key)?;
            url_text = Some(utf8_string_from_bytes(&mut buf)?);
            unlock_memory(&mut sub_key);
            sub_key.zeroize();
        }

        list.push(EntryMetaPreview {
            entry_id: entry.entry_id.clone(),
            url: url_text,
            created_at: entry.created_at,
        });
    }
    Ok(list)
}

/// 读取/创建金库根对象。
pub fn load_or_create_store(master_key: &VaultMasterKey, path: &str) -> Result<VaultStoreRoot, VaultError> {
    let path_buf = Path::new(path);
    if !path_buf.exists() {
        let salt = random_salt_array()?;
        return Ok(init_empty_vault(&salt));
    }

    decrypt_store_from_file(master_key, path)
}

/// 读取并解密当前库，返回预览列表。
pub fn load_vault_store(master_key: &VaultMasterKey, path: &str) -> Result<Vec<EntryMetaPreview>, VaultError> {
    let root = load_or_create_store(master_key, path)?;
    Ok(list_all_entry_meta(&root, &master_key.inner)?)
}

/// 手动强制写回当前金库（若不存在则创建空库）。
pub fn save_vault_store(root: &VaultStoreRoot, master_key: &VaultMasterKey, salt: &[u8;16], path: &str) -> Result<(), VaultError> {
    encrypt_store_to_file(root, master_key, salt, path)
}

// ── 2FA 条目 CRUD ──────────────────────────────────────────────

/// 创建一条2FA记录并持久化。
pub fn create_two_fa_entry(
    root: &mut VaultStoreRoot,
    master_key: &[u8],
    issuer: &str,
    account: &str,
    secret: &str,
) -> Result<(), VaultError> {
    let entry_id = Uuid::new_v4().to_string();
    let mut sub_key = derive_entry_sub_key(master_key, &entry_id)?;

    let encrypted_issuer = encrypt_single_plaintext(issuer.as_bytes(), &sub_key)?;
    let encrypted_account = encrypt_single_plaintext(account.as_bytes(), &sub_key)?;
    let encrypted_secret = encrypt_single_plaintext(secret.as_bytes(), &sub_key)?;

    let now = unix_timestamp();
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
pub fn list_two_fa_entries(root: &VaultStoreRoot, master_key: &[u8]) -> Result<Vec<TwoFAEntryPreview>, VaultError> {
    let mut list = Vec::new();
    for entry in &root.two_fa_entries {
        let mut sub_key = derive_entry_sub_key(master_key, &entry.entry_id)?;

        let mut issuer_buf = decrypt_single_cipher(&entry.encrypted_issuer, &sub_key)?;
        let mut account_buf = decrypt_single_cipher(&entry.encrypted_account, &sub_key)?;
        let issuer = utf8_string_from_bytes(&mut issuer_buf)?;
        let account = utf8_string_from_bytes(&mut account_buf)?;

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
    root: &VaultStoreRoot,
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

    let mut sub_key = derive_entry_sub_key(master_key, entry_id)?;

    let mut issuer_buf = decrypt_single_cipher(&entry.encrypted_issuer, &sub_key)?;
    let mut account_buf = decrypt_single_cipher(&entry.encrypted_account, &sub_key)?;
    let mut secret_buf = decrypt_single_cipher(&entry.encrypted_secret, &sub_key)?;

    let issuer = utf8_string_from_bytes(&mut issuer_buf)?;
    let account = utf8_string_from_bytes(&mut account_buf)?;
    let secret = utf8_string_from_bytes(&mut secret_buf)?;

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
    root: &mut VaultStoreRoot,
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

    let mut sub_key = derive_entry_sub_key(master_key, entry_id)?;
    let now = unix_timestamp();

    if let Some(issuer) = new_issuer {
        entry.encrypted_issuer = encrypt_single_plaintext(issuer.as_bytes(), &sub_key)?;
    }
    if let Some(account) = new_account {
        entry.encrypted_account = encrypt_single_plaintext(account.as_bytes(), &sub_key)?;
    }
    if let Some(secret) = new_secret {
        entry.encrypted_secret = encrypt_single_plaintext(secret.as_bytes(), &sub_key)?;
    }

    entry.updated_at = now;
    root.meta.last_modified = now;

    unlock_memory(&mut sub_key);
    sub_key.zeroize();

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

use hmac::{Hmac, Mac};

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
pub fn get_totp_for_entry(root: &VaultStoreRoot, master_key: &[u8], entry_id: &str) -> Result<(String, u64), VaultError> {
    let entry = root
        .two_fa_entries
        .iter()
        .find(|item| item.entry_id == entry_id)
        .ok_or_else(|| {
            VaultError::FileIo(std::io::Error::new(std::io::ErrorKind::NotFound, "指定2FA记录不存在"))
        })?;

    let mut sub_key = derive_entry_sub_key(master_key, entry_id)?;
    let mut secret_buf = decrypt_single_cipher(&entry.encrypted_secret, &sub_key)?;
    let secret = utf8_string_from_bytes(&mut secret_buf)?;

    unlock_memory(&mut sub_key);
    sub_key.zeroize();

    let now = unix_timestamp();

    let period = 30u64;
    let time_remaining = period - (now % period);
    let code = compute_totp_code(&secret, now, 6, period)?;

    Ok((code, time_remaining))
}

/// 删除指定2FA记录。
pub fn delete_two_fa_entry(root: &mut VaultStoreRoot, entry_id: &str) -> Result<(), VaultError> {
    let index = root
        .two_fa_entries
        .iter()
        .position(|item| item.entry_id == entry_id)
        .ok_or_else(|| {
            VaultError::FileIo(std::io::Error::new(std::io::ErrorKind::NotFound, "指定2FA记录不存在"))
        })?;
    root.two_fa_entries.remove(index);
    root.meta.last_modified = unix_timestamp();
    Ok(())
}

fn write_atomic(data: &[u8], path: &str) -> Result<(), VaultError> {
    let path_buf = Path::new(path);
    if let Some(parent) = path_buf.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)?;
        }
    }

    let tmp_path = format!("{}.tmp", path);
    fs::write(&tmp_path, data)?;

    if path_buf.exists() {
        fs::remove_file(path_buf)?;
    }
    fs::rename(&tmp_path, path_buf)?;
    Ok(())
}

fn unix_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn random_salt_array() -> Result<[u8; 16], VaultError> {
    let mut salt = [0u8; 16];
    getrandom::fill(&mut salt).map_err(map_rand_error)?;
    Ok(salt)
}

fn utf8_string_from_bytes(bytes: &mut Vec<u8>) -> Result<String, VaultError> {
    let text = std::str::from_utf8(bytes.as_slice())
        .map_err(map_str_error)?
        .to_string();
    bytes.zeroize();
    Ok(text)
}

fn map_json_error(error: serde_json::Error) -> VaultError {
    VaultError::FileIo(std::io::Error::new(std::io::ErrorKind::InvalidData, error.to_string()))
}

fn map_hex_error(error: hex::FromHexError) -> VaultError {
    VaultError::FileIo(std::io::Error::new(std::io::ErrorKind::InvalidData, error.to_string()))
}

fn map_str_error(error: std::str::Utf8Error) -> VaultError {
    VaultError::FileIo(std::io::Error::new(std::io::ErrorKind::InvalidData, error.to_string()))
}

fn map_aead_error(error: aes_gcm::Error) -> VaultError {
    VaultError::FileIo(std::io::Error::new(std::io::ErrorKind::InvalidData, error.to_string()))
}

fn map_rand_error(error: getrandom::Error) -> VaultError {
    VaultError::FileIo(std::io::Error::new(std::io::ErrorKind::Other, error.to_string()))
}

fn get_password_sha1_hex(password: &str) -> String {
    let mut hasher = Sha1::new();
    hasher.update(password.as_bytes());
    let result = hasher.finalize();
    result.encode_hex::<String>().to_uppercase()
}

/// 判断密码是否泄露
static HIBP_CLIENT: Lazy<Client> = Lazy::new(|| {
    Client::builder()
        .timeout(Duration::from_secs(8))
        .user_agent("Tresor/1.0")
        .build()
        .unwrap()
});
#[derive(Debug, Serialize)]
pub struct PasswordLeakCheckResult {
    pub entry_id: String,
    /// true = 已泄露; false = 未泄露; None = 网络异常/检测失败
    pub compromised: Option<bool>,
}
async fn is_password_compromised(password: &str) -> Result<bool, VaultError> {
    let pwd_sha1_hex = get_password_sha1_hex(password);
    let (prefix, target_suffix) = pwd_sha1_hex.split_at(5);

    let url = format!("https://api.pwnedpasswords.com/range/{prefix}");

    let text = HIBP_CLIENT.get(url)
        .send()
        .await
        .map_err(|e| VaultError::Network(e.to_string()))?
        .text()
        .await
        .map_err(|e| VaultError::Network(e.to_string()))?;

    for line in text.lines() {
        if let Some((hibp_suffix, _count)) = line.split_once(':') {
            if hibp_suffix == target_suffix {
                return Ok(true);
            }
        }
    }

    Ok(false)
}

/// 批量扫描金库内所有密码，检测是否泄露
/// limit_concurrent: 并发数量，建议 2~4，防止触发HIBP限流
pub async fn check_all_password_leaks(
    root: &VaultStoreRoot,
    master_key: &[u8],
    concurrent_limit: usize,
) -> Vec<PasswordLeakCheckResult> {
    use futures::{stream, StreamExt};

    let tasks: Vec<_> = root.entries.iter().map(|entry| {
        let entry_id = entry.entry_id.clone();
        async move {
            let mut sub_key = match derive_entry_sub_key(master_key, &entry_id) {
                Ok(k) => k,
                Err(_) => {
                    return PasswordLeakCheckResult {
                        entry_id,
                        compromised: None,
                    };
                }
            };

            let mut pwd_buf = match decrypt_single_cipher(&entry.encrypted_password, &sub_key) {
                Ok(buf) => buf,
                Err(_) => {
                    unlock_memory(&mut sub_key);
                    sub_key.zeroize();
                    return PasswordLeakCheckResult {
                        entry_id,
                        compromised: None,
                    };
                }
            };

            let password = match std::str::from_utf8(&pwd_buf) {
                Ok(s) => zeroize::Zeroizing::new(s.to_string()),
                Err(_) => {
                    pwd_buf.zeroize();
                    unlock_memory(&mut sub_key);
                    sub_key.zeroize();
                    return PasswordLeakCheckResult {
                        entry_id,
                        compromised: None,
                    };
                }
            };

            let check_res = match is_password_compromised(&password).await {
                Ok(is_comp) => Some(is_comp),
                Err(_) => None,
            };

            pwd_buf.zeroize();
            unlock_memory(&mut pwd_buf);

            unlock_memory(&mut sub_key);
            sub_key.zeroize();

            PasswordLeakCheckResult {
                entry_id,
                compromised: check_res,
            }
        }
    }).collect();

    let results = stream::iter(tasks)
        .buffer_unordered(concurrent_limit)
        .collect()
        .await;

    results
}