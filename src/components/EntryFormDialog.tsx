import { useState, useEffect } from 'react';
import { Sparkles, RefreshCw, Eye, EyeOff } from 'lucide-react';
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogFooter } from '@/components/ui/dialog';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Textarea } from '@/components/ui/textarea';
import { Popover, PopoverContent, PopoverTrigger } from '@/components/ui/popover';
import { usePasswordStore } from '@/store/passwordStore';
import { generateStrongPassword } from '@/api/tauriInvoke';
import type { DecryptedEntry, NewEntryParams, UpdateEntryParams, PasswordGeneratorConfig } from '@/types';
import { toast } from 'sonner';
import { FieldGroup, Field, FieldLabel } from '@/components/ui/field';
import { Checkbox } from '@/components/ui/checkbox.tsx';

const checkboxList = [
    ['大写字母 (A-Z)', 'include_uppercase'] as const,
    ['小写字母 (a-z)', 'include_lowercase'] as const,
    ['数字 (0-9)', 'include_digits'] as const,
    ['符号 (!@#$%...)', 'include_symbols'] as const,
    ['排除易混淆 (0O1lI|)', 'exclude_ambiguous'] as const,
];

interface EntryFormDialogProps {
    open: boolean;
    onOpenChange: (open: boolean) => void;
    editEntry?: DecryptedEntry;
}

const DEFAULT_GEN_CONFIG: PasswordGeneratorConfig = {
    length: 12,
    include_uppercase: true,
    include_lowercase: true,
    include_digits: true,
    include_symbols: true,
    exclude_ambiguous: false,
    custom_symbols: undefined,
};

export function EntryFormDialog({ open, onOpenChange, editEntry }: EntryFormDialogProps) {
    const { createEntry, updateEntry, isLoading } = usePasswordStore();
    const [account, setAccount] = useState('');
    const [password, setPassword] = useState('');
    const [url, setUrl] = useState('');
    const [note, setNote] = useState('');
    const [showPassword, setShowPassword] = useState(false);
    const [genConfig, setGenConfig] = useState<PasswordGeneratorConfig>(DEFAULT_GEN_CONFIG);
    const [isGenerating, setIsGenerating] = useState(false);

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
        setShowPassword(false);
        onOpenChange(false);
    };

    const handleGenerate = async () => {
        setIsGenerating(true);
        try {
            const result = await generateStrongPassword(genConfig);
            setPassword(result);
        } catch (err: any) {
            toast.error(`生成密码失败: ${err}`);
        } finally {
            setIsGenerating(false);
        }
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
                        <div className="flex gap-2">
                            <div className="relative flex-1">
                                <Input
                                    id="password"
                                    type={showPassword ? 'text' : 'password'}
                                    value={password}
                                    onChange={e => setPassword(e.target.value)}
                                    placeholder="输入密码"
                                    disabled={isLoading}
                                    className="pr-10"
                                />
                                <Button
                                    type="button"
                                    variant="ghost"
                                    size="icon"
                                    onClick={() => setShowPassword(!showPassword)}
                                    disabled={isLoading}
                                    className="absolute right-0 top-0 h-full"
                                >
                                    {showPassword ? <EyeOff className="h-4 w-4" /> : <Eye className="h-4 w-4" />}
                                </Button>
                            </div>
                            <Popover>
                                <PopoverTrigger
                                    render={<Button type="button" variant="outline" size="icon" disabled={isLoading} />}
                                >
                                    <Sparkles className="h-4 w-4" />
                                </PopoverTrigger>
                                <PopoverContent align="end" className="w-64">
                                    <div className="space-y-3">
                                        <div className="font-medium text-sm">生成密码</div>
                                        <div className="space-y-2">
                                            <div className="flex items-center justify-between">
                                                <Label htmlFor="gen-length" className="text-xs">
                                                    密码长度
                                                </Label>
                                                <Input
                                                    id="gen-length"
                                                    type="number"
                                                    value={genConfig.length}
                                                    onBlur={e => {
                                                        const currentNum = Number(e.target.value);
                                                        if (currentNum < 4) {
                                                            e.target.value = '4';
                                                            return;
                                                        } else if (currentNum > 128) {
                                                            e.target.value = '128';
                                                        }
                                                    }}
                                                    onChange={e => {
                                                        setGenConfig({
                                                            ...genConfig,
                                                            length: Number(e.target.value) || 12,
                                                        });
                                                    }}
                                                    className="w-16 h-7 text-xs text-center"
                                                />
                                            </div>
                                            <FieldGroup className="mx-auto w-56">
                                                {checkboxList.map(([label, key]) => (
                                                    <Field key={key} orientation="horizontal">
                                                        <Checkbox
                                                            id={key}
                                                            name={key}
                                                            checked={genConfig[key]}
                                                            onCheckedChange={checked => {
                                                                setGenConfig(prev => ({ ...prev, [key]: checked }));
                                                            }}
                                                        />
                                                        <FieldLabel htmlFor={key} className="text-xs cursor-pointer">
                                                            {label}
                                                        </FieldLabel>
                                                    </Field>
                                                ))}
                                            </FieldGroup>
                                            {genConfig.include_symbols && (
                                                <Input
                                                    placeholder="自定义符号，留空使用默认集 !@#$%..."
                                                    value={genConfig.custom_symbols ?? ''}
                                                    onChange={e =>
                                                        setGenConfig({
                                                            ...genConfig,
                                                            custom_symbols: e.target.value || undefined,
                                                        })
                                                    }
                                                    className="h-7 text-xs"
                                                />
                                            )}
                                        </div>
                                        <div className="flex gap-2">
                                            <Button
                                                size="sm"
                                                className="flex-1"
                                                onClick={handleGenerate}
                                                disabled={isGenerating}
                                            >
                                                {isGenerating ? '生成中...' : '生成并填入'}
                                            </Button>
                                            <Button
                                                size="icon"
                                                variant="outline"
                                                onClick={handleGenerate}
                                                disabled={isGenerating}
                                                title="重新生成"
                                            >
                                                <RefreshCw
                                                    className={`h-3.5 w-3.5 ${isGenerating ? 'animate-spin' : ''}`}
                                                />
                                            </Button>
                                        </div>
                                    </div>
                                </PopoverContent>
                            </Popover>
                        </div>
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
