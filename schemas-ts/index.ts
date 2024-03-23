export * from './channel_instruction';
export * from './input_type';
export * from './claim_v1';
export * from './deploy_v1';
export * from './execution_request_v1';
export * from './status_v1';

export enum ExitCode {
  Success = 0,
  VerifyError = 1,
  ProvingError = 2,
  InputError = 3
}
