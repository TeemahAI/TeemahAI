<script lang="ts">
    import { Zap, Clock, DollarSign, Shield, CheckCircle, XCircle, AlertTriangle } from 'lucide-svelte';
    
    export let currentIntent = null;
    
    // Mock execution data
    const executionSteps = [
        { id: 1, name: 'Intent Parsed', status: 'completed', time: '100ms' },
        { id: 2, name: 'Route Discovery', status: 'completed', time: '250ms' },
        { id: 3, name: 'Solver Auction', status: 'in-progress', time: '1.2s' },
        { id: 4, name: 'Execution', status: 'pending', time: '~5s' },
        { id: 5, name: 'Settlement', status: 'pending', time: '~2s' },
    ];
    
    const routes = [
        { protocol: 'Uniswap V3', output: '1520 USDC', cost: '$12.50', time: '15s', best: true },
        { protocol: '1inch + GMX', output: '1528 USDC', cost: '$8.20', time: '45s', best: false },
        { protocol: 'Curve + Aave', output: '1515 USDC', cost: '$5.50', time: '30s', best: false },
    ];
</script>

<div class="h-full flex flex-col">
    <!-- Header -->
    <div class="p-4 border-b border-gray-800">
        <h2 class="text-lg font-semibold flex items-center gap-2">
            <Zap size={20} class="text-blue-500" />
            Execution Details
        </h2>
        <p class="text-sm text-gray-500 mt-1">Real-time intent execution tracking</p>
    </div>
    
    <!-- Content -->
    <div class="flex-1 overflow-y-auto p-4 space-y-6">
        {#if currentIntent}
            <!-- Intent Summary -->
            <div class="bg-gray-800/30 rounded-xl p-4 border border-gray-700">
                <h3 class="font-medium mb-2">Intent Summary</h3>
                <div class="space-y-2 text-sm">
                    <div class="flex justify-between">
                        <span class="text-gray-400">Type:</span>
                        <span class="text-blue-400">Swap</span>
                    </div>
                    <div class="flex justify-between">
                        <span class="text-gray-400">From:</span>
                        <span>1 ETH</span>
                    </div>
                    <div class="flex justify-between">
                        <span class="text-gray-400">To:</span>
                        <span>USDC</span>
                    </div>
                    <div class="flex justify-between">
                        <span class="text-gray-400">Network:</span>
                        <span class="text-green-400">Arbitrum</span>
                    </div>
                </div>
            </div>
            
            <!-- Execution Steps -->
            <div>
                <h3 class="font-medium mb-3">Execution Pipeline</h3>
                <div class="space-y-3">
                    {#each executionSteps as step}
                        <div class="flex items-center gap-3">
                            <div class="w-8 h-8 rounded-full flex items-center justify-center
                                {step.status === 'completed' ? 'bg-green-500/20 text-green-400' : 
                                 step.status === 'in-progress' ? 'bg-blue-500/20 text-blue-400 animate-pulse' : 
                                 'bg-gray-800 text-gray-500'}">
                                {#if step.status === 'completed'}
                                    <CheckCircle size={16} />
                                {:else if step.status === 'in-progress'}
                                    <div class="w-3 h-3 rounded-full bg-blue-500 animate-pulse"></div>
                                {:else}
                                    <Clock size={16} />
                                {/if}
                            </div>
                            <div class="flex-1">
                                <div class="font-medium text-sm">{step.name}</div>
                                <div class="text-xs text-gray-500">{step.time}</div>
                            </div>
                            {#if step.status === 'completed'}
                                <div class="text-xs px-2 py-1 bg-green-500/20 text-green-400 rounded">Done</div>
                            {:else if step.status === 'in-progress'}
                                <div class="text-xs px-2 py-1 bg-blue-500/20 text-blue-400 rounded">Live</div>
                            {/if}
                        </div>
                    {/each}
                </div>
            </div>
            
            <!-- Route Options -->
            <div>
                <div class="flex items-center justify-between mb-3">
                    <h3 class="font-medium">Available Routes</h3>
                    <span class="text-xs text-gray-500">3 solvers bidding</span>
                </div>
                <div class="space-y-2">
                    {#each routes as route}
                        <div class="p-3 rounded-lg border {route.best ? 'border-blue-500/50 bg-blue-500/5' : 'border-gray-700 bg-gray-800/30'}">
                            <div class="flex items-center justify-between mb-2">
                                <div class="flex items-center gap-2">
                                    <span class="font-medium">{route.protocol}</span>
                                    {#if route.best}
                                        <span class="text-xs px-2 py-0.5 bg-blue-500/20 text-blue-400 rounded-full">Best</span>
                                    {/if}
                                </div>
                                <span class="font-semibold">{route.output}</span>
                            </div>
                            <div class="flex justify-between text-sm text-gray-400">
                                <div class="flex items-center gap-4">
                                    <span class="flex items-center gap-1">
                                        <DollarSign size={12} />
                                        {route.cost}
                                    </span>
                                    <span class="flex items-center gap-1">
                                        <Clock size={12} />
                                        {route.time}
                                    </span>
                                </div>
                                <button class="text-blue-400 hover:text-blue-300 text-sm">
                                    Select
                                </button>
                            </div>
                        </div>
                    {/each}
                </div>
            </div>
        {:else}
            <!-- Empty State -->
            <div class="h-full flex flex-col items-center justify-center text-center p-8">
                <div class="w-16 h-16 rounded-full bg-gray-800/50 flex items-center justify-center mb-4">
                    <Zap size={24} class="text-gray-600" />
                </div>
                <h3 class="font-medium mb-2">No Active Intent</h3>
                <p class="text-sm text-gray-500 max-w-xs">
                    Submit an intent to see real-time execution details, route options, and solver competition.
                </p>
            </div>
        {/if}
    </div>
</div>
