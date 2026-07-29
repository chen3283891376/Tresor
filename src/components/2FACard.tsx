import { useState, useEffect, useCallback, useRef } from 'react';
import { Card, CardContent, CardFooter } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Copy, Check, Pencil, Trash2, KeyRound } from 'lucide-react';
import { cn } from '@/lib/utils';
import { toast } from 'sonner';
import { computeTotpCode } from '@/api/tauriInvoke.ts';

export interface TwoFAEntry {
    entry_id: string;
    issuer: string;
    account: string;
}

interface TwoFACardProps {
    entry: TwoFAEntry;
    onEdit?: (entryId: string) => void;
    onDelete?: (entryId: string) => void;
}

export function TwoFACard({ entry, onEdit, onDelete }: TwoFACardProps) {
    const [copied, setCopied] = useState(false);
    const [currentCode, setCurrentCode] = useState<string | null>(null);
    const [timeRemaining, setTimeRemaining] = useState(30);
    const [loading, setLoading] = useState(false);
    const intervalRef = useRef<ReturnType<typeof setInterval> | null>(null);

    const fetchCode = useCallback(async () => {
        if (loading) return;
        setLoading(true);
        try {
            const [code, remaining] = await computeTotpCode(entry.entry_id);
            setCurrentCode(code);
            setTimeRemaining(remaining);
        } catch {
            setCurrentCode(null);
        } finally {
            setLoading(false);
        }
    }, [entry.entry_id]);

    useEffect(() => {
        fetchCode().then();

        intervalRef.current = setInterval(() => {
            setTimeRemaining(prev => {
                if (prev <= 1) {
                    fetchCode().then();
                    return 30;
                }
                return prev - 1;
            });
        }, 1000);

        return () => {
            if (intervalRef.current) {
                clearInterval(intervalRef.current);
            }
        };
    }, [fetchCode]);

    const handleCopy = async () => {
        if (!currentCode) return;
        try {
            await navigator.clipboard.writeText(currentCode);
            setCopied(true);
            toast.success('验证码已复制到剪贴板');
            setTimeout(() => setCopied(false), 2000);
        } catch {
            toast.error('复制失败');
        }
    };

    const isExpiring = timeRemaining <= 5;

    return (
        <Card>
            <CardContent className="pt-[--card-spacing]">
                <div className="flex items-start justify-between">
                    <div className="flex items-center gap-3">
                        <div className="flex h-10 w-10 items-center justify-center rounded-lg bg-primary/10">
                            <KeyRound className="h-5 w-5 text-primary" />
                        </div>
                        <div>
                            <p className="font-medium">{entry.issuer}</p>
                            <p className="text-sm text-muted-foreground">{entry.account}</p>
                        </div>
                    </div>
                </div>

                <div className="mt-4">
                    {currentCode ? (
                        <>
                            <div className="flex items-center gap-3">
                                <span
                                    className={cn(
                                        'text-3xl font-mono font-bold tracking-[0.2em]',
                                        isExpiring && 'text-destructive',
                                    )}
                                >
                                    {currentCode}
                                </span>
                                <Button variant="ghost" size="icon" onClick={handleCopy}>
                                    {copied ? <Check className="h-4 w-4" /> : <Copy className="h-4 w-4" />}
                                </Button>
                            </div>

                            <div className="mt-2 h-1 w-full rounded-full bg-muted">
                                <div
                                    className={cn(
                                        'h-full rounded-full transition-all duration-1000',
                                        isExpiring ? 'bg-destructive' : 'bg-primary',
                                    )}
                                    style={{ width: `${(timeRemaining / 30) * 100}%` }}
                                />
                            </div>
                            <p className="mt-1 text-xs text-muted-foreground">{timeRemaining}s</p>
                        </>
                    ) : (
                        <Button
                            variant="outline"
                            className="w-full"
                            onClick={fetchCode}
                            disabled={loading}
                        >
                            {loading ? '加载中...' : '显示验证码'}
                        </Button>
                    )}
                </div>
            </CardContent>
            {(onEdit || onDelete) && (
                <CardFooter className="justify-end gap-2">
                    {onEdit && (
                        <Button variant="outline" size="sm" onClick={() => onEdit(entry.entry_id)}>
                            <Pencil className="mr-1 h-4 w-4" />
                            编辑
                        </Button>
                    )}
                    {onDelete && (
                        <Button variant="outline" size="sm" onClick={() => onDelete(entry.entry_id)}>
                            <Trash2 className="mr-1 h-4 w-4" />
                            删除
                        </Button>
                    )}
                </CardFooter>
            )}
        </Card>
    );
}
