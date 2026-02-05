<script lang="ts">
    import { onMount, afterUpdate } from 'svelte';
    import { Bot, Send, Zap, Shield, Clock, Server, AlertCircle } from 'lucide-svelte';
    
    // State
    let messages = [
        {
            id: 1,
            role: 'assistant',
            content: '🚀 **Welcome to Teemah AI**\n\nI\'m your Onchain Ai Assistant. What can we Transact today!',
            timestamp: new Date()
        }
    ];
    let input = '';
    let isProcessing = false;
    let backendConnected = false;
    let isTestingConnection = false;

    let chatContainer: HTMLDivElement;

    afterUpdate(() => {
        if (chatContainer) {
            chatContainer.scrollTop = chatContainer.scrollHeight;
        }
    });
    
    onMount(async () => {
        await checkBackendConnection();
    });
    
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
        } catch (error) {
            backendConnected = false;
            addSystemMessage('❌ Could not connect to backend.');
        } finally {
            isTestingConnection = false;
        }
    }
    
    // Test backend
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
        } catch (error) {
            addAssistantMessage(`❌ **Network Error**\n\n${error.message}`);
            backendConnected = false;
        } finally {
            isProcessing = false;
        }
    }
    
    // Handle message submission - NO USER ID SENT
    async function handleSubmit() {
        if (!input.trim() || isProcessing) return;
        
        const userInput = input;
        addUserMessage(userInput);
        input = '';
        isProcessing = true;
        
        if (backendConnected) {
            try {
                // Send only user_input - backend will generate user_id automatically
                const response = await fetch('http://localhost:3001/api/intents', {
                    method: 'POST',
                    headers: { 
                        'Content-Type': 'application/json',
                        'Accept': 'application/json'
                    },
                    body: JSON.stringify({ 
                        user_input: userInput
                        // NO user_id sent - backend will generate it
                    })
                });
                
                console.log('Intent response status:', response.status);
                
                if (response.ok) {
                    const data = await response.json();
                    console.log('Intent response data:', data);
                    
                    // Clean response message without showing IDs
                    let displayMessage = data.ai_message;
                    
                    // Format the response nicely
                    if (data.status === 'completed') {
                        addAssistantMessage(`\n${displayMessage}`);
                    } else if (data.status === 'failed') {
                        addAssistantMessage(`\n${displayMessage}`);
                    } else {
                        addAssistantMessage(`📝 **Status: ${data.status}**\n\n${displayMessage}`);
                    }
                    
                    // If there's transaction data, show it in a clean way
                    if (data.data) {
                        const cleanData = { ...data.data };
                        // Remove any ID fields from display if they exist
                        delete cleanData.user_id;
                        delete cleanData.internal_id;
                        
                        if (Object.keys(cleanData).length > 0) {
                            addAssistantMessage(`📊 **Details:**\n\n${JSON.stringify(cleanData, null, 2)}`);
                        }
                    }
                } else {
                    const errorText = await response.text();
                    console.error('Intent error response:', errorText);
                    addAssistantMessage(`❌ **Request Failed**\n\nStatus: ${response.status}\nError: ${errorText}`);
                }
            } catch (error) {
                console.error('Intent fetch error:', error);
                addAssistantMessage(`❌ **Network Error**\n\n${error.message}`);
            }
        } else {
            // Simulate response
            setTimeout(() => {
                addAssistantMessage(`🤖 **Simulated Response**\n\nProcessing: "${userInput}"\n\nℹ️ Connect to backend for real transactions.`);
                isProcessing = false;
            }, 800);
        }
        isProcessing = false;
    }
    
    // Helper functions
    function addUserMessage(content: string) {
        messages = [...messages, {
            id: messages.length + 1,
            role: 'user',
            content,
            timestamp: new Date()
        }];
    }
    
    function addAssistantMessage(content: string) {
        messages = [...messages, {
            id: messages.length + 1,
            role: 'assistant',
            content,
            timestamp: new Date()
        }];
    }
    
    function addSystemMessage(content: string) {
        messages = [...messages, {
            id: messages.length + 1,
            role: 'assistant',
            content: `⚙️ ${content}`,
            timestamp: new Date()
        }];
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
        }
        
        if (testMessage) {
            input = testMessage;
            // Auto-submit after a brief delay
            setTimeout(() => handleSubmit(), 100);
        }
    }
</script>

