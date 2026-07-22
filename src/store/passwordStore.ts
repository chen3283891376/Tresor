import { create } from 'zustand';
import {
    loadVaultPreview,
    createPasswordEntry,
    getEntryDetail,
    updateEntry,
    deleteEntry,
    loadPasswordLeaks,
} from '../api/tauriInvoke';
import { toast } from 'sonner';
import type { EntryMetaPreview, DecryptedEntry, NewEntryParams, UpdateEntryParams, PasswordLeak } from '../types';

interface PasswordState {
    previewList: EntryMetaPreview[];
    passwordLeaks: PasswordLeak[];
    currentDetailEntry: DecryptedEntry | null;
    isLoading: boolean;
    refreshPreviewList: () => Promise<void>;
    refreshPasswordLeaks: () => Promise<void>;
    createEntry: (params: NewEntryParams) => Promise<void>;
    getEntryDetail: (entryId: string) => Promise<void>;
    updateEntry: (params: UpdateEntryParams) => Promise<void>;
    deleteEntry: (entryId: string) => Promise<void>;
    clearCurrentDetail: () => void;
    setLoading: (loading: boolean) => void;
}

export const usePasswordStore = create<PasswordState>((set, get) => ({
    previewList: [],
    passwordLeaks: [],
    currentDetailEntry: null,
    isLoading: false,

    setLoading: (loading: boolean) => set({ isLoading: loading }),

    refreshPreviewList: async () => {
        set({ isLoading: true });
        try {
            const list = await loadVaultPreview();
            set({ previewList: list || [] });
        } catch (err: any) {
            toast.error(`加载密码列表失败: ${err}`);
            set({ previewList: [] });
        } finally {
            set({ isLoading: false });
        }
    },

    refreshPasswordLeaks: async () => {
        set({ isLoading: true });
        try {
            const list = await loadPasswordLeaks();
            set({ passwordLeaks: list || [] });
        } catch (err: any) {
            toast.error(`加载密码泄露记录失败: ${err}`);
            set({ passwordLeaks: [] });
        } finally {
            set({ isLoading: false });
        }
    },

    createEntry: async (params: NewEntryParams) => {
        set({ isLoading: true });
        try {
            await createPasswordEntry(params.account, params.password, params.url, params.note);
            toast.success('密码记录创建成功');
            await get().refreshPreviewList();
            await get().refreshPasswordLeaks();
        } catch (err: any) {
            toast.error(`创建密码记录失败: ${err}`);
        } finally {
            set({ isLoading: false });
        }
    },

    getEntryDetail: async (entryId: string) => {
        set({ isLoading: true });
        try {
            const detail = await getEntryDetail(entryId);
            set({ currentDetailEntry: detail });
        } catch (err: any) {
            toast.error(`获取密码详情失败: ${err}`);
        } finally {
            set({ isLoading: false });
        }
    },

    updateEntry: async (params: UpdateEntryParams) => {
        set({ isLoading: true });
        try {
            await updateEntry(params.entry_id, params.account, params.password, params.url, params.note);
            toast.success('密码记录更新成功');
            await get().refreshPreviewList();
        } catch (err: any) {
            toast.error(`更新密码记录失败: ${err}`);
        } finally {
            set({ isLoading: false });
        }
    },

    deleteEntry: async (entryId: string) => {
        set({ isLoading: true });
        try {
            await deleteEntry(entryId);
            toast.success('密码记录删除成功');
            await get().refreshPreviewList();
        } catch (err: any) {
            toast.error(`删除密码记录失败: ${err}`);
        } finally {
            set({ isLoading: false });
        }
    },

    clearCurrentDetail: () => {
        set({ currentDetailEntry: null });
    },
}));
