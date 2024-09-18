import { Cluster } from '@luzid/sdk';
import fs from 'fs/promises';
import * as flatbuffers from 'flatbuffers';
import {
  signTransaction,
  lamports,
  getBase64EncodedWireTransaction,
  createKeyPairFromBytes,
  createSolanaRpc,
  generateKeyPair,
  getAddressFromPublicKey,
  getSignatureFromTransaction,
  createTransactionMessage,
  setTransactionMessageFeePayer,
  setTransactionMessageLifetimeUsingBlockhash,
  appendTransactionMessageInstruction,
  compileTransaction,
  parseBase64RpcAccount,
  Address
} from '@solana/web3.js';
import { isSolanaError, SOLANA_ERROR__JSON_RPC__SERVER_ERROR_SEND_TRANSACTION_PREFLIGHT_FAILURE } from '@solana/errors';
import { pipe } from '@solana/functional';
import { DeployV1, InputType, ProgramInputType } from 'bonsol-schemas';
import { Execute, Deploy, deploymentAddress } from '../src/';
import { randomBytes } from '@noble/hashes/utils';

const SIMPLE_IMAGE_ID = "20b9db715f989e3f57842787badafae101ce0b16202491bac1a3aebf573da0ba";
const RPC_ENDPOINT = process.env.RPC_ENDPOINT || Cluster.Development.apiUrl;
const api = createSolanaRpc(RPC_ENDPOINT);

async function getKeyPair(): Promise<CryptoKeyPair> {
  if (process.env.KP) {
    const kpBytes = await fs.readFile(process.env.KP);
    const bytes = new Uint8Array(JSON.parse(kpBytes.toString()));
    return createKeyPairFromBytes(bytes);
  }
  return generateKeyPair();
}

async function requestAirdropIfNeeded(publicKey: Address) {
  if (RPC_ENDPOINT === "http://localhost:8899") {
    await api.requestAirdrop(publicKey, lamports(1000000000n)).send();
    await new Promise((resolve) => setTimeout(resolve, 1000));
  }
}

async function deployProgram(keyPair: CryptoKeyPair) {
  const publicKey = await getAddressFromPublicKey(keyPair.publicKey);
  await requestAirdropIfNeeded(publicKey);

  const deployParams = {
    imageUrl: "https://bonsol-public-images.s3.us-east-1.amazonaws.com/simple-20b9db715f989e3f57842787badafae101ce0b16202491bac1a3aebf573da0ba",
    imageId: SIMPLE_IMAGE_ID,
    imageSize: 266608n,
    programName: "simple6",
    deployer: publicKey,
    inputs: [ProgramInputType.Public, ProgramInputType.Private]
  };

  const deployInstruction = await Deploy(deployParams);
  await sendAndConfirmTransaction(keyPair, deployInstruction);
}

async function sendAndConfirmTransaction(keyPair: CryptoKeyPair, instruction: any) {
  const publicKey = await getAddressFromPublicKey(keyPair.publicKey);
  const { value: latestBlockhash } = await api.getLatestBlockhash().send();

  const transaction = pipe(
    createTransactionMessage({ version: 0 }),
    tx => appendTransactionMessageInstruction(instruction, tx),
    tx => setTransactionMessageLifetimeUsingBlockhash(latestBlockhash, tx),
    tx => setTransactionMessageFeePayer(publicKey, tx),
    tx => compileTransaction(tx),
  );

  const signedTx = await signTransaction([keyPair], transaction);
  const signature = getSignatureFromTransaction(signedTx);

  try {
    await api.sendTransaction(getBase64EncodedWireTransaction(signedTx), { skipPreflight: true, encoding: 'base64' }).send();
    await api.confirmTransaction(signature, { commitment: "confirmed" }).send();
  } catch (e) {
    if (isSolanaError(e, SOLANA_ERROR__JSON_RPC__SERVER_ERROR_SEND_TRANSACTION_PREFLIGHT_FAILURE)) {
      console.error("Transaction failed:", e.context);
    }
    throw e;
  }

  const txInfo = await api.getTransaction(signature, { commitment: "confirmed", maxSupportedTransactionVersion: 0 }).send();
  console.log("Transaction logs:", txInfo?.meta?.logMessages);
}

describe('BonsolProgram', () => {
  beforeEach(async () => {
    const deploymentAddr = await deploymentAddress(SIMPLE_IMAGE_ID);
    const deployAccount = await api.getAccountInfo(deploymentAddr, { commitment: "confirmed", encoding: "base64" }).send();

    if (!deployAccount.value && process.env.DEPLOY === "true") {
      const keyPair = await getKeyPair();
      await deployProgram(keyPair);
    } else if (deployAccount.value) {
      const accountData = parseBase64RpcAccount(deploymentAddr, deployAccount.value);
      // The `exists` property doesn't exist on EncodedAccount
      // We can check if the account exists by verifying if it has data
      if (accountData.data.length > 0) {
        const deployData = DeployV1.getRootAsDeployV1(new flatbuffers.ByteBuffer(accountData.data as Uint8Array));
        console.log("Deployed program details:", {
          imageId: deployData.imageId(),
          programName: deployData.programName(),
          url: deployData.url(),
          size: deployData.size(),
          owner: deployData.ownerArray(),
          inputs: deployData.inputs,
        });
      }
    }

    await new Promise((resolve) => setTimeout(resolve, 1000));
  });

  it('should create valid execution requests', async () => {
    const keyPair = await getKeyPair();
    const publicKey = await getAddressFromPublicKey(keyPair.publicKey);
    await requestAirdropIfNeeded(publicKey);

    const executionId = randomBytes(5).toString();
    const input = JSON.stringify({ "attestation": "test" });
    const currentBlockHeight = await api.getBlockHeight().send();

    const executeInstruction = await Execute({
      executionId,
      imageId: SIMPLE_IMAGE_ID,
      inputs: [
        { inputType: InputType.PublicData, input: Buffer.from(input, 'utf-8') },
        { inputType: InputType.Private, input: Buffer.from("https://echoserver.dev/server?response=N4IgFgpghgJhBOBnEAuA2mkBjA9gOwBcJCBaAgTwAcIQAaEIgDwIHpKAbKASzxAF0+9AEY4Y5VKArVUDCMzogYUAlBlFEBEAF96G5QFdkKAEwAGU1qA", 'utf-8') }
      ],
      tip: 1000000,
      expiry: currentBlockHeight + 800n,
      requester: publicKey
    });

    await expect(sendAndConfirmTransaction(keyPair, executeInstruction)).resolves.not.toThrow();
  });
});
