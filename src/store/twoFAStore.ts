import { create } from 'zustand';
import {
    createTwoFAEntry,
    loadTwoFAStore,
    getTwoFAEntryDetail,
    updateTwoFAEntry,
    deleteTwoFAEntry,
} from '@/api/tauriInvoke.ts';
import { toast } from 'sonner';
import { useVaultStore } from '@/store/vaultStore.ts';
import type { TwoFAEntryPreview, DecryptedTwoFAEntry, NewTwoFAParams, UpdateTwoFAParams } from '@/types';

interface TwoFAState {
    twoFAList: TwoFAEntryPreview[];
    currentDetail: DecryptedTwoFAEntry | null;
    isLoading: boolean;
    refreshList: () => Promise<void>;
    createEntry: (params: NewTwoFAParams) => Promise<void>;
    getEntryDetail: (entryId: string) => Promise<void>;
    updateEntry: (params: UpdateTwoFAParams) => Promise<void>;
    deleteEntry: (entryId: string) => Promise<void>;
    clearCurrentDetail: () => void;
    setLoading: (loading: boolean) => void;
}

export const useTwoFAStore = create<TwoFAState>((set, get) => ({
    twoFAList: [],
    currentDetail: null,
    isLoading: false,

    setLoading: (loading: boolean) => set({ isLoading: loading }),

    refreshList: async () => {
        try {
            const list = await loadTwoFAStore();
            set({ twoFAList: list || [] });
        } catch (err: any) {
            toast.error(`加载2FA列表失败: ${err}`);
            set({ twoFAList: [] });
            await useVaultStore.getState().lockVault();
        }
    },

    createEntry: async (params: NewTwoFAParams) => {
        set({ isLoading: true });
        try {
            await createTwoFAEntry(params.issuer, params.account, params.secret);
            toast.success('2FA记录创建成功');
            await get().refreshList();
        } catch (err: any) {
            toast.error(`创建2FA记录失败: ${err}`);
        } finally {
            set({ isLoading: false });
        }
    },

    getEntryDetail: async (entryId: string) => {
        set({ isLoading: true });
        try {
            const detail = await getTwoFAEntryDetail(entryId);
            set({ currentDetail: detail });
        } catch (err: any) {
            await useVaultStore.getState().lockVault();
            toast.error(`获取2FA详情失败: ${err}`);
        } finally {
            set({ isLoading: false });
        }
    },

    updateEntry: async (params: UpdateTwoFAParams) => {
        set({ isLoading: true });
        try {
            await updateTwoFAEntry(params.entry_id, params.issuer, params.account, params.secret);
            toast.success('2FA记录更新成功');
            await get().refreshList();
        } catch (err: any) {
            toast.error(`更新2FA记录失败: ${err}`);
        } finally {
            set({ isLoading: false });
        }
    },

    deleteEntry: async (entryId: string) => {
        set({ isLoading: true });
        try {
            await deleteTwoFAEntry(entryId);
            toast.success('2FA记录删除成功');
            await get().refreshList();
        } catch (err: any) {
            toast.error(`删除2FA记录失败: ${err}`);
        } finally {
            set({ isLoading: false });
        }
    },

    clearCurrentDetail: () => {
        set({ currentDetail: null });
    },
}));
