import { useEffect } from 'react';
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogFooter } from '@/components/ui/dialog';
import { Button } from '@/components/ui/button';
import { Label } from '@/components/ui/label';
import { Copy, Check } from 'lucide-react';
import { usePasswordStore } from '@/store/passwordStore';
import { useState } from 'react';
import { toast } from 'sonner';

interface EntryDetailDialogProps {
    open: boolean;
    onOpenChange: (open: boolean) => void;
}

export function EntryDetailDialog({ open, onOpenChange }: EntryDetailDialogProps) {
    const { currentDetailEntry, clearCurrentDetail } = usePasswordStore();
    const [copied, setCopied] = useState<'account' | 'password' | null>(null);

    const handleClose = () => {
        clearCurrentDetail();
        onOpenChange(false);
    };

    const copyToClipboard = async (text: string, type: 'account' | 'password') => {
        try {
            await navigator.clipboard.writeText(text);
            setCopied(type);
            toast.success(`${type === 'account' ? '账号' : '密码'}已复制到剪贴板`);
            setTimeout(() => {
                setCopied(null);
                if (type === 'password') {
                    clearCurrentDetail();
                }
            }, 2000);
        } catch (err) {
            toast.error('复制失败');
        }
    };

    useEffect(() => {
        return () => {
            clearCurrentDetail();
        };
    }, [clearCurrentDetail]);

    if (!currentDetailEntry) return null;

    return (
        <Dialog open={open} onOpenChange={onOpenChange}>
            <DialogContent className="sm:max-w-lg">
                <DialogHeader>
                    <DialogTitle>密码详情</DialogTitle>
                </DialogHeader>
                <div className="space-y-4">
                    <div className="space-y-2">
                        <Label>账号 / 用户名</Label>
                        <div className="flex gap-2">
                            <div className="flex-1 p-2 bg-muted rounded-md font-mono text-sm break-all">
                                {currentDetailEntry.account}
                            </div>
                            <Button
                                variant="outline"
                                size="icon"
                                onClick={() => copyToClipboard(currentDetailEntry.account, 'account')}
                            >
                                {copied === 'account' ? <Check className="h-4 w-4" /> : <Copy className="h-4 w-4" />}
                            </Button>
                        </div>
                    </div>
                    <div className="space-y-2">
                        <Label>密码</Label>
                        <div className="flex gap-2">
                            <div className="flex-1 p-2 bg-muted rounded-md font-mono text-sm break-all">
                                {currentDetailEntry.password}
                            </div>
                            <Button
                                variant="outline"
                                size="icon"
                                onClick={() => copyToClipboard(currentDetailEntry.password, 'password')}
                            >
                                {copied === 'password' ? <Check className="h-4 w-4" /> : <Copy className="h-4 w-4" />}
                            </Button>
                        </div>
                    </div>
                    {currentDetailEntry.url && (
                        <div className="space-y-2">
                            <Label>网址</Label>
                            <div className="p-2 bg-muted rounded-md text-sm break-all">{currentDetailEntry.url}</div>
                        </div>
                    )}
                    {currentDetailEntry.note && (
                        <div className="space-y-2">
                            <Label>备注</Label>
                            <div className="p-2 bg-muted rounded-md text-sm break-all whitespace-pre-wrap">
                                {currentDetailEntry.note}
                            </div>
                        </div>
                    )}
                </div>
                <DialogFooter>
                    <Button onClick={handleClose}>关闭</Button>
                </DialogFooter>
            </DialogContent>
        </Dialog>
    );
}
