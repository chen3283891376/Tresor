import { invoke } from '@tauri-apps/api/core';
import type { TwoFAEntryPreview, DecryptedTwoFAEntry, PasswordGeneratorConfig, QrScanResult, ShortcutConfig } from '@/types';

// 金库密钥相关
export const pickVaultKeyFile = (): Promise<boolean> => invoke('open_key_file_picker');
export const clearSavedKeyPath = (): Promise<void> => invoke('clear_stored_key_path');
export const registerVault = (user_pwd: string): Promise<void> => invoke('register_vault', { userPwd: user_pwd });
export const unlockVault = (user_pwd: string): Promise<void> => invoke('unlock_vault', { userPwd: user_pwd });
export const lockVault = (): Promise<void> => invoke('clear_active_master_key');

// 金库存储相关
export const setVaultStoragePath = (path: string): Promise<void> => invoke('set_vault_storage_path', { path });
export const openVaultFilePicker = (): Promise<string> => invoke('open_vault_file_picker');
export const saveVaultFilePicker = (): Promise<string> => invoke('save_vault_file_picker');
export const loadVaultPreview = (): Promise<any[]> => invoke('load_vault_store');
export const loadPasswordLeaks = (): Promise<any[]> => invoke('load_password_leaks');
export const createPasswordEntry = (account: string, pwd: string, url?: string, note?: string): Promise<void> =>
    invoke('create_password_entry', { account, pwd, url, note });
export const getEntryDetail = (entry_id: string): Promise<any> => invoke('get_decrypted_entry', { entryId: entry_id });
export const updateEntry = (
    entry_id: string,
    new_account?: string,
    new_pwd?: string,
    new_url?: string,
    new_note?: string,
): Promise<void> =>
    invoke('update_password_entry', {
        entryId: entry_id,
        newAccount: new_account,
        newPwd: new_pwd,
        newUrl: new_url,
        newNote: new_note,
    });
export const deleteEntry = (entry_id: string): Promise<void> => invoke('delete_password_entry', { entryId: entry_id });
export const forceSaveVault = (): Promise<void> => invoke('save_vault_store');

export const copyEntry = (entry_id: string): Promise<void> => invoke('set_paste_pwd', { entryId: entry_id });

// 密码生成
export const generateStrongPassword = (config: PasswordGeneratorConfig): Promise<string> =>
    invoke('generate_strong_password', { config });

// 2FA
export const createTwoFAEntry = (issuer: string, account: string, secret: string): Promise<void> =>
    invoke('create_two_fa_entry', { issuer, account, secret });
export const loadTwoFAStore = (): Promise<TwoFAEntryPreview[]> => invoke('load_two_fa_store');
export const getTwoFAEntryDetail = (entry_id: string): Promise<DecryptedTwoFAEntry> =>
    invoke('get_decrypted_two_fa_entry', { entryId: entry_id });
export const updateTwoFAEntry = (
    entry_id: string,
    new_issuer?: string,
    new_account?: string,
    new_secret?: string,
): Promise<void> =>
    invoke('update_two_fa_entry', {
        entryId: entry_id,
        newIssuer: new_issuer,
        newAccount: new_account,
        newSecret: new_secret,
    });
export const deleteTwoFAEntry = (entry_id: string): Promise<void> =>
    invoke('delete_two_fa_entry', { entryId: entry_id });

export const computeTotpCode = (entry_id: string): Promise<[string, number]> =>
    invoke('compute_totp_code', { entryId: entry_id });

export const scanQrFromScreenshot = (): Promise<QrScanResult> =>
    invoke('scan_qr_from_screenshot');

export const scanQrFromImage = (): Promise<QrScanResult> =>
    invoke('scan_qr_from_image');

// 快捷键设置
export const getShortcutConfig = (): Promise<ShortcutConfig> =>
    invoke('get_shortcut_config');

export const saveAndApplyShortcutConfig = (config: ShortcutConfig): Promise<void> =>
    invoke('save_and_apply_shortcut_config', { config });

export const checkShortcutAvailable = (shortcutStr: string): Promise<boolean> =>
    invoke('check_shortcut_available', { shortcutStr });
