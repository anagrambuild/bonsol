import { Entity, PrimaryColumn, Column, CreateDateColumn, UpdateDateColumn } from 'typeorm';

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

@Entity()
export class ExecutionRequest {
  constructor() {
    this.status = RequestStatus.PENDING;
    this.chainStatus = ChainStatus.UNCLAIMED;
    this.requestData = '{}';
  }

  @PrimaryColumn()
  id!: string;

  @Column({
    type: 'simple-enum',
    enum: RequestStatus,
    default: RequestStatus.PENDING
  })
  status!: RequestStatus;

  @Column({
    type: 'simple-enum',
    enum: ChainStatus,
    default: ChainStatus.UNCLAIMED
  })
  chainStatus!: ChainStatus;

  @Column({ nullable: true })
  proverId?: string;

  @Column('text')
  requestData!: string;

  @Column({ nullable: true })
  result?: string;

  @Column({ nullable: true })
  error?: string;

  @Column({ nullable: true })
  imageId?: string;

  @Column({ nullable: true })
  requesterAddress?: string;

  @Column({ nullable: true })
  executionAddress?: string;

  @CreateDateColumn()
  createdAt!: Date;

  @UpdateDateColumn()
  updatedAt!: Date;
} 
