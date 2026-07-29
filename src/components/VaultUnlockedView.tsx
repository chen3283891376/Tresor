import { useState, useEffect } from 'react';
import { Sidebar } from './Sidebar';
import { SidebarProvider, SidebarInset } from '@/components/ui/sidebar';
import { EntryFormDialog } from './EntryFormDialog';
import { usePasswordStore } from '@/store/passwordStore';
import { usePageStore } from '@/store/pageStore.ts';
import { PasswordPage } from '@/pages/PasswordPage.tsx';
import { TwoFAPage } from '@/pages/2FAPage.tsx';

export function VaultUnlockedView() {
    const [newEntryOpen, setNewEntryOpen] = useState(false);
    const { refreshAll } = usePasswordStore();
    const { currentPage } = usePageStore();

    useEffect(() => {
        refreshAll().then();
    }, [refreshAll]);

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
