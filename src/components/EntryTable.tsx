import { useState } from 'react';
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table';
import { Button } from '@/components/ui/button';
import { Pencil, Trash2 } from 'lucide-react';
import { usePasswordStore } from '@/store/passwordStore';
import { EntryFormDialog } from './EntryFormDialog';
import { EntryDetailDialog } from './EntryDetailDialog';
import { DeleteConfirmDialog } from './DeleteConfirmDialog';

export function EntryTable() {
    const { previewList, passwordLeaks, getEntryDetail, isLoading, currentDetailEntry } = usePasswordStore();
    const [detailOpen, setDetailOpen] = useState(false);
    const [editOpen, setEditOpen] = useState(false);
    const [deleteOpen, setDeleteOpen] = useState(false);
    const [deleteEntryId, setDeleteEntryId] = useState<string>('');

    const handleEdit = async (entryId: string) => {
        await getEntryDetail(entryId);
        setEditOpen(true);
    };

    const handleDelete = (entryId: string) => {
        setDeleteEntryId(entryId);
        setDeleteOpen(true);
    };

    const formatDate = (timestamp: number) => {
        return new Date(timestamp * 1000).toLocaleDateString('zh-CN');
    };

    return (
        <>
            <div className="rounded-md border">
                <Table>
                    <TableHeader>
                        <TableRow>
                            <TableHead>网址</TableHead>
                            <TableHead>创建时间</TableHead>
                            <TableHead>状态</TableHead>
                            <TableHead className="text-right">操作</TableHead>
                        </TableRow>
                    </TableHeader>
                    <TableBody>
                        {previewList.length === 0 ? (
                            <TableRow>
                                <TableCell colSpan={3} className="text-center text-muted-foreground py-8">
                                    {isLoading ? '加载中...' : '暂无密码记录'}
                                </TableCell>
                            </TableRow>
                        ) : (
                            previewList.map(entry => (
                                <TableRow key={entry.entry_id}>
                                    <TableCell className="font-medium">{entry.url || '-'}</TableCell>
                                    <TableCell>{formatDate(entry.created_at)}</TableCell>
                                    <TableCell>
                                        {(() => {
                                            const leakItem = passwordLeaks.find(
                                                leak => leak.entry_id === entry.entry_id,
                                            );
                                            if (!leakItem) return '未检测';
                                            switch (leakItem.compromised) {
                                                case true:
                                                    return <span className="text-red-500 font-medium">已泄露</span>;
                                                case false:
                                                    return <span className="text-green-500">安全</span>;
                                                case null:
                                                    return <span className="text-gray-400">检测失败</span>;
                                            }
                                        })()}
                                    </TableCell>
                                    <TableCell className="text-right">
                                        <div className="flex justify-end gap-2">
                                            <Button
                                                variant="outline"
                                                size="icon"
                                                onClick={() => handleEdit(entry.entry_id)}
                                                disabled={isLoading}
                                            >
                                                <Pencil className="h-4 w-4" />
                                            </Button>
                                            <Button
                                                variant="outline"
                                                size="icon"
                                                onClick={() => handleDelete(entry.entry_id)}
                                                disabled={isLoading}
                                            >
                                                <Trash2 className="h-4 w-4" />
                                            </Button>
                                        </div>
                                    </TableCell>
                                </TableRow>
                            ))
                        )}
                    </TableBody>
                </Table>
            </div>

            <EntryDetailDialog open={detailOpen} onOpenChange={setDetailOpen} />
            <EntryFormDialog open={editOpen} onOpenChange={setEditOpen} editEntry={currentDetailEntry || undefined} />
            <DeleteConfirmDialog open={deleteOpen} onOpenChange={setDeleteOpen} entryId={deleteEntryId} />
        </>
    );
}
