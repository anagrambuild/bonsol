import { Connection, PublicKey } from '@solana/web3.js';
import { createExecutionRequest, updateExecutionRequest } from './database';
import { RequestStatus, ChainStatus } from '../entities/ExecutionRequest';

export class ChainMonitor {
  private connection: Connection;
  private intervalId?: NodeJS.Timeout;
  private programId: PublicKey;

  constructor(rpcUrl: string, programId: string) {
    this.connection = new Connection(rpcUrl, 'confirmed');
    this.programId = new PublicKey(programId);
  }

  async start(pollInterval: number = 10000) {
    // Subscribe to program account changes
    this.connection.onProgramAccountChange(
      this.programId,
      async (accountInfo) => {
        try {
          await this.handleAccountChange(accountInfo);
        } catch (error) {
          console.error('Error handling account change:', error);
        }
      },
      'confirmed'
    );

    // Poll for updates to existing requests
    this.intervalId = setInterval(async () => {
      try {
        await this.checkPendingRequests();
      } catch (error) {
        console.error('Error checking pending requests:', error);
      }
    }, pollInterval);
  }

  stop() {
    if (this.intervalId) {
      clearInterval(this.intervalId);
    }
  }

  private async handleAccountChange(accountInfo: any) {
    const { accountId, accountData } = accountInfo;
    const pubkey = accountId.toBase58();

    try {
      // Parse account data based on the Bonsol schema
      // This is a simplified example - you'll need to implement the actual parsing logic
      const data = accountData.data;
      
      // Create or update request in our database
      const request = {
        id: pubkey,
        executionAddress: pubkey,
        status: this.parseRequestStatus(data),
        chainStatus: this.parseChainStatus(data),
        imageId: this.parseImageId(data),
        requestData: this.parseRequestData(data),
        proverId: this.parseProverId(data),
        result: this.parseResult(data),
        error: this.parseError(data),
      };

      const existingRequest = await updateExecutionRequest(pubkey, request);
      if (!existingRequest) {
        await createExecutionRequest(request);
      }
    } catch (error) {
      console.error(`Error processing account ${pubkey}:`, error);
    }
  }

  private async checkPendingRequests() {
    // This method will be called periodically to check the status of pending requests
    // Implementation will be similar to the existing checkPendingRequests method
  }

  // Helper methods to parse account data
  private parseRequestStatus(data: Buffer): RequestStatus {
    // Implement parsing logic based on the Bonsol schema
    return RequestStatus.PENDING;
  }

  private parseChainStatus(data: Buffer): ChainStatus {
    // Implement parsing logic based on the Bonsol schema
    return ChainStatus.UNCLAIMED;
  }

  private parseImageId(data: Buffer): string {
    // Implement parsing logic based on the Bonsol schema
    return '';
  }

  private parseRequestData(data: Buffer): string {
    // Implement parsing logic based on the Bonsol schema
    return '{}';
  }

  private parseProverId(data: Buffer): string | undefined {
    // Implement parsing logic based on the Bonsol schema
    return undefined;
  }

  private parseResult(data: Buffer): string | undefined {
    // Implement parsing logic based on the Bonsol schema
    return undefined;
  }

  private parseError(data: Buffer): string | undefined {
    // Implement parsing logic based on the Bonsol schema
    return undefined;
  }
} 
