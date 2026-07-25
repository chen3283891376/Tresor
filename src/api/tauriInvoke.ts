import { invoke } from '@tauri-apps/api/core';

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
