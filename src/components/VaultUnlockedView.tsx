import { useState, useEffect } from 'react';
import { Sidebar } from './Sidebar';
import { SidebarProvider, SidebarInset } from '@/components/ui/sidebar';
import { EntryTable } from './EntryTable';
import { EntryFormDialog } from './EntryFormDialog';
import { Button } from '@/components/ui/button';
import { Plus, RefreshCcw } from 'lucide-react';
import { usePasswordStore } from '@/store/passwordStore';

export function VaultUnlockedView() {
    const [newEntryOpen, setNewEntryOpen] = useState(false);
    const { refreshAll } = usePasswordStore();

    useEffect(() => {
        refreshAll().then();
    }, [refreshAll]);

    return (
        <SidebarProvider>
            <Sidebar onNewEntry={() => setNewEntryOpen(true)} />
            <SidebarInset>
                <div className="p-6">
                    <div className="flex items-center justify-between mb-6">
                        <div>
                            <h2 className="text-2xl font-bold">密码记录</h2>
                            <p className="text-muted-foreground">管理您保存的所有密码</p>
                        </div>

                        <div className={'flex space-x-2'}>
                            <Button onClick={() => refreshAll()}>
                                <RefreshCcw className="h-4 w-4 mr-2" />
                                重载
                            </Button>
                            <Button onClick={() => setNewEntryOpen(true)}>
                                <Plus className="h-4 w-4 mr-2" />
                                新建密码
                            </Button>
                        </div>
                    </div>
                    <EntryTable />
                </div>
            </SidebarInset>
            <EntryFormDialog open={newEntryOpen} onOpenChange={setNewEntryOpen} />
        </SidebarProvider>
    );
}
