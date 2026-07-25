import { create } from 'zustand';
import {
    pickVaultKeyFile,
    clearSavedKeyPath,
    registerVault,
    unlockVault,
    lockVault,
    setVaultStoragePath,
    openVaultFilePicker,
    saveVaultFilePicker,
} from '../api/tauriInvoke';
import { toast } from 'sonner';

interface VaultState {
    isUnlocked: boolean;
    vaultFilePath: string | null;
    keyFilePath: string | null;
    vaultMeta: any | null;
    isLoading: boolean;
    setVaultFilePath: (path: string) => Promise<void>;
    openVaultFilePicker: () => Promise<void>;
    saveVaultFilePicker: () => Promise<void>;
    registerVault: (masterPwd: string) => Promise<void>;
    unlockVault: (masterPwd: string) => Promise<void>;
    lockVault: () => Promise<void>;
    pickKeyFile: () => Promise<boolean>;
    clearKeyFile: () => void;
    setLoading: (loading: boolean) => void;
}

export const useVaultStore = create<VaultState>((set, get) => ({
    isUnlocked: false,
    vaultFilePath: null,
    keyFilePath: null,
    vaultMeta: null,
    isLoading: false,

    setLoading: (loading: boolean) => set({ isLoading: loading }),

    setVaultFilePath: async (path: string) => {
        set({ isLoading: true });
        try {
            await setVaultStoragePath(path);
            set({ vaultFilePath: path });
            toast.success('金库路径设置成功');
        } catch (err: any) {
            toast.error(`设置金库路径失败: ${err}`);
        } finally {
            set({ isLoading: false });
        }
    },

    openVaultFilePicker: async () => {
        set({ isLoading: true });
        try {
            const path = await openVaultFilePicker();
            set({ vaultFilePath: path });
            toast.success('金库文件选择成功');
        } catch (err: any) {
            if (!String(err).includes('未选择文件')) {
                toast.error(`选择金库文件失败: ${err}`);
            }
        } finally {
            set({ isLoading: false });
        }
    },

    saveVaultFilePicker: async () => {
        set({ isLoading: true });
        try {
            const path = await saveVaultFilePicker();
            set({ vaultFilePath: path });
            toast.success('金库文件保存成功');
        } catch (err: any) {
            if (!String(err).includes('用户取消保存')) {
                toast.error(`保存金库文件失败: ${err}`);
            }
        } finally {
            set({ isLoading: false });
        }
    },

    registerVault: async (masterPwd: string) => {
        set({ isLoading: true });
        try {
            await registerVault(masterPwd);
            toast.success('金库创建成功！现在请选择或创建金库文件');
            const state = get();
            if (!state.vaultFilePath) {
                await state.saveVaultFilePicker();
            }
            set({ isUnlocked: true });
            toast.success('金库已解锁！');
        } catch (err: any) {
            if (String(err).includes('用户取消')) {
                toast.info('未保存密钥，创建流程已取消');
            } else if (String(err).includes('用户取消保存')) {
                toast.info('未保存金库文件，创建流程已取消');
            } else {
                toast.error(`创建金库失败: ${err}`);
            }
        } finally {
            set({ isLoading: false });
        }
    },

    unlockVault: async (masterPwd: string) => {
        set({ isLoading: true });
        try {
            await unlockVault(masterPwd);
            const state = get();
            if (!state.vaultFilePath) {
                toast.info('请先选择金库文件');
                await state.openVaultFilePicker();
            }
            set({ isUnlocked: true });
            toast.success('金库解锁成功！');

            await clearSavedKeyPath();
            set({ keyFilePath: null });
        } catch (err: any) {
            toast.error(`解锁失败: ${err}`);
            set({ isUnlocked: false });
        } finally {
            set({ isLoading: false });
        }
    },

    lockVault: async () => {
        set({ isLoading: true });
        try {
            await lockVault();
            set({ isUnlocked: false, vaultMeta: null });
            toast.info('金库已锁定');

            await clearSavedKeyPath();
            set({ keyFilePath: null });
        } catch (err: any) {
            toast.error(`锁定金库失败: ${err}`);
        } finally {
            set({ isLoading: false });
        }
    },

    pickKeyFile: async () => {
        try {
            const ok = await pickVaultKeyFile();
            if (ok) {
                set({ keyFilePath: 'selected' });
                return true;
            } else {
                toast.info('未选择有效 .key 密钥文件');
                return false;
            }
        } catch (err: any) {
            toast.error(`文件选择失败: ${err}`);
            return false;
        }
    },

    clearKeyFile: async () => {
        try {
            await clearSavedKeyPath();
            set({ keyFilePath: null });
        } catch (err: any) {
            console.error('清空密钥文件路径失败:', err);
        }
    },
}));
