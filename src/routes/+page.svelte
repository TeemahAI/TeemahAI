<script lang="ts">
    import { onMount, afterUpdate } from 'svelte';
    import { Bot, Send, Zap, Shield, Clock, Server, Wallet, LogOut, Coins, AlertCircle, ExternalLink } from 'lucide-svelte';
    import { BrowserProvider, ethers } from 'ethers';
    
    // Ethereum provider interface
    interface EthereumProvider {
        isMetaMask?: boolean;
        isCoinbaseWallet?: boolean;
        isTrust?: boolean;
        isPhantom?: boolean;
        request: (args: { method: string; params?: any[] }) => Promise<any>;
        on: (event: string, callback: (...args: any[]) => void) => void;
        removeListener: (event: string, callback: (...args: any[]) => void) => void;
        selectedAddress?: string;
        chainId?: string;
    }
    
    declare global {
        interface Window {
            ethereum?: EthereumProvider;
            lastBackendData?: any;
        }
    }
    
    // State
    let messages = [
        {
            id: 1,
            role: 'assistant' as const,
            content: '🚀 Welcome to Teemah AI\n\nConnect your wallet to start making on-chain transactions!',
            timestamp: new Date(),
            isTyping: false,
            isComplete: true,
            isThinking: false
        }
    ];
    let input = '';
    let isProcessing = false;
    let backendConnected = false;
    let isTestingConnection = false;
    let typingMessageId: number | null = null;
    let typingTimeout: any = null;
    let isTypingComplete = false;
    
    // Wallet state
    let walletConnected = false;
    let walletAddress = '';
    let walletBalance = '0';
    let chainId = 97;
    let walletType = '';
    let isConnectingWallet = false;

    // Agent state
    let agentInitialized = false;
    let isInitializingAgent = false;

    // Transaction state
    let pendingTransaction: any = null;
    let isSigningTransaction = false;

    let chatContainer: HTMLDivElement;

    afterUpdate(() => {
        if (chatContainer) {
            chatContainer.scrollTop = chatContainer.scrollHeight;
        }
    });
    
    onMount(async () => {
        await checkBackendConnection();
        if (backendConnected) {
            await initializeAgent();
        }
        await checkWalletConnection();
        setupWalletListeners();
    });

    // ============ TYPING EFFECT FUNCTIONS ============
    function startTypingEffect(messageId: number) {
        typingMessageId = messageId;
        isTypingComplete = false;
        const messageIndex = messages.findIndex(m => m.id === messageId);
        if (messageIndex !== -1) {
            messages[messageIndex].isTyping = true;
            messages[messageIndex].isComplete = false;
            messages = [...messages];
        }
    }

    function stopTypingEffect(messageId: number) {
        typingMessageId = null;
        isTypingComplete = true;
        const messageIndex = messages.findIndex(m => m.id === messageId);
        if (messageIndex !== -1) {
            messages[messageIndex].isTyping = false;
            messages[messageIndex].isComplete = true;
            messages = [...messages];
        }
    }

    function cancelTyping() {
        if (typingTimeout) {
            clearTimeout(typingTimeout);
            typingTimeout = null;
        }
        if (typingMessageId !== null) {
            stopTypingEffect(typingMessageId);
        }
    }

    function typeMessageCharacter(messageId: number, fullContent: string, currentIndex: number) {
        if (currentIndex >= fullContent.length) {
            stopTypingEffect(messageId);
            return;
        }
        
        const messageIndex = messages.findIndex(m => m.id === messageId);
        if (messageIndex === -1) return;
        
        // Add next character
        const nextChar = fullContent[currentIndex];
        messages[messageIndex].content += nextChar;
        messages = [...messages];
        
        // Schedule next character with variable delay
        let delay = 30; // Slower base delay for better readability
        
        // Add longer delays for punctuation and line breaks
        if (nextChar === '.' || nextChar === '!' || nextChar === '?') {
            delay = 300; // Longer pause after sentences
        } else if (nextChar === ',' || nextChar === ';') {
            delay = 150; // Pause after commas
        } else if (nextChar === '\n') {
            delay = 200; // Pause after line breaks
        } else if (nextChar === ' ') {
            delay = 50; // Slight pause after spaces
        }
        
        // Random variation for natural feel
        delay += Math.random() * 30;
        
        typingTimeout = setTimeout(() => {
            typeMessageCharacter(messageId, fullContent, currentIndex + 1);
        }, delay);
    }

    function simulateTyping(messageId: number, fullContent: string) {
        startTypingEffect(messageId);
        
        // Clear any existing timeout
        if (typingTimeout) {
            clearTimeout(typingTimeout);
        }
        
        // Start typing after a short delay
        setTimeout(() => {
            const messageIndex = messages.findIndex(m => m.id === messageId);
            if (messageIndex !== -1) {
                messages[messageIndex].content = '';
                messages = [...messages];
            }
            
            // Start typing characters
            typeMessageCharacter(messageId, fullContent, 0);
        }, 500); // Slight delay before starting to type
    }

    // ============ MESSAGE FUNCTIONS ============
    function addUserMessage(content: string) {
        const newId = messages.length + 1;
        messages = [...messages, {
            id: newId,
            role: 'user' as const,
            content,
            timestamp: new Date(),
            isTyping: false,
            isComplete: true,
            isThinking: false
        }];
    }

    function addAssistantMessage(content: string) {
        const newId = messages.length + 1;
        messages = [...messages, {
            id: newId,
            role: 'assistant' as const,
            content: '',
            timestamp: new Date(),
            isTyping: true,
            isComplete: false,
            isThinking: false
        }];
        
        // Start typing effect
        setTimeout(() => {
            simulateTyping(newId, content);
        }, 300);
        
        return newId;
    }

    function addSystemMessage(content: string) {
        const newId = messages.length + 1;
        messages = [...messages, {
            id: newId,
            role: 'assistant' as const,
            content: `⚙️ ${content}`,
            timestamp: new Date(),
            isTyping: false,
            isComplete: true,
            isThinking: false
        }];
    }

    function addThinkingIndicator() {
        const thinkingId = messages.length + 1;
        messages = [...messages, {
            id: thinkingId,
            role: 'assistant' as const,
            content: '🤔 Thinking...',
            timestamp: new Date(),
            isTyping: false,
            isComplete: true,
            isThinking: true
        }];
        return thinkingId;
    }

    function replaceThinkingWithResponse(thinkingId: number, response: string) {
        const thinkingIndex = messages.findIndex(m => m.id === thinkingId);
        if (thinkingIndex !== -1) {
            messages.splice(thinkingIndex, 1);
            messages = [...messages];
            addAssistantMessage(response);
        }
    }

    // ============ UTILITY FUNCTIONS ============
    function formatMessageContent(content: string): string {
        if (!content) return '';
        
        // Simple formatting - no bold headers, just normal text
        return content
            .replace(/\n/g, '<br>')
            .replace(/`([^`]+)`/g, '<code class="bg-gray-700 px-1.5 py-0.5 rounded font-mono text-sm">$1</code>')
            .replace(/\*\*([^*]+)\*\*/g, '$1') // Remove bold
            .replace(/\*([^*]+)\*/g, '$1') // Remove italic
            .replace(/^# (.*$)/gm, '<div class="text-base mt-2 mb-1">$1</div>')
            .replace(/^## (.*$)/gm, '<div class="text-base mt-2 mb-1">$1</div>')
            .replace(/^### (.*$)/gm, '<div class="text-base mt-2 mb-1">$1</div>')
            .replace(/\[([^\]]+)\]\(([^)]+)\)/g, '<a href="$2" target="_blank" class="text-blue-400 hover:text-blue-300 underline">$1</a>');
    }

    function clearPendingTransaction() {
        pendingTransaction = null;
        if (window.lastBackendData) {
            window.lastBackendData = null;
        }
        console.log('🧹 Cleared pending transaction');
    }

    // ============ WALLET FUNCTIONS ============
    function setupWalletListeners() {
        if (window.ethereum) {
            window.ethereum.on('accountsChanged', (accounts: string[]) => {
                console.log('Accounts changed:', accounts);
                if (accounts.length === 0) {
                    handleWalletDisconnect();
                } else {
                    walletAddress = accounts[0];
                    addSystemMessage(`🔄 Account changed: ${shortenAddress(walletAddress)}`);
                    updateWalletInfo();
                }
            });
            
            window.ethereum.on('chainChanged', (chainIdHex: string) => {
                console.log('Chain changed:', chainIdHex);
                const newChainId = parseInt(chainIdHex, 16);
                chainId = newChainId;
                addSystemMessage(`🔗 Network changed: Chain ${newChainId}`);
                updateWalletInfo();
            });
            
            window.ethereum.on('disconnect', (error: any) => {
                console.log('Wallet disconnected:', error);
                handleWalletDisconnect();
            });
        }
    }
    
    function handleWalletDisconnect() {
        walletConnected = false;
        walletAddress = '';
        walletBalance = '0';
        walletType = '';
        addSystemMessage('🔌 Wallet disconnected');
    }
    
    async function updateWalletInfo() {
        await checkWalletConnection();
        await updateWalletBalance();
    }
    
    async function checkBackendConnection() {
        isTestingConnection = true;
        try {
            const response = await fetch('http://localhost:3001/api/health');
            const data = await response.json();
            backendConnected = response.ok;
            if (backendConnected) {
                addSystemMessage('✅ Backend connected successfully!');
            }
        } catch (error: any) {
            backendConnected = false;
            addSystemMessage('❌ Could not connect to backend.');
        } finally {
            isTestingConnection = false;
        }
    }
    
    async function initializeAgent() {
        if (isInitializingAgent) return;
        
        isInitializingAgent = true;
        addSystemMessage('🤖 Initializing AI Agent...');
        
        try {
            const response = await fetch('http://localhost:3001/api/agent/initialize', {
                method: 'POST',
                headers: { 
                    'Content-Type': 'application/json',
                    'Accept': 'application/json'
                },
                body: JSON.stringify({ 
                    deepseek_api_key: "sk-fc91eb7c1b6f4df9acc85a3e2b3811b0",
                    rpc_url: "https://data-seed-prebsc-1-s1.binance.org:8545",
                    contract_address: "0xE7392b8ee167980602d38225674bB377De5Fe287"
                })
            });
            
            if (response.ok) {
                const data = await response.json();
                agentInitialized = data.success;
                if (agentInitialized) {
                    addSystemMessage(`✅ ${data.message}`);
                } else {
                    addSystemMessage(`❌ Agent initialization failed: ${data.message}`);
                }
            } else {
                const errorText = await response.text();
                addSystemMessage(`❌ Failed to initialize agent: ${errorText}`);
            }
        } catch (error: any) {
            addSystemMessage(`❌ Agent initialization error: ${error.message}`);
        } finally {
            isInitializingAgent = false;
        }
    }
    
    async function checkAgentStatus() {
        try {
            const response = await fetch('http://localhost:3001/api/agent/status');
            if (response.ok) {
                const data = await response.json();
                agentInitialized = data.initialized;
                return data;
            }
        } catch (error) {
            console.log('Failed to check agent status:', error);
        }
        return null;
    }
    
    async function checkWalletConnection() {
        try {
            const response = await fetch('http://localhost:3001/api/wallet/status');
            const data = await response.json();
            walletConnected = data.connected;
            if (walletConnected) {
                walletAddress = data.address || '';
                walletBalance = data.balance_eth || '0';
                chainId = data.chain_id || 0;
                walletType = data.wallet_type || 'Unknown';
            }
        } catch (error) {
            console.log('No wallet connected or backend error');
        }
    }

    async function connectMetaMask() {
        if (typeof window.ethereum === 'undefined') {
            addAssistantMessage('❌ MetaMask not detected.');
            return;
        }
        
        isConnectingWallet = true;
        try {
            const provider = new BrowserProvider(window.ethereum);
            
            const accounts = await window.ethereum.request({ 
                method: 'eth_requestAccounts' 
            });
            
            if (accounts.length === 0) {
                throw new Error('No accounts found');
            }
            
            const address = accounts[0];
            const chainIdHex = await window.ethereum.request({ 
                method: 'eth_chainId' 
            });
            const chainId = parseInt(chainIdHex, 16);
            
            const response = await fetch('http://localhost:3001/api/wallet/connect', {
                method: 'POST',
                headers: { 
                    'Content-Type': 'application/json',
                    'Accept': 'application/json'
                },
                body: JSON.stringify({ 
                    address,
                    chain_id: chainId,
                    wallet_type: 'MetaMask'
                })
            });
            
            if (response.ok) {
                const data = await response.json();
                walletConnected = true;
                walletAddress = address;
                walletType = 'MetaMask';
                
                addSystemMessage(`✅ MetaMask connected: ${shortenAddress(address)} on chain ${chainId}`);
                
                const balance = await provider.getBalance(address);
                walletBalance = (Number(balance) / 1e18).toFixed(6);
                
            }
        } catch (error: any) {
            console.error('Failed to connect wallet:', error);
            addAssistantMessage(`❌ Failed to connect wallet: ${error.message}`);
        } finally {
            isConnectingWallet = false;
        }
    }

    async function connectWalletConnect() {
        addAssistantMessage('🔗 WalletConnect integration coming soon!');
    }

    async function disconnectWallet() {
        try {
            const response = await fetch('http://localhost:3001/api/wallet/disconnect', {
                method: 'POST',
                headers: { 
                    'Content-Type': 'application/json',
                    'Accept': 'application/json'
                }
            });
            
            if (response.ok) {
                walletConnected = false;
                walletAddress = '';
                walletBalance = '0';
                walletType = '';
                addSystemMessage('🔌 Wallet disconnected');
            }
        } catch (error) {
            console.error('Failed to disconnect wallet:', error);
        }
    }

    async function updateWalletBalance() {
        try {
            const response = await fetch('http://localhost:3001/api/wallet/balance');
            const data = await response.json();
            if (data.success && data.balance_eth) {
                walletBalance = data.balance_eth;
            }
        } catch (error) {
            console.error('Failed to get balance:', error);
        }
    }

    // ============ TRANSACTION FUNCTIONS ============
    async function signTransaction(transactionData: any, backendData: any = null): Promise<string | null> {
        if (!window.ethereum || !walletConnected) {
            addAssistantMessage('❌ Please connect your wallet first.');
            return null;
        }
        
        isSigningTransaction = true;
        
        try {
            const provider = new BrowserProvider(window.ethereum);
            const signer = await provider.getSigner();
            
            const isCreateProject = backendData?.action === 'create_project';
            let tx;
            
            if (isCreateProject && backendData?.parameters) {
                const params = backendData.parameters;
                const contractInterface = new ethers.Interface([
                    'function createProjectWithTokenViaTelegram(address creator, string token_name, string token_symbol, uint8 token_decimals, uint256 initial_supply) external returns (address)'
                ]);
                
                try {
                    const creatorAddress = walletAddress;
                    const tokenDecimals = parseInt(params.token_decimals);
                    const initialSupply = ethers.toBigInt(params.initial_supply);
                    
                    const encodedData = contractInterface.encodeFunctionData('createProjectWithTokenViaTelegram', [
                        creatorAddress,
                        params.token_name,
                        params.token_symbol,
                        tokenDecimals,
                        initialSupply
                    ]);
                    
                    tx = {
                        to: transactionData.to,
                        data: encodedData,
                        value: ethers.parseEther(transactionData.value || "0"),
                        chainId: transactionData.chain_id,
                    };
                    
                } catch (encodingError) {
                    throw new Error(`Failed to encode contract call: ${encodingError.message}`);
                }
            } else {
                // Simple test transaction
                tx = {
                    to: "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045",
                    value: ethers.parseEther("0"),
                    chainId: transactionData.chain_id,
                };
            }
            
            addSystemMessage('⏳ Opening MetaMask... Please sign the transaction.');
            
            // Wait a moment for user to see the message
            await new Promise(resolve => setTimeout(resolve, 1000));
            
            // Estimate gas
            try {
                const gasEstimate = await provider.estimateGas(tx);
                tx.gasLimit = gasEstimate * 120n / 100n;
            } catch (gasError) {
                console.warn('Gas estimation failed:', gasError.message);
                tx.gasLimit = 8000000n;
            }
            
            // Get fee data
            try {
                const feeData = await provider.getFeeData();
                if (feeData.gasPrice) {
                    tx.gasPrice = feeData.gasPrice;
                } else if (feeData.maxFeePerGas && feeData.maxPriorityFeePerGas) {
                    tx.maxFeePerGas = feeData.maxFeePerGas;
                    tx.maxPriorityFeePerGas = feeData.maxPriorityFeePerGas;
                }
            } catch (feeError) {
                console.warn('Fee data fetch failed:', feeError.message);
            }
            
            // Send transaction
            const txResponse = await signer.sendTransaction(tx);
            addSystemMessage(`✅ Transaction sent! Hash: ${txResponse.hash}\n⏳ Waiting for confirmation...`);
            
            const receipt = await txResponse.wait();
            
            if (receipt && receipt.hash) {
                if (receipt.status === 1) {
                    addSystemMessage(`🎉 Transaction confirmed! Hash: ${receipt.hash}`);
                } else {
                    addSystemMessage(`⚠️ Transaction failed! Hash: ${receipt.hash} (Status: ${receipt.status})`);
                    throw new Error(`Transaction failed with status ${receipt.status}`);
                }
                
                isSigningTransaction = false;
                clearPendingTransaction();
                return receipt.hash;
            }
            
            isSigningTransaction = false;
            return null;
            
        } catch (error: any) {
            console.error('Transaction error:', error);
            
            isSigningTransaction = false;
            clearPendingTransaction();
            
            // User-friendly error messages
            if (error.code === 4001) {
                addAssistantMessage('❌ Transaction rejected by user in MetaMask.');
            } else if (error.code === 'INSUFFICIENT_FUNDS') {
                addAssistantMessage('❌ Insufficient funds for gas fee.');
            } else if (error.message.includes('user rejected')) {
                addAssistantMessage('❌ You rejected the transaction in MetaMask.');
            } else if (error.message.includes('execution reverted')) {
                addAssistantMessage('❌ Contract execution reverted. This could be due to invalid parameters or contract requirements.');
            } else if (error.message.includes('network')) {
                addAssistantMessage('❌ Network error. Please check your connection.');
            } else if (error.message.includes('chain')) {
                addAssistantMessage('❌ Wrong network. Please switch to BSC Testnet (Chain ID: 97).');
            } else {
                addAssistantMessage(`❌ Transaction failed: ${error.message}`);
            }
            
            return null;
        }
    }

    async function testContractCall() {
        if (!window.ethereum || !walletConnected) {
            addAssistantMessage('❌ Please connect your wallet first.');
            return;
        }
        
        try {
            const provider = new BrowserProvider(window.ethereum);
            const contractInterface = new ethers.Interface([
                'function getProjectStatistics() external view returns (uint256, uint256, uint256, uint256, uint256)'
            ]);
            
            const contractAddress = "0xE7392b8ee167980602d38225674bB377De5Fe287";
            const callData = contractInterface.encodeFunctionData('getProjectStatistics', []);
            
            const result = await provider.call({
                to: contractAddress,
                data: callData
            });
            
            addSystemMessage(`✅ Contract test call successful!`);
        } catch (error) {
            console.error('Contract test failed:', error);
            addAssistantMessage(`❌ Contract test failed: ${error.message}`);
        }
    }

    // ============ MAIN HANDLERS ============
    async function handleSubmit() {
        if (!input.trim() || isProcessing) return;
        
        const userInput = input;
        addUserMessage(userInput);
        input = '';
        isProcessing = true;
        
        const thinkingId = addThinkingIndicator();
        
        if (!backendConnected) {
            replaceThinkingWithResponse(thinkingId, '❌ Backend not connected.');
            isProcessing = false;
            return;
        }
        
        if (!agentInitialized) {
            replaceThinkingWithResponse(thinkingId, '🤖 AI Agent not initialized.');
            isProcessing = false;
            return;
        }
        
        if (!walletConnected) {
            replaceThinkingWithResponse(thinkingId, '🔐 Please connect your wallet first!');
            isProcessing = false;
            return;
        }
        
        try {
            const response = await fetch('http://localhost:3001/api/intents/signed', {
                method: 'POST',
                headers: { 
                    'Content-Type': 'application/json',
                    'Accept': 'application/json'
                },
                body: JSON.stringify({ 
                    user_input: userInput,
                    address: walletAddress,
                    chain_id: chainId,
                    signature: 'demo-signature-placeholder'
                })
            });
            
            if (response.ok) {
                const data = await response.json();
                
                // Replace thinking with AI response
                replaceThinkingWithResponse(thinkingId, data.ai_message);
                
                if (data.data && data.data.transaction_data) {
                    pendingTransaction = data.data.transaction_data;
                    window.lastBackendData = data.data;
                    
                    if (!pendingTransaction.chain_id) {
                        pendingTransaction.chain_id = chainId;
                    }
                    
                    // Wait for typing to complete before auto-signing
                    const checkTypingInterval = setInterval(() => {
                        if (isTypingComplete) {
                            clearInterval(checkTypingInterval);
                            // Add a small delay after typing finishes
                            setTimeout(() => {
                                addSystemMessage('⏳ Ready to sign transaction...');
                                // Wait another moment before triggering
                                setTimeout(() => {
                                    signTransaction(pendingTransaction, data.data)
                                        .then(txHash => {
                                            if (txHash) {
                                                handleTransactionSuccess(txHash, data.data);
                                            }
                                        })
                                        .catch(error => {
                                            addAssistantMessage(`❌ Auto-sign failed: ${error.message}`);
                                        });
                                }, 1500);
                            }, 500);
                        }
                    }, 100);
                }
            } else {
                const errorText = await response.text();
                replaceThinkingWithResponse(thinkingId, `❌ Request Failed\n\n${errorText}`);
            }
        } catch (error: any) {
            replaceThinkingWithResponse(thinkingId, `❌ Network Error\n\n${error.message}`);
        } finally {
            isProcessing = false;
        }
    }
    
    function handleTransactionSuccess(txHash: string, originalData: any) {
        addAssistantMessage(`🎉 Transaction Successful!\n\nTransaction Hash: \`${txHash}\``);
        addSystemMessage(`🔗 View on explorer: ${getExplorerUrl(txHash, chainId)}`);
        clearPendingTransaction();
    }
    
    // ============ HELPER FUNCTIONS ============
    function getExplorerUrl(txHash: string, chainId: number): string {
        switch (chainId) {
            case 1: return `https://etherscan.io/tx/${txHash}`;
            case 5: return `https://goerli.etherscan.io/tx/${txHash}`;
            case 97: return `https://testnet.bscscan.com/tx/${txHash}`;
            case 56: return `https://bscscan.com/tx/${txHash}`;
            case 137: return `https://polygonscan.com/tx/${txHash}`;
            case 80001: return `https://mumbai.polygonscan.com/tx/${txHash}`;
            default: return `#`;
        }
    }
    
    function handleKeyDown(e: KeyboardEvent) {
        if (e.key === 'Enter' && !e.shiftKey) {
            e.preventDefault();
            handleSubmit();
        }
    }
    
    function shortenAddress(address: string): string {
        if (!address) return '';
        return `${address.substring(0, 6)}...${address.substring(address.length - 4)}`;
    }
    
    function formatBalance(balance: string): string {
        const num = parseFloat(balance);
        if (isNaN(num)) return '0.000000';
        return num.toFixed(6);
    }
    
    function getNetworkName(chainId: number): string {
        switch (chainId) {
            case 1: return 'Ethereum Mainnet';
            case 5: return 'Goerli Testnet';
            case 97: return 'BSC Testnet';
            case 56: return 'BSC Mainnet';
            case 137: return 'Polygon Mainnet';
            case 80001: return 'Polygon Mumbai Testnet';
            default: return `Chain ${chainId}`;
        }
    }
    
    async function testBackend() {
        isProcessing = true;
        addUserMessage('Test backend connection');
        
        try {
            const response = await fetch('http://localhost:3001/api/hello', {
                method: 'POST',
                headers: { 
                    'Content-Type': 'application/json',
                    'Accept': 'application/json'
                },
                body: JSON.stringify({ name: 'Teemah User' })
            });
            
            if (response.ok) {
                const data = await response.json();
                addAssistantMessage(`✅ Backend Response\n\n${data.message}`);
                backendConnected = true;
            } else {
                const errorText = await response.text();
                addAssistantMessage(`❌ Backend Error\n\nStatus: ${response.status}\nError: ${errorText}`);
            }
        } catch (error: any) {
            addAssistantMessage(`❌ Network Error\n\n${error.message}`);
            backendConnected = false;
        } finally {
            isProcessing = false;
        }
    }
    
    async function testSimpleTransaction() {
        if (!window.ethereum || !walletConnected) {
            addAssistantMessage('❌ Please connect your wallet first.');
            return;
        }
        
        try {
            const provider = new BrowserProvider(window.ethereum);
            const signer = await provider.getSigner();
            
            const tx = {
                to: walletAddress,
                value: ethers.parseEther("0"),
                chainId: chainId,
            };
            
            const txResponse = await signer.sendTransaction(tx);
            const receipt = await txResponse.wait();
            
            addSystemMessage(`✅ MetaMask test successful! Transaction hash: ${receipt.hash}`);
        } catch (error: any) {
            if (error.code === 4001) {
                addAssistantMessage('❌ Test transaction rejected by user in MetaMask.');
            } else if (error.code === 'INSUFFICIENT_FUNDS') {
                addAssistantMessage('❌ Insufficient funds for gas fee.');
            } else {
                addAssistantMessage(`❌ Test failed: ${error.message}`);
            }
        }
    }
    
    async function manualTriggerSign() {
        if (!pendingTransaction) {
            addAssistantMessage('❌ No pending transaction to sign.');
            return;
        }
        
        if (!window.lastBackendData) {
            addAssistantMessage('❌ No transaction data available. Please try again.');
            return;
        }
        
        try {
            const txHash = await signTransaction(pendingTransaction, window.lastBackendData);
            if (txHash) {
                handleTransactionSuccess(txHash, window.lastBackendData);
            }
        } catch (error) {
            addAssistantMessage(`❌ Manual trigger failed: ${error.message}`);
        }
    }
    
    function testIntent(type: string) {
        let testMessage = '';
        
        switch(type) {
            case 'create': testMessage = 'Create a new project called CryptoToken with symbol CTK'; break;
            case 'invest': testMessage = 'Invest 0.5 ETH in project 0x1234567890abcdef'; break;
            case 'info': testMessage = 'Show me project details'; break;
            case 'list': testMessage = 'List all available projects'; break;
            case 'balance': testMessage = 'What is my current balance?'; break;
        }
        
        if (testMessage) {
            input = testMessage;
            setTimeout(() => handleSubmit(), 100);
        }
    }
