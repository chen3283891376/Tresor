import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert';
import { Lock, Unlock } from 'lucide-react';
import { useVaultStore } from '@/store/vaultStore';

export function VaultStatusAlert() {
    const { isUnlocked } = useVaultStore();

    return (
        <Alert variant={isUnlocked ? 'default' : 'destructive'}>
            {isUnlocked ? <Unlock className="h-4 w-4" /> : <Lock className="h-4 w-4" />}
            <AlertTitle>{isUnlocked ? 'Tresor已解锁' : 'Tresor已锁定'}</AlertTitle>
            <AlertDescription>
                {isUnlocked ? '您现在可以管理密码记录' : '请先解锁Tresor以访问您的密码'}
            </AlertDescription>
        </Alert>
    );
}
