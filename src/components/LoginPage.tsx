import { useState, useMemo } from 'react';
import { toast } from 'sonner';
import { Button } from '@/components/ui/button';
import { Field, FieldError, FieldLabel } from './ui/field';
import { Label } from '@/components/ui/label';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { PasswordInput } from './PasswordInput';
import { useVaultStore } from '@/store/vaultStore';

export function LoginPage() {
    const [password, setPassword] = useState('');
    const [registerPassword, setRegisterPassword] = useState('');
    const [confirmPassword, setConfirmPassword] = useState('');
    const {
        registerVault,
        unlockVault,
        keyFilePath,
        vaultFilePath,
        isLoading,
        pickKeyFile,
        clearKeyFile,
        openVaultFilePicker,
        saveVaultFilePicker,
    } = useVaultStore();

    const passwordError = useMemo(() => {
        if (password.length > 0 && password.length < 8) return '密码长度至少 8 位';
        return '';
    }, [password]);

    const registerPasswordError = useMemo(() => {
        if (registerPassword.length > 0 && registerPassword.length < 8) return '密码长度至少 8 位';
        if (confirmPassword && registerPassword !== confirmPassword) return '两次输入的密码不一致';
        return '';
    }, [registerPassword, confirmPassword]);

    const handleRegister = async () => {
        if (!registerPassword || registerPassword.length < 8) {
            toast.error('请设置至少 8 位的密码');
            return;
        }
        if (registerPassword !== confirmPassword) {
            toast.error('两次输入的密码不一致');
            return;
        }

        await registerVault(registerPassword);
        setRegisterPassword('');
        setConfirmPassword('');
    };

    const handleLogin = async () => {
        if (!password) {
            toast.info('请输入主密码');
            return;
        }
        if (!keyFilePath) {
            toast.info('请先选择 .key 密钥文件');
            return;
        }

        await unlockVault(password);
        setPassword('');
    };

    return (
        <div className="min-h-screen flex items-center justify-center p-4">
            <Card className="w-full max-w-lg">
                <CardHeader>
                    <CardTitle className="text-center text-2xl">Tresor 密码管理器</CardTitle>
                </CardHeader>
                <CardContent>
                    <Tabs defaultValue="login" className="mt-2">
                        <TabsList className="grid w-full grid-cols-2">
                            <TabsTrigger value="login">解锁金库</TabsTrigger>
                            <TabsTrigger value="register">新建金库</TabsTrigger>
                        </TabsList>

                        <TabsContent value="login" className="space-y-4 mt-4">
                            <div className="space-y-2">
                                <Label className="font-medium">密钥文件</Label>
                                <div className="flex items-center gap-2 flex-wrap">
                                    <Button variant="outline" type="button" onClick={pickKeyFile} disabled={isLoading}>
                                        选择 .key 密钥文件
                                    </Button>
                                    {keyFilePath && (
                                        <>
                                            <span className="text-sm text-muted-foreground">已选择密钥文件</span>
                                            <Button
                                                variant="ghost"
                                                size="sm"
                                                type="button"
                                                onClick={clearKeyFile}
                                                disabled={isLoading}
                                            >
                                                移除
                                            </Button>
                                        </>
                                    )}
                                </div>
                            </div>

                            <div className="space-y-2">
                                <Label className="font-medium">金库文件</Label>
                                <div className="flex items-center gap-2 flex-wrap">
                                    <Button
                                        variant="outline"
                                        type="button"
                                        onClick={openVaultFilePicker}
                                        disabled={isLoading}
                                    >
                                        选择金库文件
                                    </Button>
                                    {vaultFilePath && (
                                        <span className="text-sm text-muted-foreground truncate max-w-xs">
                                            {vaultFilePath}
                                        </span>
                                    )}
                                </div>
                            </div>

                            <Field data-invalid={!!passwordError}>
                                <FieldLabel>主密码</FieldLabel>
                                <PasswordInput
                                    value={password}
                                    onChange={setPassword}
                                    placeholder="请输入主密码"
                                    disabled={isLoading}
                                />
                                <FieldError>{passwordError}</FieldError>
                            </Field>

                            <Button className="w-full" onClick={handleLogin} disabled={isLoading}>
                                {isLoading ? '解锁中...' : '解锁金库'}
                            </Button>
                        </TabsContent>

                        <TabsContent value="register" className="space-y-4 mt-4">
                            <Field data-invalid={!!registerPasswordError}>
                                <FieldLabel>设置主密码</FieldLabel>
                                <PasswordInput
                                    value={registerPassword}
                                    onChange={setRegisterPassword}
                                    placeholder="至少 8 位字符"
                                    disabled={isLoading}
                                />
                            </Field>

                            <Field data-invalid={!!registerPasswordError}>
                                <FieldLabel>确认主密码</FieldLabel>
                                <PasswordInput
                                    value={confirmPassword}
                                    onChange={setConfirmPassword}
                                    placeholder="再次输入密码"
                                    disabled={isLoading}
                                />
                                <FieldError>{registerPasswordError}</FieldError>
                            </Field>

                            <div className="space-y-2">
                                <Label className="font-medium">金库文件 (可选)</Label>
                                <div className="flex items-center gap-2 flex-wrap">
                                    <Button
                                        variant="outline"
                                        type="button"
                                        onClick={saveVaultFilePicker}
                                        disabled={isLoading}
                                    >
                                        设置金库文件位置
                                    </Button>
                                    {vaultFilePath && (
                                        <span className="text-sm text-muted-foreground truncate max-w-xs">
                                            {vaultFilePath}
                                        </span>
                                    )}
                                </div>
                            </div>

                            <Button className="w-full" onClick={handleRegister} disabled={isLoading}>
                                {isLoading ? '创建中...' : '创建新金库'}
                            </Button>
                        </TabsContent>
                    </Tabs>
                </CardContent>
            </Card>
        </div>
    );
}
