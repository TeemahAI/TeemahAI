<script lang="ts">
    import { Bot, History, Zap, Settings, Wallet, TrendingUp, Users, FileCode } from 'lucide-svelte';
    
    const navItems = [
        { icon: Bot, label: 'New Chat', active: true },
        { icon: History, label: 'History' },
        { icon: Zap, label: 'Quick Intents' },
        { icon: TrendingUp, label: 'Dashboard' },
        { icon: Users, label: 'Solver Network' },
        { icon: FileCode, label: 'Intent Templates' },
        { icon: Wallet, label: 'Wallet' },
        { icon: Settings, label: 'Settings' },
    ];
    
    const recentIntents = [
        { id: 1, text: 'Swap 1 ETH for USDC', time: '2 min ago' },
        { id: 2, text: 'Bridge to Arbitrum', time: '1 hour ago' },
        { id: 3, text: 'Maximize yield on USDC', time: '3 hours ago' },
    ];
</script>

<aside class="w-64 border-r border-gray-800 bg-gray-900/50 flex flex-col h-full">
    <!-- Logo -->
    <div class="p-4 border-b border-gray-800">
        <div class="flex items-center gap-3">
            <div class="w-8 h-8 bg-gradient-to-br from-blue-500 to-purple-600 rounded-lg flex items-center justify-center">
                <Bot size={18} class="text-white" />
            </div>
            <div>
                <h1 class="text-lg font-semibold">Teemah AI</h1>
                <p class="text-xs text-gray-500">Intent Engine v0.1</p>
            </div>
        </div>
    </div>
    
    <!-- Navigation -->
    <nav class="flex-1 p-2">
        <div class="space-y-1">
            {#each navItems as item}
                <button
                    class="w-full flex items-center gap-3 px-3 py-2 rounded-lg text-sm
                           {item.active ? 'bg-gray-800 text-white' : 'text-gray-400 hover:bg-gray-800/50 hover:text-gray-300'}"
                >
                    <item.icon size={16} />
                    <span>{item.label}</span>
                </button>
            {/each}
        </div>
        
        <!-- Recent Intents -->
        <div class="mt-8 px-3">
            <h3 class="text-xs font-medium text-gray-500 uppercase tracking-wider mb-2">
                Recent Intents
            </h3>
            <div class="space-y-1">
                {#each recentIntents as intent}
                    <div class="text-sm p-2 rounded hover:bg-gray-800/30 cursor-pointer">
                        <div class="text-gray-300 truncate">{intent.text}</div>
                        <div class="text-xs text-gray-500">{intent.time}</div>
                    </div>
                {/each}
            </div>
        </div>
    </nav>
    
    <!-- Footer -->
    <div class="p-4 border-t border-gray-800">
        <div class="flex items-center justify-between mb-2">
            <span class="text-xs text-gray-500">Status</span>
            <div class="flex items-center gap-1">
                <div class="w-2 h-2 rounded-full bg-green-500"></div>
                <span class="text-xs text-green-400">Live</span>
            </div>
        </div>
        <div class="text-xs text-gray-400">
            Connected to <span class="text-blue-400">12 chains</span>
        </div>
    </div>
</aside>
