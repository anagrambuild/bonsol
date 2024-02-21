import { ExecutionRequestV1 } from '../bonsol/execution-request-v1.js';
import { StatusV1 } from '../bonsol/status-v1.js';
export declare enum ChannelInstructionData {
    NONE = 0,
    ExecuteV1 = 1,
    StatusV1 = 2
}
export declare function unionToChannelInstructionData(type: ChannelInstructionData, accessor: (obj: ExecutionRequestV1 | StatusV1) => ExecutionRequestV1 | StatusV1 | null): ExecutionRequestV1 | StatusV1 | null;
export declare function unionListToChannelInstructionData(type: ChannelInstructionData, accessor: (index: number, obj: ExecutionRequestV1 | StatusV1) => ExecutionRequestV1 | StatusV1 | null, index: number): ExecutionRequestV1 | StatusV1 | null;
