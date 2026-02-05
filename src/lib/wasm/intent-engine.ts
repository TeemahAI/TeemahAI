import type { IntentParser, estimate_intent_cost } from '$wasm/web_bindings';

export interface Intent {
	id: string;
	user_input: string;
	parsed: ParsedIntent;
	constraints: Constraint[];
	status: 'pending' | 'auctioning' | 'executing' | 'completed' | 'failed';
}

export interface ParsedIntent {
	type: 'swap' | 'bridge' | 'yield' | 'limit' | 'composite';
	parameters: Record<string, any>;
	routes: Route[];
}

export interface Constraint {
	type: 'max_slippage' | 'deadline' | 'min_output' | 'max_cost';
	value: string | number;
}

export interface Route {
	protocol: string;
	steps: Step[];
	estimated_output: string;
	estimated_cost: string;
	execution_time: number;
}

export interface Step {
	action: string;
	parameters: Record<string, any>;
}

class IntentWASM {
	private parser: IntentParser | null = null;
	private initialized = false;
	
	async init() {
		if (this.initialized) return;
		
		try {
			// Dynamically import WASM
			const module = await import('$wasm/web_bindings');
			await module.default();
			
			// @ts-ignore - WASM bindings are dynamically loaded
			this.parser = new module.IntentParser();
			this.initialized = true;
			
			console.log('WASM Intent Engine initialized');
		} catch (error) {
			console.error('Failed to initialize WASM:', error);
			throw error;
		}
	}
	
	async parse(input: string): Promise<Intent> {
		await this.init();
		if (!this.parser) throw new Error('WASM parser not initialized');
		
		try {
			const result = this.parser.parse(input);
			const intent: Intent = JSON.parse(JSON.stringify(result));
			return intent;
		} catch (error) {
			console.error('Failed to parse intent:', error);
			throw new Error(`Failed to parse intent: ${error}`);
		}
	}
	
	async validate(intent: Intent): Promise<{ valid: boolean; errors: string[] }> {
		await this.init();
		if (!this.parser) throw new Error('WASM parser not initialized');
		
		try {
			const result = this.parser.validate(JSON.stringify(intent));
			return JSON.parse(JSON.stringify(result));
		} catch (error) {
			return { valid: false, errors: ['Validation failed'] };
		}
	}
	
	async estimateCost(intent: Intent): Promise<{
		min_cost: string;
		max_cost: string;
		confidence: number;
		currency: string;
	}> {
		await this.init();
		
		try {
			// @ts-ignore - WASM function
			const result = await estimate_intent_cost(JSON.stringify(intent));
			return JSON.parse(JSON.stringify(result));
		} catch (error) {
			console.error('Failed to estimate cost:', error);
			return {
				min_cost: '0.01',
				max_cost: '0.05',
				confidence: 0.5,
				currency: 'ETH'
			};
		}
	}
	
	isReady(): boolean {
		return this.initialized && this.parser !== null;
	}
}

export const intentEngine = new IntentWASM();
