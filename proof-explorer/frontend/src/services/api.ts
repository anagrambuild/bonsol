import axios from 'axios';
import { ExecutionRequest, PaginatedResponse, RequestFilters } from '../types';

const API_URL = import.meta.env.VITE_API_URL || 'http://localhost:3000/api';

const api = axios.create({
  baseURL: API_URL,
  headers: {
    'Content-Type': 'application/json',
  },
});

export const getRequests = async (filters: RequestFilters): Promise<PaginatedResponse<ExecutionRequest>> => {
  const { data } = await api.get('/requests', {
    params: filters,
  });
  return data;
};

export const getRequest = async (id: string): Promise<ExecutionRequest> => {
  const { data } = await api.get(`/requests/${id}`);
  return data;
};

export const createRequest = async (requestData: Partial<ExecutionRequest>): Promise<ExecutionRequest> => {
  const { data } = await api.post('/requests', requestData);
  return data;
};

export const updateRequest = async (id: string, requestData: Partial<ExecutionRequest>): Promise<ExecutionRequest> => {
  const { data } = await api.patch(`/requests/${id}`, requestData);
  return data;
};

// WebSocket connection for real-time updates
export class WebSocketService {
  private ws: WebSocket | null = null;
  private reconnectAttempts = 0;
  private maxReconnectAttempts = 5;
  private listeners: ((data: any) => void)[] = [];

  constructor() {
    this.connect();
  }

  private connect() {
    const wsUrl = import.meta.env.VITE_WS_URL || 'ws://localhost:8080';
    this.ws = new WebSocket(wsUrl);

    this.ws.onmessage = (event) => {
      const data = JSON.parse(event.data);
      this.listeners.forEach(listener => listener(data));
    };

    this.ws.onclose = () => {
      if (this.reconnectAttempts < this.maxReconnectAttempts) {
        setTimeout(() => {
          this.reconnectAttempts++;
          this.connect();
        }, 1000 * Math.pow(2, this.reconnectAttempts));
      }
    };

    this.ws.onerror = (error) => {
      console.error('WebSocket error:', error);
    };
  }

  addListener(listener: (data: any) => void) {
    this.listeners.push(listener);
  }

  removeListener(listener: (data: any) => void) {
    this.listeners = this.listeners.filter(l => l !== listener);
  }

  close() {
    if (this.ws) {
      this.ws.close();
    }
  }
} 
