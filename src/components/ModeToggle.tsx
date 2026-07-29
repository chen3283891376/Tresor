import { Sun, Moon } from 'lucide-react';
import { Button } from '@/components/ui/button';
import {
    DropdownMenu,
    DropdownMenuContent,
    DropdownMenuItem,
    DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu';
import { useTheme } from '@/components/ThemeProvider';

export function ModeToggle() {
    const { setTheme } = useTheme();

    return (
        <DropdownMenu>
            <DropdownMenuTrigger
                render={
                    <Button variant="outline" size="icon" className={'my-auto'}>
                        <Sun className="h-4 w-4 dark:hidden" />
                        <Moon className="h-4 w-4 hidden dark:inline-block" />
                        <span className="sr-only">切换主题</span>
                    </Button>
                }
            />
            <DropdownMenuContent align="end">
                <DropdownMenuItem onClick={() => setTheme('light')}>亮色</DropdownMenuItem>
                <DropdownMenuItem onClick={() => setTheme('dark')}>暗色</DropdownMenuItem>
                <DropdownMenuItem onClick={() => setTheme('system')}>跟随系统</DropdownMenuItem>
            </DropdownMenuContent>
        </DropdownMenu>
    );
}
