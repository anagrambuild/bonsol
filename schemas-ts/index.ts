export * from './channel_instruction';


export enum ExitCode {
  Success = 0,
  VerifyError = 1,
  ProvingError = 2,
  InputError = 3
}
