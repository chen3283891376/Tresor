import { useEffect, useState } from 'react';
import { Button } from '@/components/ui/button.tsx';
import { usePageStore } from '@/store/pageStore.ts';
import { useTwoFAStore } from '@/store/twoFAStore.ts';
import { TwoFACard } from '@/components/2FACard.tsx';
import { StepBack, Plus } from 'lucide-react';
import { TwoFAFormDialog } from '@/components/TwoFAFormDialog.tsx';

export const TwoFAPage = () => {
    const { setCurrentPage } = usePageStore();
    const { twoFAList, refreshList, deleteEntry, isLoading } = useTwoFAStore();
    const [formOpen, setFormOpen] = useState(false);

    useEffect(() => {
        refreshList().then();
    }, [refreshList]);

    const handleDelete = async (entryId: string) => {
        await deleteEntry(entryId);
    };

    return (
        <div className="p-6">
            <div className="flex items-center justify-between mb-6">
                <div>
                    <h2 className="text-2xl font-bold">2FA 管理</h2>
                    <p className="text-muted-foreground">管理您的双重身份验证设置</p>
                </div>

                <div className="flex space-x-2">
                    <Button onClick={() => setCurrentPage('passwords')} variant="outline">
                        <StepBack className="mr-2 h-4 w-4" />
                        返回
                    </Button>
                    <Button onClick={() => setFormOpen(true)}>
                        <Plus className="mr-2 h-4 w-4" />
                        添加
                    </Button>
                </div>
            </div>

            {isLoading && twoFAList.length === 0 ? (
                <div className="text-center text-muted-foreground py-12">加载中...</div>
            ) : twoFAList.length === 0 ? (
                <div className="text-center text-muted-foreground py-12">暂无2FA记录，点击"添加"创建</div>
            ) : (
                <div className="grid gap-4 sm:grid-cols-1 md:grid-cols-2 lg:grid-cols-3">
                    {twoFAList.map(entry => (
                        <TwoFACard
                            key={entry.entry_id}
                            entry={{
                                entry_id: entry.entry_id,
                                issuer: entry.issuer,
                                account: entry.account,
                            }}
                            onDelete={handleDelete}
                        />
                    ))}
                </div>
            )}

            <TwoFAFormDialog open={formOpen} onOpenChange={setFormOpen} />
        </div>
    );
};
