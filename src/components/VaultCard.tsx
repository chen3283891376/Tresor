import { useRef, useState } from 'react';
import { Check, Copy, Eye, EyeOff, Globe } from 'lucide-react';
import { Card, CardContent, CardFooter } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import type { VaultMeta } from '@/types/vault';

interface VaultCardProps {
    data: VaultMeta;
    fetchPreview: (id: string) => Promise<string>;
    copyPassword: (id: string) => Promise<void>;
}

export default function VaultCard({ data, fetchPreview, copyPassword }: VaultCardProps) {
    const [showFlag, setShowFlag] = useState(false);
    const [copiedTip, setCopiedTip] = useState(false);
    const [loading, setLoading] = useState(false);
    const previewStr = useRef<string>('');

    const handleToggleView = async () => {
        if (showFlag) {
            previewStr.current = '';
            setShowFlag(false);
            return;
        }

        setLoading(true);
        try {
            previewStr.current = await fetchPreview(data.id);
            setShowFlag(true);

            // 3秒自动清空，缩短敏感数据内存存活时间
            setTimeout(() => {
                previewStr.current = '';
                setShowFlag(false);
            }, 3000);
        } catch (err) {
            console.error('获取预览失败', err);
        } finally {
            setLoading(false);
        }
    };

    const handleCopy = async () => {
        await copyPassword(data.id);
        setCopiedTip(true);
        setTimeout(() => setCopiedTip(false), 1400);
    };

    return (
        <Card className="w-full hover:shadow-lg transition-shadow">
            <CardContent className="p-4 md:p-5 space-y-3">
                <div className="flex flex-col gap-1 overflow-hidden">
                    <h3 className="text-base font-semibold truncate">{data.title}</h3>
                    {data.website && (
                        <div className="flex items-center gap-1 text-xs text-gray-500 truncate">
                            <Globe size={12} />
                            <span className="truncate">{data.website}</span>
                        </div>
                    )}
                </div>

                <div className="text-sm">
                    <span className="text-muted-foreground">账号：</span>
                    <span>{data.username}</span>
                </div>

                <div className="flex items-center gap-2">
                    <div className="flex-1 bg-secondary px-3 py-2 rounded text-sm font-mono select-none">
                        {loading ? '加载中...' : showFlag ? previewStr.current : '点击眼睛查看掩码'}
                    </div>
                    <Button variant="ghost" size="icon" onClick={handleToggleView}>
                        {showFlag ? <EyeOff size={16} /> : <Eye size={16} />}
                    </Button>
                </div>
            </CardContent>

            <CardFooter className="px-4 md:px-5 py-3 border-t justify-end">
                <Button size="sm" className="w-full md:w-auto" onClick={handleCopy}>
                    {copiedTip ? <Check size={14} className="mr-1" /> : <Copy size={14} className="mr-1" />}
                    {copiedTip ? '已复制' : '复制密码'}
                </Button>
            </CardFooter>
        </Card>
    );
}
