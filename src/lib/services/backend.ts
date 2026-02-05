import axios from 'axios';

const BACKEND_URL = 'http://localhost:3001';

const api = axios.create({
    baseURL: BACKEND_URL,
    timeout: 5000,
    headers: {
        'Content-Type': 'application/json',
    },
});

export interface HelloRequest {
    name: string;
}

export interface HelloResponse {
    message: string;
    count: number;
    timestamp: string;
}

export const backendService = {
    // Test connection
    async sayHello(name: string): Promise<HelloResponse> {
        const response = await api.post<HelloResponse>('/api/hello', { name });
        return response.data;
    },
    
    // Check if backend is running
    async checkConnection(): Promise<boolean> {
        try {
            await api.get('/api/health');
            return true;
        } catch (error) {
            console.error('Backend connection failed:', error);
            return false;
        }
    }
};
