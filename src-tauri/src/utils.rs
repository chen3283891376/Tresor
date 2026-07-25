use argon2::Argon2;
use hkdf::{Hkdf, InvalidLength};
use once_cell::sync::Lazy;
use sha2::Sha256;
use std::sync::RwLock;
use thiserror::Error;
use zeroize::Zeroize;

#[cfg(unix)]
use libc;
#[cfg(windows)]
use winapi::{ctypes::c_void, um::memoryapi::{VirtualLock, VirtualUnlock}};

// 全局活跃主密钥单例
pub static ACTIVE_VAULT_KEY: Lazy<RwLock<Option<VaultMasterKey>>> =
    Lazy::new(|| RwLock::new(None));
pub static NEED_PASTE_PWD: Lazy<RwLock<Option<String>>> = Lazy::new(|| RwLock::new(None));

/// 金库主密钥：内存锁定 + 自动释放锁+清零
#[derive(Zeroize, Clone)]
pub struct VaultMasterKey {
    pub inner: Vec<u8>,
}

impl Drop for VaultMasterKey {
    fn drop(&mut self) {
        unlock_memory(&mut self.inner);
        self.inner.zeroize();
    }
}

/// 锁定缓冲区禁止swap交换
pub fn lock_memory(buf: &mut [u8]) -> Result<(), VaultError> {
    let len = buf.len();
    #[cfg(unix)]
    unsafe {
        let ptr = buf.as_mut_ptr() as *mut libc::c_void;
        if libc::mlock(ptr, len) != 0 {
            return Err(VaultError::MemLock(std::io::Error::last_os_error()));
        }
    }
    #[cfg(windows)]
    unsafe {
        let ptr = buf.as_mut_ptr() as *mut c_void;
        if VirtualLock(ptr, len) == 0 {
            return Err(VaultError::MemLock(std::io::Error::last_os_error()));
        }
    }
    Ok(())
}

/// 解除内存锁定
pub fn unlock_memory(buf: &mut [u8]) {
    let len = buf.len();
    #[cfg(unix)]
    unsafe {
        let ptr = buf.as_mut_ptr() as *mut libc::c_void;
        libc::munlock(ptr, len);
    }
    #[cfg(windows)]
    unsafe {
        let ptr = buf.as_mut_ptr() as *mut c_void;
        VirtualUnlock(ptr, len);
    }
}

impl VaultMasterKey {
    /// 用户密码 + 密钥文件数据 + 盐 派生主密钥
    pub fn derive(user_pwd: &[u8], key_file_data: &[u8], salt: &[u8]) -> Result<Self, VaultError> {
        let argon2 = Argon2::default();
        let mut temp_key = [0u8; 32];
        argon2.hash_password_into(user_pwd, salt, &mut temp_key)?;

        let hkdf: Hkdf<Sha256> = Hkdf::new(Some(&temp_key), &[]);
        let mut master_buf = vec![0u8; 32];
        hkdf.expand(key_file_data, &mut master_buf)?;

        lock_memory(&mut master_buf)?;
        temp_key.zeroize();

        Ok(Self { inner: master_buf })
    }
}

#[derive(Error, Debug)]
pub enum VaultError {
    #[error("文件IO失败: {0}")]
    FileIo(std::io::Error),
    #[error("Argon2密码派生失败: {0}")]
    KdfError(argon2::Error),
    #[error("HKDF密钥长度非法")]
    HkdfLength(InvalidLength),
    #[error("未选择U盘密钥文件")]
    NoKeyFile,
    #[error("内存锁定失败: {0}")]
    MemLock(std::io::Error),
    #[error("用户取消密钥保存")]
    UserCancelSave,
    #[error("网络出错")]
    Network(String),
}

impl From<std::io::Error> for VaultError {
    fn from(e: std::io::Error) -> Self {
        VaultError::FileIo(e)
    }
}
impl From<argon2::Error> for VaultError {
    fn from(e: argon2::Error) -> Self {
        VaultError::KdfError(e)
    }
}
impl From<InvalidLength> for VaultError {
    fn from(e: InvalidLength) -> Self {
        VaultError::HkdfLength(e)
    }
}

/// 设置全局激活主密钥
/// salt：新建金库/加载金库从vault文件meta读取，外部传入，不再内部随机
pub fn set_active_master_key(
    pwd_buf: &[u8],
    key_file_data: &[u8],
    salt: &[u8; 16],
) -> Result<(), Box<dyn std::error::Error>> {
    let key = VaultMasterKey::derive(pwd_buf, key_file_data, salt)
        .map_err(|e| e.to_string())?;

    let mut lock = ACTIVE_VAULT_KEY.write()?;
    if let Some(old_key) = lock.take() {
        drop(old_key);
    }
    *lock = Some(key);
    Ok(())
}

/// 获取当前解锁的主密钥
pub fn get_active_master_key() -> Result<VaultMasterKey, String> {
    let lock = ACTIVE_VAULT_KEY.read().unwrap();
    lock.clone().ok_or("金库未解锁，请先输入密码并加载密钥文件".parse().unwrap())
}

/// 清空全局主密钥（登出）
#[tauri::command]
pub fn clear_active_master_key() {
    let mut lock = ACTIVE_VAULT_KEY.write().unwrap();
    lock.take();
}

pub fn set_need_paste_pwd(pwd: String) -> Result<(), Box<dyn std::error::Error>> {
    let mut lock = NEED_PASTE_PWD.write()?;
    if let Some(old_pwd) = lock.take() {
        drop(old_pwd);
    }
    *lock = Some(pwd);
    Ok(())
}
pub fn get_need_paste_pwd() -> Result<String, Box<dyn std::error::Error>> {
    let lock = NEED_PASTE_PWD.read()?;
    lock.clone().ok_or("没有需要粘贴的密码".into())
}
pub fn clear_need_paste_pwd() {
    let mut lock = NEED_PASTE_PWD.write().unwrap();
    lock.take();
}