import { Button } from '@/components/ui/button';
import { useVaultStore } from '@/store/vaultStore';

interface KeyFileSelectBtnProps {
    disabled?: boolean;
}

export function KeyFileSelectBtn({ disabled }: KeyFileSelectBtnProps) {
    const { keyFilePath, pickKeyFile, clearKeyFile } = useVaultStore();

    return (
        <div className="flex items-center gap-2 flex-wrap">
            <Button variant="outline" type="button" onClick={pickKeyFile} disabled={disabled}>
                选择 .key 密钥文件
            </Button>
            {keyFilePath && (
                <>
                    <span className="text-sm text-muted-foreground">已选择密钥文件</span>
                    <Button variant="ghost" size="sm" type="button" onClick={clearKeyFile} disabled={disabled}>
                        移除
                    </Button>
                </>
            )}
        </div>
    );
}
