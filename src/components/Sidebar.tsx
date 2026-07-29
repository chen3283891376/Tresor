import { useEffect } from 'react';
import {
    Sidebar as ShadcnSidebar,
    SidebarContent,
    SidebarHeader,
    SidebarFooter,
    SidebarGroup,
    SidebarGroupContent,
    SidebarGroupLabel,
    SidebarMenu,
    SidebarMenuItem,
    SidebarMenuButton,
} from '@/components/ui/sidebar';
import { Button } from '@/components/ui/button';
import { Plus, Lock, FolderOpen, KeyRound, Globe } from 'lucide-react';
import { VaultStatusAlert } from './VaultStatusAlert';
import { useVaultStore } from '@/store/vaultStore';
import { usePasswordStore } from '@/store/passwordStore';
import { usePageStore } from '@/store/pageStore.ts';
import { ModeToggle } from '@/components/ModeToggle.tsx';

interface SidebarProps {
    onNewEntry: () => void;
}

export function Sidebar({ onNewEntry }: SidebarProps) {
    const { isUnlocked, lockVault, vaultFilePath, openVaultFilePicker, saveVaultFilePicker } = useVaultStore();
    const { refreshPreviewList } = usePasswordStore();
    const { currentPage, setCurrentPage } = usePageStore();

    useEffect(() => {
        if (isUnlocked) {
            refreshPreviewList().then();
        }
    }, [isUnlocked, refreshPreviewList]);

    return (
        <ShadcnSidebar>
            <SidebarHeader>
                <div className={'flex justify-between gap-2'}>
                    <div className="p-4">
                        <h1 className="text-xl font-bold">Tresor</h1>
                        <p className="text-sm text-muted-foreground">密码管理器</p>
                    </div>
                    <ModeToggle />
                </div>
            </SidebarHeader>
            <SidebarContent>
                <SidebarGroup>
                    <SidebarGroupLabel>Tresor状态</SidebarGroupLabel>
                    <SidebarGroupContent>
                        <div className="px-2">
                            <VaultStatusAlert />
                        </div>
                    </SidebarGroupContent>
                </SidebarGroup>

                {isUnlocked && (
                    <>
                        <SidebarGroup>
                            <SidebarGroupLabel>导航</SidebarGroupLabel>
                            <SidebarGroupContent>
                                <SidebarMenu>
                                    <SidebarMenuItem>
                                        <SidebarMenuButton
                                            onClick={() => setCurrentPage('passwords')}
                                            isActive={currentPage === 'passwords'}
                                        >
                                            <Globe className="h-4 w-4" />
                                            <span>密码</span>
                                        </SidebarMenuButton>
                                    </SidebarMenuItem>
                                    <SidebarMenuItem>
                                        <SidebarMenuButton
                                            onClick={() => setCurrentPage('2fa')}
                                            isActive={currentPage === '2fa'}
                                        >
                                            <KeyRound className="h-4 w-4" />
                                            <span>2FA</span>
                                        </SidebarMenuButton>
                                    </SidebarMenuItem>
                                </SidebarMenu>
                            </SidebarGroupContent>
                        </SidebarGroup>

                        <SidebarGroup>
                            <SidebarGroupLabel>Tresor信息</SidebarGroupLabel>
                            <SidebarGroupContent>
                                <div className="px-2">
                                    {vaultFilePath ? (
                                        <div className="text-xs text-muted-foreground truncate" title={vaultFilePath}>
                                            {vaultFilePath}
                                        </div>
                                    ) : (
                                        <div className="text-xs text-muted-foreground">未设置Tresor文件</div>
                                    )}
                                </div>
                            </SidebarGroupContent>
                        </SidebarGroup>

                        <SidebarGroup>
                            <SidebarGroupLabel>操作</SidebarGroupLabel>
                            <SidebarGroupContent>
                                <SidebarMenu>
                                    <SidebarMenuItem>
                                        <SidebarMenuButton onClick={onNewEntry}>
                                            <Plus className="h-4 w-4" />
                                            <span>新建密码记录</span>
                                        </SidebarMenuButton>
                                    </SidebarMenuItem>
                                    <SidebarMenuItem>
                                        <SidebarMenuButton onClick={openVaultFilePicker}>
                                            <FolderOpen className="h-4 w-4" />
                                            <span>选择数据库文件</span>
                                        </SidebarMenuButton>
                                    </SidebarMenuItem>
                                    <SidebarMenuItem>
                                        <SidebarMenuButton onClick={saveVaultFilePicker}>
                                            <FolderOpen className="h-4 w-4" />
                                            <span>更改数据文件位置</span>
                                        </SidebarMenuButton>
                                    </SidebarMenuItem>
                                </SidebarMenu>
                            </SidebarGroupContent>
                        </SidebarGroup>
                    </>
                )}
            </SidebarContent>
            <SidebarFooter>
                <div className="p-4">
                    {isUnlocked ? (
                        <Button className="w-full" onClick={lockVault}>
                            <Lock className="h-4 w-4 mr-2" />
                            锁定Tresor
                        </Button>
                    ) : (
                        <div className="text-sm text-muted-foreground text-center">请解锁Tresor</div>
                    )}
                </div>
            </SidebarFooter>
        </ShadcnSidebar>
    );
}