<div class="flex h-screen bg-gray-900 text-white">
    <!-- Sidebar -->
    <div class="w-64 bg-gray-800 border-r border-gray-700 p-4">
        <div class="flex items-center gap-3 mb-8">
            <div class="w-10 h-10 bg-gradient-to-br from-blue-500 to-purple-600 rounded-lg flex items-center justify-center">
                <Bot size={20} />
            </div>
            <div>
                <h1 class="text-xl font-bold">Teemah AI</h1>
                <p class="text-sm text-gray-400">v0.1 Beta</p>
            </div>
        </div>
        
        <!-- Backend Status -->
        <div class="mb-6 p-3 rounded-lg {backendConnected ? 'bg-green-900/30 border border-green-700' : 'bg-red-900/30 border border-red-700'}">
            <div class="flex items-center gap-2 mb-2">
                <div class="w-2 h-2 rounded-full {backendConnected ? 'bg-green-500 animate-pulse' : 'bg-red-500'}"></div>
                <span class="text-sm font-medium">
                    {backendConnected ? '✅ Connected' : '❌ Offline'}
                </span>
            </div>
            
            <button
                on:click={testBackend}
                disabled={isTestingConnection || isProcessing}
                class="w-full px-3 py-2 text-sm {backendConnected ? 'bg-green-700 hover:bg-green-800' : 'bg-blue-600 hover:bg-blue-700'} rounded disabled:opacity-50 transition-colors flex items-center justify-center gap-2 mb-2"
            >
                {#if isTestingConnection}
                    <div class="w-4 h-4 border-2 border-white border-t-transparent rounded-full animate-spin"></div>
                    <span>Testing...</span>
                {:else}
                    <Server size={14} />
                    <span>{backendConnected ? 'Test Backend' : 'Connect'}</span>
                {/if}
            </button>
            
            <button 
                on:click={checkBackendConnection}
                disabled={isTestingConnection}
                class="w-full px-3 py-2 text-sm bg-gray-700 hover:bg-gray-600 rounded disabled:opacity-50 transition-colors flex items-center justify-center gap-2"
            >
                <Server size={14} />
                <span>Refresh Status</span>
            </button>
        </div>
        
        <!-- Quick Test Buttons -->
        <div class="mb-6">
            <h3 class="text-sm font-medium text-gray-400 mb-2">Quick Tests</h3>
            <div class="space-y-2">
                <button
                    on:click={() => testIntent('create')}
                    disabled={isProcessing || !backendConnected}
                    class="w-full text-left px-3 py-2 text-sm bg-blue-900/30 hover:bg-blue-900/50 rounded disabled:opacity-50 transition-colors"
                >
                    🏗️ Create Project
                </button>
                <button
                    on:click={() => testIntent('invest')}
                    disabled={isProcessing || !backendConnected}
                    class="w-full text-left px-3 py-2 text-sm bg-green-900/30 hover:bg-green-900/50 rounded disabled:opacity-50 transition-colors"
                >
                    💰 Invest
                </button>
                <button
                    on:click={() => testIntent('info')}
                    disabled={isProcessing || !backendConnected}
                    class="w-full text-left px-3 py-2 text-sm bg-purple-900/30 hover:bg-purple-900/50 rounded disabled:opacity-50 transition-colors"
                >
                    📊 Project Info
                </button>
                <button
                    on:click={() => testIntent('list')}
                    disabled={isProcessing || !backendConnected}
                    class="w-full text-left px-3 py-2 text-sm bg-yellow-900/30 hover:bg-yellow-900/50 rounded disabled:opacity-50 transition-colors"
                >
                    📋 List Projects
                </button>
            </div>
        </div>
        
        <nav class="space-y-2">
            <button 
                on:click={() => {
                    messages = [messages[0]];
                    addSystemMessage('🆕 New chat session started.');
                }}
                class="w-full text-left px-3 py-2 rounded-lg bg-blue-600 hover:bg-blue-700 transition-colors flex items-center gap-2"
            >
                <Bot size={16} />
                <span>New Chat</span>
            </button>
        </nav>
    </div>
    
    <!-- Main Content -->
    <div class="flex-1 flex flex-col">
        <!-- Status Bar - NO USER ID SHOWN -->
        <div class="border-b border-gray-700 px-6 py-3 text-sm">
            <div class="flex items-center justify-between">
                <div class="flex items-center gap-4">
                    <div class="flex items-center gap-2">
                        <div class="w-2 h-2 {backendConnected ? 'bg-green-500' : 'bg-yellow-500'} rounded-full {backendConnected ? 'animate-pulse' : ''}"></div>
                        <span>{backendConnected ? 'Backend Ready' : 'Simulation Mode'}</span>
                    </div>
                    <!-- No user ID displayed here -->
                </div>
                <div class="flex items-center gap-2 text-gray-400">
                    <AlertCircle size={14} />
                    <span>Privacy Mode: No user tracking</span>
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
                        <div class="whitespace-pre-line">{message.content}</div>
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
                            <span class="text-gray-400">Processing intent...</span>
                        </div>
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
                        placeholder="What's your intent? (e.g., 'Create a new project called MyToken' or 'Invest 0.1 ETH in project')"
                        class="flex-1 bg-gray-800 border border-gray-700 rounded-xl px-4 py-3 focus:outline-none focus:ring-2 focus:ring-blue-500 resize-none"
                        on:keydown={(e) => {
                            if (e.key === 'Enter' && !e.shiftKey) {
                                e.preventDefault();
                                handleSubmit();
                            }
                        }}
                    />
                    <button
                        type="submit"
                        disabled={!input.trim() || isProcessing}
                        class="self-end px-6 py-3 {backendConnected ? 'bg-green-600 hover:bg-green-700' : 'bg-blue-600 hover:bg-blue-700'} rounded-xl font-medium disabled:opacity-50 transition-colors flex items-center gap-2"
                    >
                        <Send size={18} />
                        <span>{backendConnected ? 'Execute' : 'Simulate'}</span>
                    </button>
                </div>
                
                <div class="mt-3 text-sm text-gray-400 flex gap-6 justify-center">
                    <div class="flex items-center gap-2">
                        <Zap size={14} />
                        <span>Fast Execution</span>
                    </div>
                    <div class="flex items-center gap-2">
                        <Shield size={14} />
                        <span>MEV Protected</span>
                    </div>
                    <div class="flex items-center gap-2">
                        <Clock size={14} />
                        <span>~2-15s</span>
                    </div>
                </div>
            </form>
        </div>
    </div>
</div>

<style>
    :global(body) {
        margin: 0;
        overflow: hidden;
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
</style>