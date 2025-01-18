import 'dotenv/config';
import express from 'express';
import cors from 'cors';
import { initializeDatabase, getExecutionRequests, getExecutionRequest, createExecutionRequest, updateExecutionRequest } from './services/database';
import { RequestStatus, ChainStatus } from './entities/ExecutionRequest';
import WebSocket from 'ws';
import { Connection } from '@solana/web3.js';
import { ChainMonitor } from './services/chainMonitor';

const app = express();
const port = process.env.PORT || 3000;

// Environment variables
const RPC_URL = process.env.SOLANA_RPC_URL || 'http://localhost:8899';
const PROGRAM_ID = process.env.BONSOL_PROGRAM_ID || 'your_program_id_here';

// Initialize chain monitor
const chainMonitor = new ChainMonitor(RPC_URL, PROGRAM_ID);

// Middleware
app.use(cors());
app.use(express.json());

// WebSocket server for real-time updates
const wss = new WebSocket.Server({ port: 8080 });

wss.on('connection', (ws) => {
  console.log('New WebSocket connection');
  
  ws.on('error', console.error);
});

// Broadcast updates to all connected clients
function broadcastUpdate(data: any) {
  wss.clients.forEach((client) => {
    if (client.readyState === WebSocket.OPEN) {
      client.send(JSON.stringify(data));
    }
  });
}

// Routes
app.get('/api/requests', async (req, res) => {
  try {
    const page = parseInt(req.query.page as string) || 1;
    const limit = parseInt(req.query.limit as string) || 20;
    const status = req.query.status as string;
    const chainStatus = req.query.chainStatus as string;

    const skip = (page - 1) * limit;
    const [requests, total] = await getExecutionRequests(skip, limit, status, chainStatus);

    res.json({
      data: requests,
      meta: {
        total,
        page,
        limit,
        totalPages: Math.ceil(total / limit)
      }
    });
  } catch (error) {
    res.status(500).json({ error: 'Failed to fetch requests' });
  }
});

app.get('/api/requests/:id', async (req, res) => {
  try {
    const request = await getExecutionRequest(req.params.id);
    if (!request) {
      return res.status(404).json({ error: 'Request not found' });
    }
    res.json(request);
  } catch (error) {
    res.status(500).json({ error: 'Failed to fetch request' });
  }
});

app.post('/api/requests', async (req, res) => {
  try {
    const request = await createExecutionRequest(req.body);
    broadcastUpdate({ type: 'NEW_REQUEST', data: request });
    res.status(201).json(request);
  } catch (error) {
    res.status(500).json({ error: 'Failed to create request' });
  }
});

app.patch('/api/requests/:id', async (req, res) => {
  try {
    const request = await updateExecutionRequest(req.params.id, req.body);
    if (!request) {
      return res.status(404).json({ error: 'Request not found' });
    }
    broadcastUpdate({ type: 'UPDATE_REQUEST', data: request });
    res.json(request);
  } catch (error) {
    res.status(500).json({ error: 'Failed to update request' });
  }
});

// Start the server
async function startServer() {
  try {
    await initializeDatabase();
    
    // Start chain monitor
    await chainMonitor.start();
    console.log('Chain monitor started');

    app.listen(port, () => {
      console.log(`Server is running on port ${port}`);
    });
  } catch (error) {
    console.error('Failed to start server:', error);
    process.exit(1);
  }
}

startServer(); 
