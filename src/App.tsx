import { Toaster } from '@/components/ui/sonner';
import { TooltipProvider } from '@/components/ui/tooltip';
import { LoginPage } from '@/components/LoginPage';
import { VaultUnlockedView } from '@/components/VaultUnlockedView';
import { useVaultStore } from '@/store/vaultStore';

function App() {
    const { isUnlocked } = useVaultStore();

    return (
        <TooltipProvider>
            {isUnlocked ? <VaultUnlockedView /> : <LoginPage />}
            <Toaster />
        </TooltipProvider>
    );
}

export default App;
