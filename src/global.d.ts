// src/app.d.ts
/// <reference types="@sveltejs/kit" />

declare global {
    interface Window {
        ethereum?: {
            isMetaMask?: boolean;
            isCoinbaseWallet?: boolean;
            isTrust?: boolean;
            isPhantom?: boolean;
            request: (args: { method: string; params?: any[] }) => Promise<any>;
            on: (event: string, callback: (...args: any[]) => void) => void;
            removeListener: (event: string, callback: (...args: any[]) => void) => void;
            selectedAddress?: string;
            chainId?: string;
        };
    }
}

export {};