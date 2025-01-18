export enum RequestStatus {
  PENDING = 'pending',
  IN_PROGRESS = 'in_progress',
  COMPLETED = 'completed',
  FAILED = 'failed'
}

export enum ChainStatus {
  UNCLAIMED = 'unclaimed',
  CLAIMED = 'claimed',
  EXECUTED = 'executed',
  POSTED = 'posted'
}

export interface ExecutionRequest {
  id: string;
  status: RequestStatus;
  chainStatus: ChainStatus;
  proverId?: string;
  requestData: string;
  result?: string;
  error?: string;
  imageId?: string;
  requesterAddress?: string;
  executionAddress?: string;
  createdAt: string;
  updatedAt: string;
}

export interface PaginatedResponse<T> {
  data: T[];
  meta: {
    total: number;
    page: number;
    limit: number;
    totalPages: number;
  };
}

export interface RequestFilters {
  status?: RequestStatus;
  chainStatus?: ChainStatus;
  page: number;
  limit: number;
} 
