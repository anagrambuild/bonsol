# Bonsol Proof Explorer

A web application for tracking and monitoring Bonsol execution requests, from submission to completion.

## Features

- Real-time tracking of execution request status
- Filtering and pagination of requests
- Detailed view of each request
- Chain status monitoring
- WebSocket updates for real-time changes

## Project Structure

```
proof-explorer/
├── backend/               # Express.js + TypeScript backend
│   ├── src/
│   │   ├── entities/     # Database entities
│   │   ├── services/     # Business logic
│   │   └── index.ts      # Main application file
│   ├── package.json
│   └── tsconfig.json
└── frontend/             # React + TypeScript frontend
    ├── src/
    │   ├── components/   # React components
    │   ├── services/     # API services
    │   ├── types/        # TypeScript types
    │   └── App.tsx       # Main application component
    ├── package.json
    └── tsconfig.json
```

## Prerequisites

- Node.js (v16 or later)
- npm or yarn
- SQLite3

## Setup

1. Clone the repository:
   ```bash
   git clone https://github.com/your-org/bonsol
   cd bonsol/proof-explorer
   ```

2. Install backend dependencies:
   ```bash
   cd backend
   npm install
   ```

3. Install frontend dependencies:
   ```bash
   cd ../frontend
   npm install
   ```

4. Create a `.env` file in the backend directory:
   ```
   PORT=3000
   DATABASE_URL=database.sqlite
   SOLANA_RPC_URL=your_solana_rpc_url
   ```

5. Create a `.env` file in the frontend directory:
   ```
   VITE_API_URL=http://localhost:3000/api
   VITE_WS_URL=ws://localhost:8080
   ```

## Running the Application

1. Start the backend:
   ```bash
   cd backend
   npm run dev
   ```

2. Start the frontend:
   ```bash
   cd frontend
   npm run dev
   ```

3. Open your browser and navigate to `http://localhost:5173`

## Development

### Backend

The backend uses:
- Express.js for the REST API
- TypeORM for database management
- WebSocket for real-time updates
- SQLite for data storage

### Frontend

The frontend uses:
- React with TypeScript
- Chakra UI for components
- React Query for data fetching
- React Router for routing
- WebSocket for real-time updates

## Testing

To run tests:

```bash
# Backend tests
cd backend
npm test

# Frontend tests
cd frontend
npm test
```

## Contributing

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under the MIT License - see the LICENSE file for details. 
