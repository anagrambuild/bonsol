import { Cluster } from "@luzid/sdk";
import fs from "fs/promises";
import * as flatbuffers from "flatbuffers";
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
} from "@solana/web3.js";
import {
  isSolanaError,
  SOLANA_ERROR__JSON_RPC__SERVER_ERROR_SEND_TRANSACTION_PREFLIGHT_FAILURE,
} from "@solana/errors";

import { pipe } from "@solana/functional";
import { DeployV1, InputType, ProgramInputType } from "bonsol-schemas";
import { Execute, Deploy, deploymentAddress } from "../src/";
import { randomBytes } from "@noble/hashes/utils";

async function getKeyPair(): Promise<CryptoKeyPair> {
  if (process.env.KP) {
    const kpBytes = await fs.readFile(process.env.KP);
    const bytes = new Uint8Array(JSON.parse(kpBytes.toString()));
    return createKeyPairFromBytes(bytes);
  }
  return generateKeyPair();
}

async function requestAirdropIfLocalNetwork(api: any, publicKey: string) {
  if (
    !process.env.RPC_ENDPOINT ||
    process.env.RPC_ENDPOINT === "http://localhost:8899"
  ) {
    await api.requestAirdrop(publicKey, lamports(1000000000n)).send();
  }
}

async function sendAndConfirmTransaction(
  api: any,
  instruction: any,
  keyPair: CryptoKeyPair
) {
  const publicKey = await getAddressFromPublicKey(keyPair.publicKey);
  const ctx = await api.getLatestBlockhash().send();
  const txn = pipe(
    createTransactionMessage({ version: 0 }),
    (tx) => appendTransactionMessageInstruction(instruction, tx),
    (tx) => setTransactionMessageLifetimeUsingBlockhash(ctx.value, tx),
    (tx) => setTransactionMessageFeePayer(publicKey, tx),
    (tx) => compileTransaction(tx)
  );

  const signed = await signTransaction([keyPair], txn);
  try {
    await api
      .sendTransaction(getBase64EncodedWireTransaction(signed), {
        skipPreflight: true,
        encoding: "base64",
      })
      .send();
    await api
      .confirmTransaction(getSignatureFromTransaction(signed), {
        commitment: "confirmed",
      })
      .send();
    return signed;
  } catch (e) {
    if (
      isSolanaError(
        e,
        SOLANA_ERROR__JSON_RPC__SERVER_ERROR_SEND_TRANSACTION_PREFLIGHT_FAILURE
      )
    ) {
      console.log(e.context);
    }
    throw e;
  }
}

async function logTransactionLogs(api: any, signed: any) {
  const tx = await api
    .getTransaction(getSignatureFromTransaction(signed), {
      commitment: "confirmed",
      maxSupportedTransactionVersion: 0,
    })
    .send();
  console.log("logs", tx?.meta?.logMessages);
}

describe("BonsolProgram", () => {
  const SIMPLE_IMAGE_ID =
    "20b9db715f989e3f57842787badafae101ce0b16202491bac1a3aebf573da0ba";
  const api = createSolanaRpc(
    process.env.RPC_ENDPOINT || Cluster.Development.apiUrl
  );
  let keyPair: CryptoKeyPair;

  async function deployIfNeeded() {
    const depl = await deploymentAddress(SIMPLE_IMAGE_ID);
    const deployAccount = await api
      .getAccountInfo(depl, { commitment: "confirmed", encoding: "base64" })
      .send();

    if (!deployAccount.value && process.env.DEPLOY === "true") {
      await deployProgram();
    } else if (deployAccount.value) {
      logDeploymentInfo(deployAccount.value);
    }
  }

  async function deployProgram() {
    const pub = await getAddressFromPublicKey(keyPair.publicKey);
    await requestAirdropIfLocalNetwork(api, pub);

    try {
      const deployInstruction = await Deploy({
        imageUrl: `https://bonsol-public-images.s3.us-east-1.amazonaws.com/simple-${SIMPLE_IMAGE_ID}`,
        imageId: SIMPLE_IMAGE_ID,
        imageSize: 266608n,
        programName: "simple6",
        deployer: pub,
        inputs: [ProgramInputType.Public, ProgramInputType.Private],
      });

      const signed = await sendAndConfirmTransaction(
        api,
        deployInstruction,
        keyPair
      );
      await logTransactionLogs(api, signed);
    } catch (error) {
      console.error("Error deploying program:", error);
      throw error;
    }
  }

  async function logDeploymentInfo(deployAccountValue: any) {
    try {
      const ea = parseBase64RpcAccount(
        await deploymentAddress(SIMPLE_IMAGE_ID),
        deployAccountValue
      );
      if (!ea.data) {
        console.log("Image not deployed");
        return;
      }
      const buf = new flatbuffers.ByteBuffer(ea.data as Uint8Array);
      const dp = DeployV1.getRootAsDeployV1(buf);
      console.log("Deployment Info:");
      console.log("Image ID:", dp.imageId());
      console.log("Program Name:", dp.programName());
      console.log("URL:", dp.url());
      console.log("Size:", dp.size());
      console.log("Owner Array:", dp.ownerArray());
      console.log("Inputs:", dp.inputs);
    } catch (error) {
      console.error("Error logging deployment info:", error);
    }
  }

  beforeEach(async () => {
    try {
      keyPair = await getKeyPair();
      await deployIfNeeded();
    } catch (error) {
      console.error("Error in beforeEach:", error);
      throw error;
    } finally {
      await new Promise((resolve) => setTimeout(resolve, 1000));
    }
  });

  it("should create valid execution requests", async () => {
    try {
      const pub = await getAddressFromPublicKey(keyPair.publicKey);
      await requestAirdropIfLocalNetwork(api, pub);

      const executionId = randomBytes(5).toString();
      const input = JSON.stringify({ attestation: "test" });
      const blockHeight = await api.getBlockHeight().send();

      const executeInstruction = await Execute({
        executionId,
        imageId: SIMPLE_IMAGE_ID,
        inputs: [
          {
            inputType: InputType.PublicData,
            input: Buffer.from(input, "utf-8"),
          },
          {
            inputType: InputType.Private,
            input: Buffer.from(
              "https://echoserver.dev/server?response=N4IgFgpghgJhBOBnEAuA2mkBjA9gOwBcJCBaAgTwAcIQAaEIgDwIHpKAbKASzxAF0+9AEY4Y5VKArVUDCMzogYUAlBlFEBEAF96G5QFdkKAEwAGU1qA",
              "utf-8"
            ),
          },
        ],
        tip: 1000000,
        expiry: blockHeight + 800n, // 400s from now
        requester: pub,
      });

      const signed = await sendAndConfirmTransaction(
        api,
        executeInstruction,
        keyPair
      );
      await logTransactionLogs(api, signed);
    } catch (error) {
      console.error("Error in execution test:", error);
      throw error;
    } finally {
      await new Promise((resolve) => setTimeout(resolve, 1000));
    }
  });
});
