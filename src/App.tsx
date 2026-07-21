import { LoginPage } from '@/components/LoginPage.tsx';
import React from 'react';

function App() {
    const [loginSuccess, setLoginSuccess] = React.useState(false);

    return loginSuccess ? <div>登录成功</div> : <LoginPage onLoginSuccess={() => setLoginSuccess(true)} />;
}

export default App;
