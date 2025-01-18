import fs from 'fs/promises';
import path from 'path';
import { ExecutionRequest, RequestStatus, ChainStatus } from '../entities/ExecutionRequest';

const DB_FILE = path.join(process.cwd(), 'database.json');

interface Database {
  requests: ExecutionRequest[];
}

let db: Database = {
  requests: [],
};

async function saveDatabase() {
  const dbToSave = {
    requests: db.requests.map(request => ({
      ...request,
      createdAt: request.createdAt.toISOString(),
      updatedAt: request.updatedAt.toISOString(),
    })),
  };
  await fs.writeFile(DB_FILE, JSON.stringify(dbToSave, null, 2));
}

async function loadDatabase() {
  try {
    const data = await fs.readFile(DB_FILE, 'utf-8');
    const parsedDb = JSON.parse(data);
    db = {
      requests: parsedDb.requests.map((request: any) => ({
        ...request,
        createdAt: new Date(request.createdAt),
        updatedAt: new Date(request.updatedAt),
      })),
    };
  } catch (error) {
    // If file doesn't exist, create it with empty database
    await saveDatabase();
  }
}

export async function initializeDatabase() {
  try {
    await loadDatabase();
    console.log('Database has been initialized!');
  } catch (error) {
    console.error('Error during database initialization:', error);
    throw error;
  }
}

export async function getExecutionRequests(
  skip: number = 0,
  take: number = 20,
  status?: string,
  chainStatus?: string
): Promise<[ExecutionRequest[], number]> {
  let filteredRequests = db.requests;

  if (status) {
    filteredRequests = filteredRequests.filter(r => r.status === status);
  }

  if (chainStatus) {
    filteredRequests = filteredRequests.filter(r => r.chainStatus === chainStatus);
  }

  // Sort by createdAt in descending order
  filteredRequests.sort((a, b) => b.createdAt.getTime() - a.createdAt.getTime());

  const total = filteredRequests.length;
  const items = filteredRequests.slice(skip, skip + take);

  return [items, total];
}

export async function getExecutionRequest(id: string): Promise<ExecutionRequest | null> {
  return db.requests.find(r => r.id === id) || null;
}

export async function createExecutionRequest(data: Partial<ExecutionRequest>): Promise<ExecutionRequest> {
  const now = new Date();
  const request: ExecutionRequest = {
    id: data.id || crypto.randomUUID(),
    status: data.status || RequestStatus.PENDING,
    chainStatus: data.chainStatus || ChainStatus.UNCLAIMED,
    requestData: data.requestData || '{}',
    createdAt: now,
    updatedAt: now,
    proverId: data.proverId,
    result: data.result,
    error: data.error,
    imageId: data.imageId,
    requesterAddress: data.requesterAddress,
    executionAddress: data.executionAddress,
  };

  db.requests.push(request);
  await saveDatabase();
  return request;
}

export async function updateExecutionRequest(
  id: string,
  data: Partial<ExecutionRequest>
): Promise<ExecutionRequest | null> {
  const index = db.requests.findIndex(r => r.id === id);
  if (index === -1) return null;

  const request = db.requests[index];
  const updatedRequest: ExecutionRequest = {
    ...request,
    ...data,
    updatedAt: new Date(),
  };

  db.requests[index] = updatedRequest;
  await saveDatabase();
  return updatedRequest;
} 
