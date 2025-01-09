import { initializeDatabase, createExecutionRequest } from '../services/database';
import { RequestStatus, ChainStatus } from '../entities/ExecutionRequest';

async function seedDatabase() {
  await initializeDatabase();

  const sampleRequests = [
    {
      imageId: 'image123',
      requestData: JSON.stringify({ input: 'test1' }),
      status: RequestStatus.COMPLETED,
      chainStatus: ChainStatus.POSTED,
      proverId: 'prover1',
      result: 'success',
      requesterAddress: 'addr1',
      executionAddress: 'exec1',
    },
    {
      imageId: 'image456',
      requestData: JSON.stringify({ input: 'test2' }),
      status: RequestStatus.IN_PROGRESS,
      chainStatus: ChainStatus.CLAIMED,
      proverId: 'prover2',
      requesterAddress: 'addr2',
      executionAddress: 'exec2',
    },
    {
      imageId: 'image789',
      requestData: JSON.stringify({ input: 'test3' }),
      status: RequestStatus.PENDING,
      chainStatus: ChainStatus.UNCLAIMED,
      requesterAddress: 'addr3',
      executionAddress: 'exec3',
    },
  ];

  for (const request of sampleRequests) {
    await createExecutionRequest(request);
    console.log(`Created request with image ID: ${request.imageId}`);
  }

  console.log('Database seeded successfully!');
  process.exit(0);
}

seedDatabase().catch(error => {
  console.error('Error seeding database:', error);
  process.exit(1);
}); 
