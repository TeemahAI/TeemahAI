<script lang="ts">
    export let isWasmReady = false;
    
    // In Svelte 4, use reactive statements instead of $derived
    $: connectionStatus = isWasmReady ? 'connected' : 'connecting';
    $: nodes = 42; // Mock
    $: solvers = 8; // Mock
</script>

<div class="border-b border-gray-800 bg-gray-900/50 backdrop-blur-sm px-4 py-2 text-sm">
    <div class="flex items-center justify-between">
        <div class="flex items-center gap-4">
            <div class="flex items-center gap-2">
                <div class="w-2 h-2 rounded-full {isWasmReady ? 'bg-green-500 animate-pulse' : 'bg-yellow-500'}"></div>
                <span class="text-gray-400">Intent Engine:</span>
                <span class="{isWasmReady ? 'text-green-400' : 'text-yellow-400'}">
                    {isWasmReady ? 'Ready' : 'Initializing...'}
                </span>
            </div>
            
            <div class="hidden md:flex items-center gap-4">
                <div class="flex items-center gap-2">
                    <div class="w-2 h-2 rounded-full bg-blue-500"></div>
                    <span class="text-gray-400">Nodes:</span>
                    <span class="text-blue-400">{nodes}</span>
                </div>
                
                <div class="flex items-center gap-2">
                    <div class="w-2 h-2 rounded-full bg-purple-500"></div>
                    <span class="text-gray-400">Solvers:</span>
                    <span class="text-purple-400">{solvers}</span>
                </div>
            </div>
        </div>
        
        <div class="flex items-center gap-3">
            <div class="hidden sm:block text-gray-500 text-xs">
                Network: <span class="text-gray-300">Arbitrum</span>
            </div>
            <div class="w-32 h-1.5 bg-gray-800 rounded-full overflow-hidden">
                <div class="h-full bg-gradient-to-r from-blue-500 to-purple-500" style="width: 75%"></div>
            </div>
        </div>
    </div>
</div>
