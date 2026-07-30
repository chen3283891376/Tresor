import { useState, useEffect, useRef } from 'react';
import { Button } from '@/components/ui/button.tsx';
import { checkShortcutAvailable } from '@/api/tauriInvoke.ts';

interface ShortcutRecorderProps {
    value: string;
    onChange: (value: string) => void;
}

const MOD_ORDER = ['Ctrl', 'Alt', 'Shift', 'Win'];

function getKeyDisplay(code: string): string | null {
    if (code.startsWith('Key')) return code.slice(3);
    if (code.startsWith('Digit')) return code.slice(5);
    if (code === 'Space') return '空格';
    if (code === 'Enter') return '回车';
    if (code === 'Tab') return 'Tab';
    if (code === 'Escape') return 'Esc';
    if (code === 'Backspace') return '退格';
    if (code === 'Delete') return '删除';
    if (code === 'ArrowUp') return '↑';
    if (code === 'ArrowDown') return '↓';
    if (code === 'ArrowLeft') return '←';
    if (code === 'ArrowRight') return '→';
    if (code.startsWith('F')) return code;
    if (code === 'Comma') return ',';
    if (code === 'Period') return '.';
    if (code === 'Minus') return '-';
    if (code === 'Equal') return '=';
    if (code === 'Semicolon') return ';';
    if (code === 'Quote') return "'";
    if (code === 'Backslash') return '\\';
    if (code === 'Slash') return '/';
    if (code === 'BracketLeft') return '[';
    if (code === 'BracketRight') return ']';
    if (code === 'Backquote') return '`';
    return code;
}

function buildCombo(e: KeyboardEvent): string {
    const mods: string[] = [];
    if (e.ctrlKey) mods.push('Ctrl');
    if (e.altKey) mods.push('Alt');
    if (e.shiftKey) mods.push('Shift');
    if (e.metaKey) mods.push('Win');
    const keyPart = e.code.replace(/^Key/, '').replace(/^Digit/, '');
    return [...mods, keyPart].join('+');
}

export function ShortcutRecorder({ value, onChange }: ShortcutRecorderProps) {
    const [recording, setRecording] = useState(false);
    const [combo, setCombo] = useState<string | null>(null);
    const [available, setAvailable] = useState<boolean | null>(null);
    const [checking, setChecking] = useState(false);
    const ref = useRef<HTMLButtonElement>(null);

    useEffect(() => {
        if (!recording) return;

        const onKeyDown = (e: KeyboardEvent) => {
            if (e.code === 'Escape') {
                setRecording(false);
                setCombo(null);
                setAvailable(null);
                return;
            }

            if (e.repeat) return;

            const isMod = e.code.startsWith('Control') || e.code.startsWith('Shift') || e.code.startsWith('Alt') || e.code.startsWith('Meta');
            if (isMod) return;

            e.preventDefault();
            e.stopPropagation();

            const newCombo = buildCombo(e);
            setCombo(newCombo);
            setChecking(true);
            setAvailable(null);

            checkShortcutAvailable(newCombo).then(result => {
                setAvailable(result);
                setChecking(false);
            });
        };

        window.addEventListener('keydown', onKeyDown, true);

        return () => {
            window.removeEventListener('keydown', onKeyDown, true);
        };
    }, [recording]);

    const handleStart = () => {
        setRecording(true);
        setCombo(null);
        setAvailable(null);
    };

    const handleConfirm = () => {
        if (!combo) return;
        onChange(combo);
        setRecording(false);
        setCombo(null);
        setAvailable(null);
        ref.current?.focus();
    };

    const handleCancel = () => {
        setRecording(false);
        setCombo(null);
        setAvailable(null);
    };

    const comboDisplay = combo
        ? combo
              .split('+')
              .map(part => (MOD_ORDER.includes(part) ? part : getKeyDisplay(part) ?? part))
              .join(' + ')
        : null;

    return (
        <div className="flex items-center gap-2">
            {recording ? (
                <div className="flex items-center gap-2 flex-1">
                    <div className="flex-1 px-3 py-2 border rounded-md bg-muted font-mono text-sm min-h-[36px] flex items-center">
                        {combo ? (
                            <span>{comboDisplay}</span>
                        ) : (
                            <span className="text-muted-foreground">按下快捷键...</span>
                        )}
                    </div>
                    {checking ? (
                        <span className="text-xs text-muted-foreground animate-pulse">检测中...</span>
                    ) : available === true ? (
                        <span className="text-xs text-green-600">可用</span>
                    ) : available === false ? (
                        <span className="text-xs text-red-500">已被占用</span>
                    ) : null}
                    <Button size="sm" onClick={handleConfirm} disabled={!combo}>确定</Button>
                    <Button size="sm" variant="ghost" onClick={handleCancel}>取消</Button>
                </div>
            ) : (
                <Button
                    ref={ref}
                    variant="outline"
                    size="sm"
                    className="font-mono text-xs"
                    onClick={handleStart}
                >
                    {value || '未设置'}
                </Button>
            )}
        </div>
    );
}
