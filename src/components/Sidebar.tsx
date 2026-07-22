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
import { Plus, Lock, FolderOpen } from 'lucide-react';
import { VaultStatusAlert } from './VaultStatusAlert';
import { useVaultStore } from '@/store/vaultStore';
import { usePasswordStore } from '@/store/passwordStore';

interface SidebarProps {
    onNewEntry: () => void;
}

export function Sidebar({ onNewEntry }: SidebarProps) {
    const { isUnlocked, lockVault, vaultFilePath, openVaultFilePicker, saveVaultFilePicker } = useVaultStore();
    const { previewList, refreshPreviewList } = usePasswordStore();

    useEffect(() => {
        if (isUnlocked) {
            refreshPreviewList();
        }
    }, [isUnlocked, refreshPreviewList]);

    return (
        <ShadcnSidebar>
            <SidebarHeader>
                <div className="p-4">
                    <h1 className="text-xl font-bold">Tresor</h1>
                    <p className="text-sm text-muted-foreground">密码管理器</p>
                </div>
            </SidebarHeader>
            <SidebarContent>
                <SidebarGroup>
                    <SidebarGroupLabel>金库状态</SidebarGroupLabel>
                    <SidebarGroupContent>
                        <div className="px-2">
                            <VaultStatusAlert />
                        </div>
                    </SidebarGroupContent>
                </SidebarGroup>

                {isUnlocked && (
                    <>
                        <SidebarGroup>
                            <SidebarGroupLabel>金库信息</SidebarGroupLabel>
                            <SidebarGroupContent>
                                <div className="px-2">
                                    {vaultFilePath ? (
                                        <div className="text-xs text-muted-foreground truncate" title={vaultFilePath}>
                                            {vaultFilePath}
                                        </div>
                                    ) : (
                                        <div className="text-xs text-muted-foreground">未设置金库文件</div>
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
                                            <span>选择金库文件</span>
                                        </SidebarMenuButton>
                                    </SidebarMenuItem>
                                    <SidebarMenuItem>
                                        <SidebarMenuButton onClick={saveVaultFilePicker}>
                                            <FolderOpen className="h-4 w-4" />
                                            <span>更改金库文件位置</span>
                                        </SidebarMenuButton>
                                    </SidebarMenuItem>
                                </SidebarMenu>
                            </SidebarGroupContent>
                        </SidebarGroup>

                        <SidebarGroup>
                            <SidebarGroupLabel>密码记录 ({previewList.length})</SidebarGroupLabel>
                            <SidebarGroupContent>
                                <SidebarMenu>
                                    {previewList.slice(0, 10).map(entry => (
                                        <SidebarMenuItem key={entry.entry_id}>
                                            <SidebarMenuButton>
                                                <span className="truncate">{entry.url || '无网址'}</span>
                                            </SidebarMenuButton>
                                        </SidebarMenuItem>
                                    ))}
                                    {previewList.length > 10 && (
                                        <SidebarMenuItem>
                                            <span className="text-sm text-muted-foreground px-2">
                                                还有 {previewList.length - 10} 条...
                                            </span>
                                        </SidebarMenuItem>
                                    )}
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
                            锁定金库
                        </Button>
                    ) : (
                        <div className="text-sm text-muted-foreground text-center">请解锁金库</div>
                    )}
                </div>
            </SidebarFooter>
        </ShadcnSidebar>
    );
}
