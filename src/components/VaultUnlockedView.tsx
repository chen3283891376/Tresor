import { useState, useEffect } from 'react';
import { listen } from '@tauri-apps/api/event';
import { Sidebar } from './Sidebar';
import { SidebarProvider, SidebarInset } from '@/components/ui/sidebar';
import { EntryFormDialog } from './EntryFormDialog';
import { usePasswordStore } from '@/store/passwordStore';
import { usePageStore } from '@/store/pageStore.ts';
import { useTwoFAStore } from '@/store/twoFAStore.ts';
import { scanQrFromScreenshot } from '@/api/tauriInvoke.ts';
import { PasswordPage } from '@/pages/PasswordPage.tsx';
import { TwoFAPage } from '@/pages/2FAPage.tsx';

export function VaultUnlockedView() {
    const [newEntryOpen, setNewEntryOpen] = useState(false);
    const { refreshAll } = usePasswordStore();
    const { currentPage, setCurrentPage } = usePageStore();
    const { setPendingScanResult } = useTwoFAStore();

    useEffect(() => {
        refreshAll().then();
    }, [refreshAll]);

    useEffect(() => {
        const unlisten = listen('trigger_qr_scan', async () => {
            try {
                const result = await scanQrFromScreenshot();
                setPendingScanResult(result);
                if (currentPage !== '2fa') {
                    setCurrentPage('2fa');
                }
            } catch {
                // error toast handled inside dialog
            }
        });
        return () => {
            unlisten.then(fn => fn());
        };
    }, [currentPage, setCurrentPage, setPendingScanResult]);

    return (
        <SidebarProvider>
            <Sidebar onNewEntry={() => setNewEntryOpen(true)} />
            <SidebarInset>
                {(() => {
                    switch (currentPage) {
                        case 'passwords':
                            return <PasswordPage setNewEntryOpen={setNewEntryOpen} />;
                        case '2fa':
                            return <TwoFAPage />;
                        default:
                            return <PasswordPage setNewEntryOpen={setNewEntryOpen} />;
                    }
                })()}
            </SidebarInset>
            <EntryFormDialog open={newEntryOpen} onOpenChange={setNewEntryOpen} />
        </SidebarProvider>
    );
}
