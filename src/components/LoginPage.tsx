import { useState, useMemo } from 'react';
import { toast } from 'sonner';

import { Dialog, DialogContent, DialogHeader, DialogTitle } from '@/components/ui/dialog';
import { Input } from '@/components/ui/input';
import { Button } from '@/components/ui/button';
import { Field, FieldError, FieldLabel } from './ui/field';
import { Label } from '@/components/ui/label';
import { invoke } from '@tauri-apps/api/core';

export function LoginPage({ onLoginSuccess }: { onLoginSuccess: () => void }) {
    const [password, setPassword] = useState('');
    const [isRegister, setIsRegister] = useState(false);
    const [loading, setLoading] = useState(false);
    const [hasSelectedKey, setHasSelectedKey] = useState<boolean>(false);
    const [fileErrMsg, setFileErrMsg] = useState('');

    const passwordError = useMemo(() => {
        if (password.length > 0 && password.length < 8) return '密码长度至少 8 位';
        return '';
    }, [password]);

    const pickKeyFile = async () => {
        setFileErrMsg('');
        try {
            const ok: boolean = await invoke('open_key_file_picker');
            setHasSelectedKey(ok);
            if (!ok) setFileErrMsg('未选择有效 .key 密钥文件');
        } catch (err: any) {
            setFileErrMsg(`文件选择失败: ${err}`);
            setHasSelectedKey(false);
        }
    };

    const clearSelectedKey = async () => {
        await invoke('clear_stored_key_path');
        setHasSelectedKey(false);
        setFileErrMsg('');
    };

    const handleRegister = async () => {
        if (!password) {
            toast.info('请设置主密码');
            return;
        }
        if (password.length < 8) {
            toast.error('密码长度至少 8 位');
            return;
        }

        setLoading(true);
        try {
            await invoke('register_vault', { userPwd: password });
            toast.success('密码库创建完成！已弹出窗口保存密钥到U盘，请妥善保管 vault.key');
            setPassword('');
        } catch (err: any) {
            if (String(err).includes('用户取消密钥保存')) {
                toast.info('未保存密钥，创建流程已取消');
            } else {
                toast.error(`创建密码库失败: ${err}`);
            }
        } finally {
            setLoading(false);
        }
    };

    const handleLogin = async () => {
        if (!password) {
            toast.info('请输入主密码');
            return;
        }
        if (!hasSelectedKey) {
            toast.info('必须选择注册时保存到U盘的 .key 私钥文件');
            return;
        }

        setLoading(true);
        try {
            const ok = await invoke<boolean>('unlock_vault', { userPwd: password });
            if (ok) {
                toast.success('密码库解锁成功');
                onLoginSuccess();
            } else {
                setFileErrMsg('解锁失败，请检查主密码和U盘密钥分片是否正确');
            }
        } catch (err: any) {
            toast.error(`解锁失败: ${err}`);
        } finally {
            setPassword('');
            setLoading(false);
        }
    };

    // 切换注册/登录时，清空密钥选择状态
    const toggleMode = () => {
        setIsRegister(!isRegister);
        clearSelectedKey().then();
    };

    return (
        <Dialog open={true} modal>
            <DialogContent className="sm:max-w-lg">
                <DialogHeader>
                    <DialogTitle>
                        {isRegister ? '创建全新密码库（仅设置主密码）' : '解锁密码库（主密码+U盘密钥分片）'}
                    </DialogTitle>
                </DialogHeader>

                <form onSubmit={e => e.preventDefault()} autoComplete="on">
                    <div className="space-y-5 py-4">
                        <Field data-invalid={!!passwordError}>
                            <FieldLabel>主密码</FieldLabel>
                            <Input
                                type="password"
                                placeholder="至少8位字符"
                                autoComplete={isRegister ? 'new-password' : 'current-password'}
                                value={password}
                                onChange={e => setPassword(e.target.value)}
                                disabled={loading}
                            />
                            <FieldError>{passwordError}</FieldError>
                        </Field>

                        {!isRegister && (
                            <div className="space-y-2">
                                <Label className="font-medium">U盘私钥分片文件</Label>
                                <div className="flex items-center gap-2 flex-wrap">
                                    <Button variant="outline" type="button" onClick={pickKeyFile} disabled={loading}>
                                        打开系统文件选择器
                                    </Button>
                                    {hasSelectedKey && (
                                        <div className="flex items-center gap-2">
                                            <span className="text-sm text-slate-600">已选择密钥文件</span>
                                            <Button
                                                variant="ghost"
                                                size="sm"
                                                type="button"
                                                onClick={clearSelectedKey}
                                                disabled={loading}
                                            >
                                                移除
                                            </Button>
                                        </div>
                                    )}
                                </div>
                                {fileErrMsg && <p className="text-sm text-destructive">{fileErrMsg}</p>}
                            </div>
                        )}

                        <Button
                            className="w-full"
                            onClick={isRegister ? handleRegister : handleLogin}
                            disabled={loading}
                        >
                            {loading ? '处理中...' : isRegister ? '创建密码库并导出密钥分片' : '解锁密码库'}
                        </Button>

                        <Button variant="ghost" className="w-full text-sm" onClick={toggleMode} disabled={loading}>
                            {isRegister ? '已有密码库？切换登录' : '首次使用？注册新密码库'}
                        </Button>
                    </div>
                </form>
            </DialogContent>
        </Dialog>
    );
}
