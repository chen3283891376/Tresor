import { useEffect, useState } from 'react';
import { Button } from '@/components/ui/button.tsx';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card.tsx';
import { ShortcutRecorder } from '@/components/ShortcutRecorder.tsx';
import { getShortcutConfig, saveAndApplyShortcutConfig } from '@/api/tauriInvoke.ts';
import { toast } from 'sonner';
import type { ShortcutConfig } from '@/types';

const ACTION_LABELS: Record<string, string> = {
    paste_password: '粘贴密码',
    scan_qr: '二维码扫描',
};

export function SettingsPage() {
    const [config, setConfig] = useState<ShortcutConfig>({});
    const [loading, setLoading] = useState(true);
    const [saving, setSaving] = useState(false);

    useEffect(() => {
        getShortcutConfig().then(cfg => {
            setConfig(cfg);
            setLoading(false);
        }).catch(() => {
            toast.error('加载设置失败');
            setLoading(false);
        });
    }, []);

    const handleChange = (action: string, value: string) => {
        setConfig(prev => ({ ...prev, [action]: value }));
    };

    const handleSave = async () => {
        setSaving(true);
        try {
            await saveAndApplyShortcutConfig(config);
            toast.success('设置已保存');
        } catch (err) {
            toast.error(typeof err === 'string' ? err : '保存设置失败');
        } finally {
            setSaving(false);
        }
    };

    if (loading) {
        return (
            <div className="p-6">
                <h2 className="text-2xl font-bold mb-6">设置</h2>
                <p className="text-muted-foreground">加载中...</p>
            </div>
        );
    }

    return (
        <div className="p-6">
            <h2 className="text-2xl font-bold mb-6">设置</h2>
            <Card>
                <CardHeader>
                    <CardTitle>快捷键</CardTitle>
                </CardHeader>
                <CardContent>
                    <div className="space-y-4">
                        {Object.entries(ACTION_LABELS).map(([action, label]) => (
                            <div key={action} className="flex items-center justify-between gap-4">
                                <span className="text-sm">{label}</span>
                                <ShortcutRecorder
                                    value={config[action] ?? ''}
                                    onChange={val => handleChange(action, val)}
                                />
                            </div>
                        ))}
                    </div>
                    <div className="mt-6">
                        <Button onClick={handleSave} disabled={saving}>
                            {saving ? '保存中...' : '保存设置'}
                        </Button>
                    </div>
                </CardContent>
            </Card>
        </div>
    );
}