</script>

<div class="flex h-screen bg-gray-900 text-white">
    <!-- Sidebar -->
    <div class="w-64 bg-gray-800 border-r border-gray-700 p-4 flex flex-col">
        <!-- Fixed header -->
        <div class="mb-8">
            <div class="flex items-center gap-3">
                <div class="w-10 h-10 bg-gradient-to-br from-blue-500 to-purple-600 rounded-lg flex items-center justify-center">
                    <Bot size={20} />
                </div>
                <div>
                    <h1 class="text-xl font-bold">Teemah AI</h1>
                    <p class="text-sm text-gray-400">v0.1 Beta</p>
                </div>
            </div>
        </div>
        
        <!-- Scrollable content -->
        <div class="flex-1 overflow-y-auto space-y-6">
            <!-- Agent Status -->
            <div class="p-3 rounded-lg {agentInitialized ? 'bg-purple-900/30 border border-purple-700' : 'bg-gray-700 border border-gray-600'}">
                <div class="flex items-center gap-2 mb-2">
                    <div class="w-2 h-2 rounded-full {agentInitialized ? 'bg-purple-500' : 'bg-gray-500'}"></div>
                    <span class="text-sm font-medium">{agentInitialized ? '🤖 Agent Ready' : '⚙️ AI Agent'}</span>
                </div>
                
                <button 
                    on:click={initializeAgent}
                    disabled={isInitializingAgent || !backendConnected}
                    class="w-full px-3 py-2 text-sm {backendConnected ? 'bg-purple-700 hover:bg-purple-800' : 'bg-gray-600'} rounded disabled:opacity-50 transition-colors flex items-center justify-center gap-2 mb-2"
                >
                    {#if isInitializingAgent}
                        <div class="w-4 h-4 border-2 border-white border-t-transparent rounded-full animate-spin"></div>
                        <span>Initializing...</span>
                    {:else}
                        <Bot size={14} />
                        <span>Initialize Agent</span>
                    {/if}
                </button>
                
                <button 
                    on:click={checkAgentStatus}
                    disabled={!backendConnected}
                    class="w-full px-3 py-2 text-sm bg-gray-700 hover:bg-gray-600 rounded disabled:opacity-50 transition-colors flex items-center justify-center gap-2"
                >
                    <Server size={14} />
                    <span>Check Status</span>
                </button>
            </div>
            
            <!-- Wallet Connection -->
            <div class="p-3 rounded-lg {walletConnected ? 'bg-green-900/30 border border-green-700' : 'bg-red-900/30 border border-red-700'}">
                <div class="flex items-center gap-2 mb-2">
                    <div class="w-2 h-2 rounded-full {walletConnected ? 'bg-green-500 animate-pulse' : 'bg-red-500'}"></div>
                    <span class="text-sm font-medium">{walletConnected ? '✅ Wallet Connected' : '❌ No Wallet'}</span>
                </div>
                
                {#if walletConnected}
                    <div class="mb-3">
                        <div class="text-xs text-gray-400 mb-1">Address</div>
                        <div class="text-sm font-mono bg-gray-900 p-2 rounded truncate" title={walletAddress}>
                            {shortenAddress(walletAddress)}
                        </div>
                        <div class="flex items-center gap-2 mt-2">
                            <Coins size={12} />
                            <span class="text-sm">{formatBalance(walletBalance)} ETH</span>
                        </div>
                        <div class="text-xs text-gray-400 mt-1">Chain ID: {chainId}</div>
                    </div>
                    
                    <button
                        on:click={disconnectWallet}
                        class="w-full px-3 py-2 text-sm bg-red-600 hover:bg-red-700 rounded disabled:opacity-50 transition-colors flex items-center justify-center gap-2"
                    >
                        <LogOut size={14} />
                        <span>Disconnect</span>
                    </button>
                {:else}
                    <button
                        on:click={connectMetaMask}
                        disabled={isConnectingWallet}
                        class="w-full px-3 py-2 text-sm bg-orange-600 hover:bg-orange-700 rounded disabled:opacity-50 transition-colors flex items-center justify-center gap-2"
                    >
                        {#if isConnectingWallet}
                            <div class="w-4 h-4 border-2 border-white border-t-transparent rounded-full animate-spin"></div>
                            <span>Connecting...</span>
                        {:else}
                            <Wallet size={14} />
                            <span>Connect MetaMask</span>
                        {/if}
                    </button>
                {/if}
            </div>
            
            <!-- Backend Status -->
            <div class="p-3 rounded-lg {backendConnected ? 'bg-blue-900/30 border border-blue-700' : 'bg-gray-700 border border-gray-600'}">
                <div class="flex items-center gap-2 mb-2">
                    <div class="w-2 h-2 rounded-full {backendConnected ? 'bg-blue-500' : 'bg-gray-500'}"></div>
                    <span class="text-sm font-medium">
                        {backendConnected ? '✅ Backend Ready' : '⚙️ Backend'}
                    </span>
                </div>
                
                <button 
                    on:click={checkBackendConnection}
                    disabled={isTestingConnection}
                    class="w-full px-3 py-2 text-sm bg-gray-700 hover:bg-gray-600 rounded disabled:opacity-50 transition-colors flex items-center justify-center gap-2 mb-2"
                >
                    <Server size={14} />
                    <span>Check Backend</span>
                </button>
                
                <button 
                    on:click={testBackend}
                    disabled={isProcessing}
                    class="w-full px-3 py-2 text-sm bg-green-700 hover:bg-green-800 rounded disabled:opacity-50 transition-colors flex items-center justify-center gap-2"
                >
                    <Server size={14} />
                    <span>Test Connection</span>
                </button>
            </div>
            
            <!-- Transaction Actions -->
            <div>
                <h3 class="text-sm font-medium text-gray-400 mb-2">Quick Actions</h3>
                <div class="space-y-2">
                    <button on:click={() => testIntent('create')} disabled={isProcessing || !walletConnected || !agentInitialized}
                        class="w-full text-left px-3 py-2 text-sm bg-blue-900/30 hover:bg-blue-900/50 rounded disabled:opacity-50 transition-colors flex items-center gap-2">
                        <div class="w-4 h-4">🏗️</div>
                        <span>Create Project</span>
                    </button>
                    <button on:click={() => testIntent('invest')} disabled={isProcessing || !walletConnected || !agentInitialized}
                        class="w-full text-left px-3 py-2 text-sm bg-green-900/30 hover:bg-green-900/50 rounded disabled:opacity-50 transition-colors flex items-center gap-2">
                        <div class="w-4 h-4">💰</div>
                        <span>Invest</span>
                    </button>
                    <button on:click={() => testIntent('info')} disabled={isProcessing || !agentInitialized}
                        class="w-full text-left px-3 py-2 text-sm bg-purple-900/30 hover:bg-purple-900/50 rounded disabled:opacity-50 transition-colors flex items-center gap-2">
                        <div class="w-4 h-4">📊</div>
                        <span>Project Info</span>
                    </button>
                    <button on:click={testSimpleTransaction} disabled={!walletConnected}
                        class="w-full text-left px-3 py-2 text-sm bg-red-900/30 hover:bg-red-900/50 rounded disabled:opacity-50 transition-colors flex items-center gap-2">
                        <div class="w-4 h-4">🐛</div>
                        <span>Debug: Test Simple TX</span>
                    </button>
                    <button on:click={manualTriggerSign} disabled={!pendingTransaction}
                        class="w-full text-left px-3 py-2 text-sm bg-orange-900/30 hover:bg-orange-900/50 rounded disabled:opacity-50 transition-colors flex items-center gap-2">
                        <div class="w-4 h-4">🔄</div>
                        <span>Manual Trigger Sign</span>
                    </button>
                    <button on:click={testContractCall} disabled={!walletConnected}
                        class="w-full text-left px-3 py-2 text-sm bg-blue-900/30 hover:bg-blue-900/50 rounded disabled:opacity-50 transition-colors flex items-center gap-2">
                        <div class="w-4 h-4">🧪</div>
                        <span>Test Contract Call</span>
                    </button>
                </div>
            </div>
            
            <!-- Bottom buttons -->
            <div class="pt-4 border-t border-gray-700">
                <button 
                    on:click={() => {
                        messages = [messages[0]];
                        pendingTransaction = null;
                        addSystemMessage('🆕 New chat session started.');
                    }}
                    class="w-full text-left px-3 py-2 rounded-lg bg-blue-600 hover:bg-blue-700 transition-colors flex items-center gap-2 mb-2"
                >
                    <Bot size={16} />
                    <span>New Chat</span>
                </button>
            </div>
        </div>
    </div>
    
    <!-- Main Content -->
    <div class="flex-1 flex flex-col">
        <!-- Status Bar -->
        <div class="border-b border-gray-700 px-6 py-3 text-sm">
            <div class="flex items-center justify-between">
                <div class="flex items-center gap-4">
                    <!-- Agent Status -->
                    <div class="flex items-center gap-2">
                        <div class="w-2 h-2 {agentInitialized ? 'bg-purple-500' : 'bg-yellow-500'} rounded-full"></div>
                        <span>{agentInitialized ? '🤖 Agent Ready' : '🤖 Initialize Agent'}</span>
                    </div>
                    
                    <!-- Wallet Status Indicator -->
                    <div class="flex items-center gap-2">
                        <div class="w-2 h-2 {walletConnected ? 'bg-green-500 animate-pulse' : 'bg-yellow-500'} rounded-full"></div>
                        <span>{walletConnected ? `Wallet: ${shortenAddress(walletAddress)}` : 'Connect Wallet'}</span>
                    </div>
                    
                    <!-- Balance Display -->
                    {#if walletConnected}
                        <div class="flex items-center gap-2 text-green-400">
                            <Coins size={12} />
                            <span>{formatBalance(walletBalance)} ETH</span>
                        </div>
                    {/if}
                    
                    <!-- Pending Transaction Indicator -->
                    {#if pendingTransaction}
                        <div class="flex items-center gap-2 text-yellow-400">
                            <div class="w-2 h-2 bg-yellow-500 rounded-full animate-pulse"></div>
                            <span>Transaction Ready</span>
                        </div>
                    {/if}
                </div>
                <div class="flex items-center gap-4">
                    <!-- Backend Status -->
                    <div class="flex items-center gap-2">
                        <div class="w-2 h-2 {backendConnected ? 'bg-blue-500' : 'bg-gray-500'} rounded-full"></div>
                        <span class="text-gray-400">{backendConnected ? 'Backend Ready' : 'Backend Offline'}</span>
                    </div>
                    
                    <!-- Network Info -->
                    {#if walletConnected}
                        <div class="text-gray-400">Chain: {getNetworkName(chainId)}</div>
                    {/if}
                </div>
            </div>
        </div>
        
        <!-- Chat Messages -->
        <div
            bind:this={chatContainer}
            class="flex-1 overflow-y-auto p-6 space-y-4"
        >
            {#each messages as message}
                <div class="flex {message.role === 'user' ? 'justify-end' : ''}">
                    <div class="max-w-2xl rounded-xl p-4 {message.role === 'user' ? 'bg-blue-600/20 border border-blue-500/30' : 'bg-gray-800/50 border border-gray-700'} {message.isTyping ? 'border-purple-500/50' : ''}">
                        <div class="flex items-center gap-2 mb-2">
                            {#if message.role === 'assistant'}
                                <Bot size={16} class="text-blue-400" />
                                <span class="font-medium">Teemah AI</span>
                                {#if message.isTyping}
                                    <div class="flex items-center gap-1 ml-2">
                                        <div class="w-1 h-1 bg-purple-500 rounded-full animate-pulse"></div>
                                        <div class="w-1 h-1 bg-purple-500 rounded-full animate-pulse" style="animation-delay: 0.2s"></div>
                                        <div class="w-1 h-1 bg-purple-500 rounded-full animate-pulse" style="animation-delay: 0.4s"></div>
                                    </div>
                                {/if}
                            {:else}
                                <span class="font-medium text-blue-300">You</span>
                            {/if}
                        </div>
                        
                        <!-- Show typing cursor for incomplete messages -->
                        <div class="whitespace-pre-wrap min-h-[1em] text-base">
                            {@html formatMessageContent(message.content)}
                            {#if message.isTyping && !message.isComplete}
                                <span class="inline-block w-2 h-5 ml-1 bg-purple-500 animate-pulse align-middle"></span>
                            {/if}
                        </div>
                    </div>
                </div>
            {/each}
            
            {#if isProcessing}
                <div class="flex">
                    <div class="max-w-2xl rounded-xl p-4 bg-gray-800/50 border border-gray-700">
                        <div class="flex items-center gap-2">
                            <div class="w-2 h-2 bg-blue-500 rounded-full animate-pulse"></div>
                            <div class="w-2 h-2 bg-purple-500 rounded-full animate-pulse" style="animation-delay: 0.1s"></div>
                            <div class="w-2 h-2 bg-pink-500 rounded-full animate-pulse" style="animation-delay: 0.2s"></div>
                            <span class="text-gray-400">Processing...</span>
                        </div>
                    </div>
                </div>
            {/if}
            
            {#if isSigningTransaction}
                <div class="flex">
                    <div class="max-w-2xl rounded-xl p-4 bg-yellow-900/20 border border-yellow-700/30">
                        <div class="flex items-center gap-2">
                            <div class="w-4 h-4 border-2 border-yellow-500 border-t-transparent rounded-full animate-spin"></div>
                            <span class="text-yellow-400">Waiting for MetaMask signature...</span>
                        </div>
                        <p class="text-sm text-yellow-300 mt-2">Please check your MetaMask wallet to sign the transaction.</p>
                    </div>
                </div>
            {/if}
        </div>
        
        <!-- Input -->
        <div class="border-t border-gray-700 p-6">
            <form on:submit|preventDefault={handleSubmit} class="max-w-3xl mx-auto">
                <div class="flex gap-3">
                    <textarea
                        bind:value={input}
                        rows="2"
                        placeholder="{!backendConnected ? 'Backend not connected' : !agentInitialized ? 'Initialize AI Agent first' : !walletConnected ? 'Connect your wallet to start transacting' : pendingTransaction ? 'Transaction ready to sign! Click "Sign Transaction" button' : 'What transaction would you like to make? (e.g., "Create a new project" or "Invest 0.1 ETH")'}"
                        class="flex-1 bg-gray-800 border border-gray-700 rounded-xl px-4 py-3 focus:outline-none focus:ring-2 focus:ring-blue-500 resize-none disabled:opacity-50 text-base"
                        on:keydown={handleKeyDown}
                        disabled={!backendConnected || !agentInitialized || !walletConnected || pendingTransaction}
                    />
                    <button
                        type="submit"
                        disabled={!input.trim() || isProcessing || !backendConnected || !agentInitialized || !walletConnected || pendingTransaction}
                        class="self-end px-6 py-3 {backendConnected && agentInitialized && walletConnected ? 'bg-green-600 hover:bg-green-700' : 'bg-gray-600'} rounded-xl font-medium disabled:opacity-50 transition-colors flex items-center gap-2"
                    >
                        <Send size={18} />
                        <span>{!backendConnected ? 'Connect Backend' : !agentInitialized ? 'Initialize Agent' : !walletConnected ? 'Connect Wallet' : pendingTransaction ? 'Transaction Pending' : 'Execute'}</span>
                    </button>
                </div>
                
                <div class="mt-3 text-sm text-gray-400 flex gap-6 justify-center">
                    <div class="flex items-center gap-2">
                        <Zap size={14} />
                        <span>On-Chain Execution</span>
                    </div>
                    <div class="flex items-center gap-2">
                        <Shield size={14} />
                        <span>Wallet Signed</span>
                    </div>
                    <div class="flex items-center gap-2">
                        <Clock size={14} />
                        <span>~2-15s</span>
                    </div>
                </div>
                
                {#if !backendConnected}
                    <div class="mt-3 text-center">
                        <div class="text-red-500 text-sm flex items-center justify-center gap-2">
                            <AlertCircle size={14} />
                            <span>Backend connection required</span>
                        </div>
                    </div>
                {:else if !agentInitialized}
                    <div class="mt-3 text-center">
                        <div class="text-yellow-500 text-sm flex items-center justify-center gap-2">
                            <AlertCircle size={14} />
                            <span>AI Agent not initialized</span>
                        </div>
                    </div>
                {:else if !walletConnected}
                    <div class="mt-3 text-center">
                        <div class="text-yellow-500 text-sm flex items-center justify-center gap-2">
                            <AlertCircle size={14} />
                            <span>Wallet required for transactions</span>
                        </div>
                    </div>
                {:else if pendingTransaction}
                    <div class="mt-3 text-center">
                        <div class="text-green-500 text-sm flex items-center justify-center gap-2">
                            <AlertCircle size={14} />
                            <span>Transaction ready! Click "Sign Transaction" in sidebar</span>
                        </div>
                    </div>
                {/if}
            </form>
        </div>
    </div>
</div>

<style>
    :global(body) {
        margin: 0;
        overflow: hidden;
        font-family: 'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, sans-serif;
    }
    
    .animate-pulse {
        animation: pulse 1s ease-in-out infinite;
    }
    
    @keyframes pulse {
        0%, 100% { opacity: 1; }
        50% { opacity: 0.5; }
    }
    
    .animate-spin {
        animation: spin 1s linear infinite;
    }
    
    @keyframes spin {
        to { transform: rotate(360deg); }
    }
    
    .truncate {
        overflow: hidden;
        text-overflow: ellipsis;
        white-space: nowrap;
    }
    
    /* Custom scrollbar */
    :global(::-webkit-scrollbar) {
        width: 8px;
    }
    
    :global(::-webkit-scrollbar-track) {
        background: rgba(255, 255, 255, 0.05);
    }
    
    :global(::-webkit-scrollbar-thumb) {
        background: rgba(255, 255, 255, 0.1);
        border-radius: 4px;
    }
    
    :global(::-webkit-scrollbar-thumb:hover) {
        background: rgba(255, 255, 255, 0.2);
    }
    
    /* Improve textarea appearance */
    textarea {
        font-family: 'Inter', sans-serif;
    }
    
    /* Smooth transitions */
    * {
        transition: background-color 0.2s ease, border-color 0.2s ease;
    }
    
    /* Code styling */
    code {
        font-family: 'Monaco', 'Menlo', 'Ubuntu Mono', monospace;
        font-size: 0.9em;
    }
    
    /* Custom typing animation */
    @keyframes typing-cursor {
        0%, 100% { opacity: 1; }
        50% { opacity: 0; }
    }

    .typing-cursor {
        animation: typing-cursor 1s infinite;
    }

    /* Smooth scroll behavior */
    .chat-container {
        scroll-behavior: smooth;
    }

    /* Message fade-in */
    .message-enter {
        animation: messageEnter 0.3s ease-out;
    }

    @keyframes messageEnter {
        from {
            opacity: 0;
            transform: translateY(10px);
        }
        to {
            opacity: 1;
            transform: translateY(0);
        }
    }
</style>
