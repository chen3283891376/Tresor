import { useState, useEffect } from 'react';
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogFooter } from '@/components/ui/dialog';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Textarea } from '@/components/ui/textarea';
import { PasswordInput } from './PasswordInput';
import { usePasswordStore } from '@/store/passwordStore';
import type { DecryptedEntry, NewEntryParams, UpdateEntryParams } from '@/types';
import { toast } from 'sonner';

interface EntryFormDialogProps {
    open: boolean;
    onOpenChange: (open: boolean) => void;
    editEntry?: DecryptedEntry;
}

export function EntryFormDialog({ open, onOpenChange, editEntry }: EntryFormDialogProps) {
    const { createEntry, updateEntry, isLoading } = usePasswordStore();
    const [account, setAccount] = useState('');
    const [password, setPassword] = useState('');
    const [url, setUrl] = useState('');
    const [note, setNote] = useState('');

    useEffect(() => {
        if (editEntry) {
            setAccount(editEntry.account);
            setPassword(editEntry.password);
            setUrl(editEntry.url || '');
            setNote(editEntry.note || '');
        } else {
            setAccount('');
            setPassword('');
            setUrl('');
            setNote('');
        }
    }, [editEntry]);

    const handleClose = () => {
        setAccount('');
        setPassword('');
        setUrl('');
        setNote('');
        onOpenChange(false);
    };

    const handleSubmit = async () => {
        if (!account || !password) {
            toast.error('请填写账号和密码');
            return;
        }

        try {
            if (editEntry) {
                const params: UpdateEntryParams = {
                    entry_id: editEntry.entry_id,
                    account: account !== editEntry.account ? account : undefined,
                    password: password !== editEntry.password ? password : undefined,
                    url: url !== (editEntry.url || '') ? url : undefined,
                    note: note !== (editEntry.note || '') ? note : undefined,
                };
                await updateEntry(params);
            } else {
                const params: NewEntryParams = {
                    account,
                    password,
                    url: url || undefined,
                    note: note || undefined,
                };
                await createEntry(params);
            }
            handleClose();
        } catch (err) {
            // Error already handled in store
        }
    };

    return (
        <Dialog open={open} onOpenChange={onOpenChange}>
            <DialogContent className="sm:max-w-lg">
                <DialogHeader>
                    <DialogTitle>{editEntry ? '编辑密码记录' : '新建密码记录'}</DialogTitle>
                </DialogHeader>
                <div className="space-y-4">
                    <div className="space-y-2">
                        <Label htmlFor="account">账号 / 用户名</Label>
                        <Input
                            id="account"
                            value={account}
                            onChange={e => setAccount(e.target.value)}
                            placeholder="例如: user@example.com"
                            disabled={isLoading}
                        />
                    </div>
                    <div className="space-y-2">
                        <Label htmlFor="password">密码</Label>
                        <PasswordInput
                            value={password}
                            onChange={setPassword}
                            placeholder="输入密码"
                            disabled={isLoading}
                        />
                    </div>
                    <div className="space-y-2">
                        <Label htmlFor="url">网址 (可选)</Label>
                        <Input
                            id="url"
                            value={url}
                            onChange={e => setUrl(e.target.value)}
                            placeholder="https://example.com"
                            disabled={isLoading}
                        />
                    </div>
                    <div className="space-y-2">
                        <Label htmlFor="note">备注 (可选)</Label>
                        <Textarea
                            id="note"
                            value={note}
                            onChange={e => setNote(e.target.value)}
                            placeholder="添加备注..."
                            disabled={isLoading}
                            rows={3}
                        />
                    </div>
                </div>
                <DialogFooter>
                    <Button variant="outline" onClick={handleClose} disabled={isLoading}>
                        取消
                    </Button>
                    <Button onClick={handleSubmit} disabled={isLoading}>
                        {isLoading ? '保存中...' : '保存'}
                    </Button>
                </DialogFooter>
            </DialogContent>
        </Dialog>
    );
}
