<script lang="ts">
    import { Send, Sparkles, Zap, Shield, Clock } from 'lucide-svelte';
    import { createEventDispatcher } from 'svelte';
    
    const dispatch = createEventDispatcher();
    
    export let messages = [];
    
    let input = '';
    
    const quickSuggestions = [
        'Swap 1 ETH for USDC on Arbitrum',
        'Bridge 0.5 ETH to Base',
        'Maximize yield on 10K USDC',
        'Exit all positions if ETH drops 10%',
        'Create DCA strategy for BTC',
    ];
    
    function handleSubmit(e: Event) {
        e.preventDefault();
        if (!input.trim()) return;
        
        dispatch('submit', input);
        input = '';
    }
    
    function handleSuggestion(suggestion: string) {
        dispatch('submit', suggestion);
    }
</script>

<div class="flex-1 flex flex-col h-full">
    <!-- Messages Area -->
    <div class="flex-1 overflow-y-auto p-4 space-y-6">
        {#each messages as message}
            <div class="{message.type === 'user' ? 'flex justify-end' : ''}">
                <div class="max-w-2xl {message.type === 'user' ? 'ml-auto' : ''}">
                    <div class="flex items-center gap-2 mb-1">
                        {#if message.type === 'assistant'}
                            <div class="w-6 h-6 rounded-full bg-gradient-to-br from-blue-500 to-purple-600 flex items-center justify-center">
                                <Sparkles size={12} class="text-white" />
                            </div>
                            <span class="text-sm font-medium">Teemah AI</span>
                        {:else if message.type === 'user'}
                            <span class="text-sm font-medium text-blue-400">You</span>
                        {/if}
                        <span class="text-xs text-gray-500">
                            {message.timestamp?.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })}
                        </span>
                    </div>
                    
                    <div class="{message.type === 'user' 
                        ? 'bg-blue-600/20 border border-blue-500/30' 
                        : 'bg-gray-800/50 border border-gray-700'} 
                        rounded-xl p-4">
                        {#if typeof message.content === 'string'}
                            <div class="whitespace-pre-wrap text-gray-200">{message.content}</div>
                        {:else}
                            <!-- Handle object content -->
                            <pre class="text-sm text-gray-300">{JSON.stringify(message.content, null, 2)}</pre>
                        {/if}
                    </div>
                </div>
            </div>
        {/each}
    </div>
    
    <!-- Quick Suggestions -->
    {#if messages.length === 1}
        <div class="px-4 pb-4">
            <div class="text-sm text-gray-500 mb-2">Try asking:</div>
            <div class="flex flex-wrap gap-2">
                {#each quickSuggestions as suggestion}
                    <button
                        on:click={() => handleSuggestion(suggestion)}
                        class="px-3 py-2 text-sm bg-gray-800/50 hover:bg-gray-800 rounded-lg border border-gray-700 
                               text-gray-300 hover:text-white transition-colors"
                    >
                        {suggestion}
                    </button>
                {/each}
            </div>
        </div>
    {/if}
    
    <!-- Input Area -->
    <form on:submit={handleSubmit} class="border-t border-gray-800 p-4">
        <div class="max-w-3xl mx-auto">
            <div class="flex gap-2">
                <div class="flex-1 relative">
                    <textarea
                        bind:value={input}
                        rows="2"
                        placeholder="What's your intent? (e.g., 'Swap 1 ETH for USDC on Arbitrum with max 1% slippage')"
                        class="w-full bg-gray-800/50 border border-gray-700 rounded-xl px-4 py-3 pr-12 
                               focus:outline-none focus:ring-2 focus:ring-blue-500/50 focus:border-transparent
                               resize-none text-gray-200 placeholder-gray-500"
                        on:keydown={(e) => {
                            if (e.key === 'Enter' && !e.shiftKey) {
                                e.preventDefault();
                                handleSubmit(e);
                            }
                        }}
                    />
                    <div class="absolute right-3 bottom-3 flex items-center gap-2">
                        <div class="flex items-center gap-1 text-xs text-gray-500">
                            <Shield size={12} />
                            <span>Private</span>
                        </div>
                    </div>
                </div>
                
                <button
                    type="submit"
                    disabled={!input.trim()}
                    class="self-end px-6 py-3 bg-gradient-to-r from-blue-600 to-purple-600 
                           hover:from-blue-700 hover:to-purple-700 rounded-xl font-medium
                           disabled:opacity-50 disabled:cursor-not-allowed transition-all
                           flex items-center gap-2"
                >
                    <Send size={18} />
                    <span class="hidden sm:inline">Execute</span>
                </button>
            </div>
            
            <div class="mt-3 flex items-center justify-between text-sm text-gray-500">
                <div class="flex items-center gap-4">
                    <div class="flex items-center gap-1">
                        <Zap size={14} />
                        <span>Fast Execution</span>
                    </div>
                    <div class="flex items-center gap-1">
                        <Shield size={14} />
                        <span>MEV Protected</span>
                    </div>
                </div>
                <div class="flex items-center gap-1">
                    <Clock size={14} />
                    <span>~2-15s execution</span>
                </div>
            </div>
        </div>
    </form>
</div>
