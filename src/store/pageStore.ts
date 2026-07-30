import { create } from 'zustand';

type page = 'passwords' | '2fa' | 'settings';
interface pageState {
    currentPage: page;
    setCurrentPage: (newPage: page) => void;
}

export const usePageStore = create<pageState>(set => ({
    currentPage: 'passwords',

    setCurrentPage: (newPage: page) => set({ currentPage: newPage }),
}));
