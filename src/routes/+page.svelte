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
        }
    }
    
    // State
    let messages = [
        {
            id: 1,
            role: 'assistant' as const,
            content: '🚀 **Welcome to Teemah AI**\n\nConnect your wallet to start making on-chain transactions!',
            timestamp: new Date()
        }
    ];
    let input = '';
    let isProcessing = false;
    let backendConnected = false;
    let isTestingConnection = false;
    
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
    
    // Setup wallet event listeners
    function setupWalletListeners() {
        if (window.ethereum) {
            // Listen for account changes
            window.ethereum.on('accountsChanged', (accounts: string[]) => {
                console.log('Accounts changed:', accounts);
                if (accounts.length === 0) {
                    // User disconnected wallet
                    handleWalletDisconnect();
                } else {
                    // Account changed
                    walletAddress = accounts[0];
                    addSystemMessage(`🔄 Account changed: ${shortenAddress(walletAddress)}`);
                    updateWalletInfo();
                }
            });
            
            // Listen for chain changes
            window.ethereum.on('chainChanged', (chainIdHex: string) => {
                console.log('Chain changed:', chainIdHex);
                const newChainId = parseInt(chainIdHex, 16);
                chainId = newChainId;
                addSystemMessage(`🔗 Network changed: Chain ${newChainId}`);
                updateWalletInfo();
            });
            
            // Listen for wallet disconnect
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
    
    // Check if backend is running
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
    
    // Initialize AI Agent
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
                    contract_address: "0xAcC8850B4664f0620fa013Ef4de5b52C57cef32C"
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
    
    // Check agent status
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
    
    // Check wallet connection status
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
        
        console.log('🔄 Connecting to MetaMask...');
        
        // Request account access
        const accounts = await window.ethereum.request({ 
            method: 'eth_requestAccounts' 
        });
        
        if (accounts.length === 0) {
            throw new Error('No accounts found');
        }
        
        const address = accounts[0];
        console.log('✅ Got address:', address);
        
        // Get chain ID
        const chainIdHex = await window.ethereum.request({ 
            method: 'eth_chainId' 
        });
        const chainId = parseInt(chainIdHex, 16);
        console.log('✅ Got chain ID:', chainId);
        
        // Connect to backend
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
            
            // Update balance
            const balance = await provider.getBalance(address);
            walletBalance = (Number(balance) / 1e18).toFixed(6);
            
            console.log('✅ Wallet fully connected:', {
                address: walletAddress,
                connected: walletConnected,
                balance: walletBalance,
                chainId: chainId
            });
        }
     } catch (error: any) {
        console.error('Failed to connect wallet:', error);
        addAssistantMessage(`❌ Failed to connect wallet: ${error.message}`);
     } finally {
        isConnectingWallet = false;
     }
    }
    
    // Update balance using ethers.js
    async function updateBalanceWithEthers(provider: BrowserProvider, address: string) {
        try {
            const balance = await provider.getBalance(address);
            // Convert from wei to ETH
            walletBalance = (Number(balance) / 1e18).toFixed(6);
        } catch (error) {
            console.error('Failed to get balance:', error);
        }
    }
    
    // Connect WalletConnect
    async function connectWalletConnect() {
        addAssistantMessage('🔗 WalletConnect integration coming soon!');
        // Implement WalletConnect connection here
    }
    
    // Disconnect wallet
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
    
    // Update wallet balance from backend
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
    async function signTransaction(transactionData: any, backendData: any = null): Promise<string | null> {
     console.log('🚨🚨🚨 SIGNTRANSACTION FUNCTION CALLED! 🚨🚨🚨');
     console.log('1. Transaction data:', transactionData);
     console.log('2. Backend data action:', backendData?.action);
     console.log('3. Wallet connected:', walletConnected);
     console.log('4. window.ethereum exists:', !!window.ethereum);
    
     // Add a simple alert to confirm it's being called
     alert('signTransaction function was called! Check console for details.');
    
     if (!window.ethereum || !walletConnected) {
        console.log('❌ No ethereum provider or wallet not connected');
        addAssistantMessage('❌ Please connect your wallet first.');
        return null;
     }
    
     // For now, let's just test with a simple transaction
     console.log('🔧 Testing with simple transaction first...');
    
     try {
        const provider = new BrowserProvider(window.ethereum);
        const signer = await provider.getSigner();
        
        // SIMPLE TEST TRANSACTION
        const tx = {
            to: walletAddress, // Send to self
            value: ethers.parseEther("0"),
            chainId: chainId,
        };
        
        console.log('📤 Sending simple test transaction...');
        
        const txResponse = await signer.sendTransaction(tx);
        console.log('✅ Simple test transaction sent! Hash:', txResponse.hash);
        
        const receipt = await txResponse.wait();
        console.log('✅ Simple test transaction confirmed! Hash:', receipt.hash);
        
        addSystemMessage(`✅ Simple test successful! Transaction hash: ${receipt.hash}`);
        
        // Now try the actual contract call
        console.log('🔄 Now trying actual contract call...');
        
        if (backendData?.action === 'create_project' && backendData?.parameters) {
            console.log('🔨 Creating actual contract call...');
            
            const params = backendData.parameters;
            const contractInterface = new ethers.Interface([
                'function createProjectWithTokenViaTelegram(address creator, string project_name, string token_name, string token_symbol, uint8 token_decimals, uint256 initial_supply, uint256 soft_cap, uint256 hard_cap, uint256 start_time, uint256 end_time, uint256 token_price, uint256 tokens_for_sale, uint16 liquidity_percent, uint16 marketing_percent, uint256 marketing_telegram_id) external returns (address)'
            ]);
            
            const encodedData = contractInterface.encodeFunctionData('createProjectWithTokenViaTelegram', [
                walletAddress,
                params.project_name,
                params.token_name,
                params.token_symbol,
                params.token_decimals,
                ethers.toBigInt(params.initial_supply),
                ethers.toBigInt(params.soft_cap),
                ethers.toBigInt(params.hard_cap),
                ethers.toBigInt(params.start_time),
                ethers.toBigInt(params.end_time),
                ethers.toBigInt(params.token_price),
                ethers.toBigInt(params.tokens_for_sale),
                params.liquidity_percent,
                params.marketing_percent,
                ethers.toBigInt(params.marketing_telegram_id)
            ]);
            
            console.log('📦 Encoded contract data length:', encodedData.length);
            
            const contractTx = {
                to: transactionData.to,
                data: encodedData,
                value: ethers.parseEther(transactionData.value || "0"),
                chainId: transactionData.chain_id || chainId,
            };
            
            console.log('📤 Sending contract transaction...');
            
            const contractTxResponse = await signer.sendTransaction(contractTx);
            console.log('✅ Contract transaction sent! Hash:', contractTxResponse.hash);
            
            const contractReceipt = await contractTxResponse.wait();
            console.log('✅ Contract transaction confirmed! Hash:', contractReceipt.hash);
            
            return contractReceipt.hash;
        }
        
        return receipt.hash;
        
     } catch (error: any) {
        console.error('❌ Transaction error:', error);
        console.error('Error details:', {
            code: error.code,
            message: error.message,
            stack: error.stack
        });
        
        if (error.code === 4001) {
            addAssistantMessage('❌ Transaction rejected by user in MetaMask.');
        } else if (error.code === 'INSUFFICIENT_FUNDS') {
            addAssistantMessage('❌ Insufficient funds for gas fee.');
        } else {
            addAssistantMessage(`❌ Transaction failed: ${error.message}`);
        }
        
        return null;
     }
    }
    
    async function switchNetwork(chainId: number) {
        if (!window.ethereum) return false;
        
        const chainIdHex = `0x${chainId.toString(16)}`;
        
        try {
            await window.ethereum.request({
                method: 'wallet_switchEthereumChain',
                params: [{ chainId: chainIdHex }],
            });
            return true;
        } catch (switchError: any) {
            // If network is not added, try to add it
            if (switchError.code === 4902) {
                try {
                    const networkParams = getNetworkParams(chainId);
                    await window.ethereum.request({
                        method: 'wallet_addEthereumChain',
                        params: [networkParams],
                    });
                    return true;
                } catch (addError) {
                    console.error('Failed to add network:', addError);
                    return false;
                }
            }
            return false;
        }
    }
    
    // Helper to get network name
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
    
    // Helper to get network params for adding to MetaMask
    function getNetworkParams(chainId: number) {
        switch (chainId) {
            case 97: // BSC Testnet
                return {
                    chainId: '0x61',
                    chainName: 'BNB Smart Chain Testnet',
                    nativeCurrency: {
                        name: 'BNB',
                        symbol: 'BNB',
                        decimals: 18
                    },
                    rpcUrls: ['https://data-seed-prebsc-1-s1.binance.org:8545/'],
                    blockExplorerUrls: ['https://testnet.bscscan.com/']
                };
            case 56: // BSC Mainnet
                return {
                    chainId: '0x38',
                    chainName: 'BNB Smart Chain Mainnet',
                    nativeCurrency: {
                        name: 'BNB',
                        symbol: 'BNB',
                        decimals: 18
                    },
                    rpcUrls: ['https://bsc-dataseed.binance.org/'],
                    blockExplorerUrls: ['https://bscscan.com/']
                };
            default:
                return {
                    chainId: `0x${chainId.toString(16)}`,
                    chainName: `Chain ${chainId}`,
                    nativeCurrency: {
                        name: 'ETH',
                        symbol: 'ETH',
                        decimals: 18
                    },
                    rpcUrls: ['https://rpc.example.com'],
                    blockExplorerUrls: ['https://explorer.example.com']
                };
        }
    }
    
    // Shorten address for display
    function shortenAddress(address: string): string {
        if (!address) return '';
        return `${address.substring(0, 6)}...${address.substring(address.length - 4)}`;
    }
    
    // Format balance for display
    function formatBalance(balance: string): string {
        const num = parseFloat(balance);
        if (isNaN(num)) return '0.000000';
        return num.toFixed(6);
    }

    // async function handleSubmit() {
    //  if (!input.trim() || isProcessing) return;
    
    //  const userInput = input;
    //  addUserMessage(userInput);
    //  input = '';
    //  isProcessing = true;
    
    //  if (!backendConnected) {
    //     addAssistantMessage('❌ Backend not connected.');
    //     isProcessing = false;
    //     return;
    //  }
    
    //  if (!agentInitialized) {
    //     addAssistantMessage('🤖 AI Agent not initialized.');
    //     isProcessing = false;
    //     return;
    //  }
    
    //  if (!walletConnected) {
    //     addAssistantMessage('🔐 Please connect your wallet first!');
    //     isProcessing = false;
    //     return;
    //  }
    
    //  try {
    //     const response = await fetch('http://localhost:3001/api/intents/signed', {
    //         method: 'POST',
    //         headers: { 
    //             'Content-Type': 'application/json',
    //             'Accept': 'application/json'
    //         },
    //         body: JSON.stringify({ 
    //             user_input: userInput,
    //             address: walletAddress,
    //             chain_id: chainId,
    //             signature: 'demo-signature-placeholder'
    //         })
    //     });
        
    //     if (response.ok) {
    //         const data = await response.json();
    //         addAssistantMessage(data.ai_message);
            
    //         if (data.data && data.data.transaction_data) {
    //             pendingTransaction = data.data.transaction_data;
    //             if (!pendingTransaction.chain_id) {
    //                 console.log('⚠️ Chain ID missing from transaction data, using wallet chain ID:', chainId);
    //                 pendingTransaction.chain_id = chainId;
    //             }
                
    //             console.log('📝 Pending transaction ready:', pendingTransaction);
    //             console.log('📊 Full backend data:', data.data);
                
    //             // Store backend data globally
    //             window.lastBackendData = data.data;
                
    //             // AUTO-SIGN: Call signTransaction directly after a short delay
    //             setTimeout(async () => {
    //                 console.log('⏰ Auto-signing transaction...');
    //                 try {
    //                     const txHash = await signTransaction(pendingTransaction, data.data);
    //                     if (txHash) {
    //                         handleTransactionSuccess(txHash, data.data);
    //                     }
    //                 } catch (error) {
    //                     console.error('Auto-sign error:', error);
    //                     addAssistantMessage(`❌ Auto-sign failed: ${error.message}`);
    //                 }
    //             }, 1000); // 1 second delay to let user read the message
                
    //         } else if (data.data) {
    //             // Just show data if no transaction needed
    //             const cleanData = { ...data.data };
    //             delete cleanData.user_id;
    //             delete cleanData.internal_id;
                
    //             if (Object.keys(cleanData).length > 0) {
    //                 addAssistantMessage(`📊 **Details:**\n\n\`\`\`json\n${JSON.stringify(cleanData, null, 2)}\n\`\`\``);
    //             }
    //         }
    //     } else {
    //         const errorText = await response.text();
    //         addAssistantMessage(`❌ **Request Failed**\n\n${errorText}`);
    //     }
    //  } catch (error: any) {
    //     addAssistantMessage(`❌ **Network Error**\n\n${error.message}`);
    //  } finally {
    //     isProcessing = false;
    //  }
    // }
    async function handleSubmit() {
     if (!input.trim() || isProcessing) return;
    
     const userInput = input;
     addUserMessage(userInput);
     input = '';
     isProcessing = true;
    
     console.log('🔄 handleSubmit called with:', userInput);
    
     if (!backendConnected) {
        console.log('❌ Backend not connected');
        addAssistantMessage('❌ Backend not connected.');
        isProcessing = false;
        return;
     }
    
     if (!agentInitialized) {
        console.log('❌ Agent not initialized');
        addAssistantMessage('🤖 AI Agent not initialized.');
        isProcessing = false;
        return;
     }
    
     if (!walletConnected) {
        console.log('❌ Wallet not connected');
        addAssistantMessage('🔐 Please connect your wallet first!');
        isProcessing = false;
        return;
     }
     
     try {
        console.log('📤 Sending request to backend...');
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
        
        console.log('✅ Backend response status:', response.status);
        
        if (response.ok) {
            const data = await response.json();
            console.log('📦 Backend data received:', data);
            addAssistantMessage(data.ai_message);
            
            if (data.data && data.data.transaction_data) {
                pendingTransaction = data.data.transaction_data;
                console.log('📝 Pending transaction set:', pendingTransaction);
                
                if (!pendingTransaction.chain_id) {
                    console.log('⚠️ Chain ID missing, using wallet chain ID:', chainId);
                    pendingTransaction.chain_id = chainId;
                }
                
                // Store backend data
                window.lastBackendData = data.data;
                console.log('💾 Stored backend data in window.lastBackendData');
                
                // Auto-sign after delay
                console.log('⏰ Setting up auto-sign timeout (500ms)...');
                setTimeout(() => {
                    console.log('🏃‍♂️ Auto-sign timeout triggered!');
                    console.log('📊 Calling signTransaction with:', {
                        transactionData: pendingTransaction,
                        backendData: data.data
                    });
                    
                    signTransaction(pendingTransaction, data.data)
                        .then(txHash => {
                            console.log('✅ signTransaction completed, hash:', txHash);
                            if (txHash) {
                                handleTransactionSuccess(txHash, data.data);
                            }
                        })
                        .catch(error => {
                            console.error('❌ signTransaction failed:', error);
                            addAssistantMessage(`❌ Auto-sign failed: ${error.message}`);
                        });
                }, 500);
                
            } else {
                console.log('📊 No transaction data, just showing details');
                if (data.data) {
                    const cleanData = { ...data.data };
                    delete cleanData.user_id;
                    delete cleanData.internal_id;
                    
                    if (Object.keys(cleanData).length > 0) {
                        addAssistantMessage(`📊 **Details:**\n\n\`\`\`json\n${JSON.stringify(cleanData, null, 2)}\n\`\`\``);
                    }
                }
            }
        } else {
            const errorText = await response.text();
            console.error('❌ Backend error:', response.status, errorText);
            addAssistantMessage(`❌ **Request Failed**\n\n${errorText}`);
        }
     } catch (error: any) {
        console.error('❌ Network error:', error);
        addAssistantMessage(`❌ **Network Error**\n\n${error.message}`);
        } finally {
         console.log('🏁 handleSubmit completed');
         isProcessing = false;
        }
    }
    
    function handleTransactionSuccess(txHash: string, originalData: any) {
        addAssistantMessage(`🎉 **Transaction Successful!**\n\nTransaction Hash: \`${txHash}\``);
        
        // Update the response with transaction hash
        if (originalData) {
            const updatedData = { 
                ...originalData, 
                transaction_hash: txHash,
                status: 'completed',
                explorer_url: getExplorerUrl(txHash, chainId)
            };
            addAssistantMessage(`📊 **Transaction Details:**\n\n\`\`\`json\n${JSON.stringify(updatedData, null, 2)}\n\`\`\``);
            
            // Add a link to view the transaction
            addSystemMessage(`🔗 View on explorer: ${getExplorerUrl(txHash, chainId)}`);
        }
        
        // Clear pending transaction
        pendingTransaction = null;
    }
    
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
    
    // Test backend connection
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
                addAssistantMessage(`✅ **Backend Response**\n\n${data.message}`);
                backendConnected = true;
            } else {
                const errorText = await response.text();
                addAssistantMessage(`❌ **Backend Error**\n\nStatus: ${response.status}\nError: ${errorText}`);
            }
        } catch (error: any) {
            addAssistantMessage(`❌ **Network Error**\n\n${error.message}`);
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
    
     console.log('🔧 Testing simple transaction...');
     addSystemMessage('🔧 Testing simple transaction to MetaMask...');
    
     try {
        const provider = new BrowserProvider(window.ethereum);
        const signer = await provider.getSigner();
        
        // Very simple transaction - send 0 ETH to yourself
        const tx = {
            to: walletAddress,
            value: ethers.parseEther("0"),
            chainId: chainId,
        };
        
        console.log('Sending test transaction:', tx);
        
        const txResponse = await signer.sendTransaction(tx);
        console.log('Test transaction sent! Hash:', txResponse.hash);
        
        const receipt = await txResponse.wait();
        console.log('Test transaction confirmed! Hash:', receipt.hash);
        
        addSystemMessage(`✅ MetaMask test successful! Transaction hash: ${receipt.hash}`);
        
     } catch (error: any) {
        console.error('Test transaction error:', error);
        console.error('Error code:', error.code);
        console.error('Error message:', error.message);
        
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
     console.log('🔄 Manual trigger sign called');
    
     if (!pendingTransaction) {
        console.log('❌ No pending transaction');
        addAssistantMessage('❌ No pending transaction to sign.');
        return;
     }
    
        if (!window.lastBackendData) {
        console.log('❌ No backend data available');
        addAssistantMessage('❌ No transaction data available. Please try creating the project again.');
         return;
        }
    
     console.log('📊 Manual trigger - pendingTransaction:', pendingTransaction);
      console.log('📊 Manual trigger - lastBackendData:', window.lastBackendData);
    
        try {
        const txHash = await signTransaction(pendingTransaction, window.lastBackendData);
        if (txHash) {
            handleTransactionSuccess(txHash, window.lastBackendData);
        }
        } catch (error) {
        console.error('❌ Manual trigger failed:', error);
        addAssistantMessage(`❌ Manual trigger failed: ${error.message}`);
        }
    }
    
    // Test with pre-defined intents
    function testIntent(type: string) {
        let testMessage = '';
        
        switch(type) {
            case 'create':
                testMessage = 'Create a new project called CryptoToken with symbol CTK';
                break;
            case 'invest':
                testMessage = 'Invest 0.5 ETH in project 0x1234567890abcdef';
                break;
            case 'info':
                testMessage = 'Show me project details';
                break;
            case 'list':
                testMessage = 'List all available projects';
                break;
            case 'balance':
                testMessage = 'What is my current balance?';
                break;
        }
        
        if (testMessage) {
            input = testMessage;
            // Auto-submit after a brief delay
            setTimeout(() => handleSubmit(), 100);
        }
    }
    
    // Helper functions
    function addUserMessage(content: string) {
        messages = [...messages, {
            id: messages.length + 1,
            role: 'user' as const,
            content,
            timestamp: new Date()
        }];
    }
    
    function addAssistantMessage(content: string) {
        messages = [...messages, {
            id: messages.length + 1,
            role: 'assistant' as const,
            content,
            timestamp: new Date()
        }];
    }
    
    function addSystemMessage(content: string) {
        messages = [...messages, {
            id: messages.length + 1,
            role: 'assistant' as const,
            content: `⚙️ ${content}`,
            timestamp: new Date()
        }];
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
                    <span class="text-sm font-medium">
                        {agentInitialized ? '🤖 Agent Ready' : '⚙️ AI Agent'}
                    </span>
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
            
            <!-- Wallet Connection Status -->
            <div class="p-3 rounded-lg {walletConnected ? 'bg-green-900/30 border border-green-700' : 'bg-red-900/30 border border-red-700'}">
                <div class="flex items-center gap-2 mb-2">
                    <div class="w-2 h-2 rounded-full {walletConnected ? 'bg-green-500 animate-pulse' : 'bg-red-500'}"></div>
                    <span class="text-sm font-medium">
                        {walletConnected ? '✅ Wallet Connected' : '❌ No Wallet'}
                    </span>
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
                        <div class="text-xs text-gray-400">Type: {walletType}</div>
                    </div>
                    
                    <button
                        on:click={disconnectWallet}
                        class="w-full px-3 py-2 text-sm bg-red-600 hover:bg-red-700 rounded disabled:opacity-50 transition-colors flex items-center justify-center gap-2"
                    >
                        <LogOut size={14} />
                        <span>Disconnect</span>
                    </button>
                {:else}
                    <div class="space-y-2">
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
                        
                        <button
                            on:click={connectWalletConnect}
                            disabled={isConnectingWallet}
                            class="w-full px-3 py-2 text-sm bg-blue-600 hover:bg-blue-700 rounded disabled:opacity-50 transition-colors flex items-center justify-center gap-2"
                        >
                            <Wallet size={14} />
                            <span>WalletConnect</span>
                        </button>
                    </div>
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
                <h3 class="text-sm font-medium text-gray-400 mb-2">Transaction Actions</h3>
                <div class="space-y-2">
                    <button
                        on:click={() => testIntent('create')}
                        disabled={isProcessing || !walletConnected || !agentInitialized}
                        class="w-full text-left px-3 py-2 text-sm bg-blue-900/30 hover:bg-blue-900/50 rounded disabled:opacity-50 transition-colors flex items-center gap-2"
                    >
                        <div class="w-4 h-4">🏗️</div>
                        <span>Create Project</span>
                    </button>
                    <button
                        on:click={() => testIntent('invest')}
                        disabled={isProcessing || !walletConnected || !agentInitialized}
                        class="w-full text-left px-3 py-2 text-sm bg-green-900/30 hover:bg-green-900/50 rounded disabled:opacity-50 transition-colors flex items-center gap-2"
                    >
                        <div class="w-4 h-4">💰</div>
                        <span>Invest</span>
                    </button>
                    <button
                        on:click={() => testIntent('info')}
                        disabled={isProcessing || !agentInitialized}
                        class="w-full text-left px-3 py-2 text-sm bg-purple-900/30 hover:bg-purple-900/50 rounded disabled:opacity-50 transition-colors flex items-center gap-2"
                    >
                        <div class="w-4 h-4">📊</div>
                        <span>Project Info</span>
                    </button>
                    <button
                        on:click={() => testIntent('balance')}
                        disabled={!walletConnected}
                        class="w-full text-left px-3 py-2 text-sm bg-yellow-900/30 hover:bg-yellow-900/50 rounded disabled:opacity-50 transition-colors flex items-center gap-2"
                    >
                        <Coins size={14} />
                        <span>Check Balance</span>
                    </button>
                    <button
                      on:click={testSimpleTransaction}
                     disabled={!walletConnected}
                    class="w-full text-left px-3 py-2 text-sm bg-red-900/30 hover:bg-red-900/50 rounded disabled:opacity-50 transition-colors flex items-center gap-2"
                    >
                    <div class="w-4 h-4">🐛</div>
                     <span>Debug: Test Simple TX</span>
                    </button>
                    <button
                     on:click={manualTriggerSign}
                     disabled={!pendingTransaction}
                     class="w-full text-left px-3 py-2 text-sm bg-orange-900/30 hover:bg-orange-900/50 rounded disabled:opacity-50 transition-colors flex items-center gap-2"
                    >
                    <div class="w-4 h-4">🔄</div>
                    <span>Manual Trigger Sign</span>
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
                    <div class="max-w-2xl rounded-xl p-4 {message.role === 'user' ? 'bg-blue-600/20 border border-blue-500/30' : 'bg-gray-800/50 border border-gray-700'}">
                        <div class="flex items-center gap-2 mb-2">
                            {#if message.role === 'assistant'}
                                <Bot size={16} class="text-blue-400" />
                                <span class="font-medium">Teemah AI</span>
                            {:else}
                                <span class="font-medium text-blue-300">You</span>
                            {/if}
                        </div>
                        <div class="whitespace-pre-line">{@html message.content.replace(/\n/g, '<br>').replace(/`([^`]+)`/g, '<code class="bg-gray-700 px-1 py-0.5 rounded">$1</code>')}</div>
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
                        class="flex-1 bg-gray-800 border border-gray-700 rounded-xl px-4 py-3 focus:outline-none focus:ring-2 focus:ring-blue-500 resize-none disabled:opacity-50"
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
</style>