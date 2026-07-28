import React from 'react';
import ReactDOM from 'react-dom/client';
import App from './App';
import '@/index.css';
import { Toaster } from '@/components/ui/sonner.tsx';
import { ThemeProvider } from '@/components/ThemeProvider.tsx';

ReactDOM.createRoot(document.getElementById('root') as HTMLElement).render(
    <React.StrictMode>
        <ThemeProvider defaultTheme="system" storageKey="app-theme">
            <Toaster />
            <App />
        </ThemeProvider>
    </React.StrictMode>,
);
