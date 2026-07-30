import { useState, useEffect } from 'react';
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogFooter } from '@/components/ui/dialog';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { useTwoFAStore } from '@/store/twoFAStore.ts';
import { toast } from 'sonner';
import { scanQrFromScreenshot } from '@/api/tauriInvoke.ts';

interface TwoFAFormDialogProps {
    open: boolean;
    onOpenChange: (open: boolean) => void;
}

export function TwoFAFormDialog({ open, onOpenChange }: TwoFAFormDialogProps) {
    const { createEntry, isLoading, pendingScanResult, setPendingScanResult } = useTwoFAStore();
    const [issuer, setIssuer] = useState('');
    const [account, setAccount] = useState('');
    const [secret, setSecret] = useState('');
    const [scanning, setScanning] = useState(false);

    useEffect(() => {
        if (open && pendingScanResult) {
            setIssuer(pendingScanResult.issuer);
            setAccount(pendingScanResult.account);
            setSecret(pendingScanResult.secret);
            setPendingScanResult(null);
        }
    }, [open, pendingScanResult, setPendingScanResult]);

    const handleScan = async () => {
        setScanning(true);
        try {
            const result = await scanQrFromScreenshot();
            setIssuer(result.issuer);
            setAccount(result.account);
            setSecret(result.secret);
            toast.success('二维码扫描成功');
        } catch (err) {
            toast.error(typeof err === 'string' ? err : '二维码扫描失败');
        } finally {
            setScanning(false);
        }
    };

    const handleClose = () => {
        setIssuer('');
        setAccount('');
        setSecret('');
        onOpenChange(false);
    };

    const handleSubmit = async () => {
        if (!issuer || !account || !secret) {
            toast.error('请填写所有必填字段');
            return;
        }

        try {
            await createEntry({ issuer, account, secret });
            handleClose();
        } catch {
            // Error already handled in store
        }
    };

    return (
        <Dialog open={open} onOpenChange={onOpenChange}>
            <DialogContent className="sm:max-w-lg">
                <DialogHeader>
                    <DialogTitle>添加2FA验证</DialogTitle>
                </DialogHeader>
                <div className="space-y-4">
                    <div className="space-y-2">
                        <Label htmlFor="issuer">颁发者</Label>
                        <Input
                            id="issuer"
                            value={issuer}
                            onChange={e => setIssuer(e.target.value)}
                            placeholder="例如: GitHub, Google"
                            disabled={isLoading}
                        />
                    </div>
                    <div className="space-y-2">
                        <Label htmlFor="account">账号</Label>
                        <Input
                            id="account"
                            value={account}
                            onChange={e => setAccount(e.target.value)}
                            placeholder="例如: user@example.com"
                            disabled={isLoading}
                        />
                    </div>
                    <div className="flex items-center gap-2 pt-2">
                        <div className="h-px flex-1 bg-border" />
                        <Button variant="outline" size="sm" onClick={handleScan} disabled={scanning}>
                            {scanning ? '扫描中...' : '扫描二维码（或按下Ctrl+Alt+S扫描）'}
                        </Button>
                        <div className="h-px flex-1 bg-border" />
                    </div>
                    <div className="space-y-2">
                        <Label htmlFor="secret">密钥 (Base32)</Label>
                        <Input
                            id="secret"
                            value={secret}
                            onChange={e => setSecret(e.target.value)}
                            placeholder="例如: JBSWY3DPEHPK3PXP"
                            disabled={isLoading}
                            className="font-mono"
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
