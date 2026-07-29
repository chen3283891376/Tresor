import { Button } from '@/components/ui/button.tsx';
import { Plus, QrCode, RefreshCcw } from 'lucide-react';
import { EntryTable } from '@/components/EntryTable.tsx';
import { useEffect } from 'react';
import { usePasswordStore } from '@/store/passwordStore.ts';
import { usePageStore } from '@/store/pageStore.ts';

export const PasswordPage = ({ setNewEntryOpen }: { setNewEntryOpen: (open: boolean) => void }) => {
    const { refreshAll } = usePasswordStore();
    const { setCurrentPage } = usePageStore();

    useEffect(() => {
        refreshAll().then();
    }, [refreshAll]);

    return (
        <div className="p-6">
            <div className="flex items-center justify-between mb-6">
                <div>
                    <h2 className="text-2xl font-bold">密码记录</h2>
                    <p className="text-muted-foreground">管理您保存的所有密码</p>
                </div>

                <div className={'flex space-x-2'}>
                    <Button onClick={() => setCurrentPage('2fa')}>
                        <QrCode className="h-4 w-4 mr-2" />
                        2FA 管理
                    </Button>
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
    );
};
