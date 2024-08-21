import { Cluster, LuzidSdk } from '@luzid/sdk'
import fs from 'fs/promises';
import * as flatbuffers from 'flatbuffers';
import {
  signTransaction,
  lamports,
  getBase64EncodedWireTransaction,
  createSolanaRpcSubscriptions,
  createKeyPairFromBytes,
  createSolanaRpc, generateKeyPair, createDefaultRpcTransport, address, getAddressFromPublicKey, signature, getSignatureFromTransaction,
  createTransactionMessage,
  setTransactionMessageFeePayer,
  setTransactionMessageLifetimeUsingBlockhash,
  appendTransactionMessageInstruction,
  compileTransaction,
  parseBase64RpcAccount
} from '@solana/web3.js';
import {
  isSolanaError,
  SOLANA_ERROR__JSON_RPC__SERVER_ERROR_SEND_TRANSACTION_PREFLIGHT_FAILURE
} from '@solana/errors';

import { pipe } from '@solana/functional';
import { createDefaultRpcSubscriptionsTransport } from '@solana/web3.js';
import { DeployV1, InputType, ProgramInputType } from 'bonsol-schemas';
import { Execute, Deploy, deploymentAddress } from '../src/';
import { randomBytes } from '@noble/hashes/utils';

describe('BonsolProgram', () => {
  const SIMPLE_IMAGE_ID = "20b9db715f989e3f57842787badafae101ce0b16202491bac1a3aebf573da0ba";
  const api = createSolanaRpc(process.env.RPC_ENDPOINT || Cluster.Development.apiUrl);
  const sub = createSolanaRpcSubscriptions((process.env.RPC_ENDPOINT || Cluster.Development.apiUrl).replace('http', 'ws').replace("8899", "8900"))


  async function kp(): Promise<CryptoKeyPair> {
    if (process.env.KP != null) {
      let kpbytes = await fs.readFile(process.env.KP)
      let js = JSON.parse(kpbytes.toString())
      let bytes = new Uint8Array(js)
      return createKeyPairFromBytes(bytes)
    }
    return generateKeyPair();
  }

  beforeEach(async () => {
    const depl = await deploymentAddress(SIMPLE_IMAGE_ID);

    const deployAccount = await api.getAccountInfo(depl, { commitment: "confirmed", encoding: "base64" }).send();
    if (!deployAccount.value && process.env.DEPLOY == "true") {
      //deploy
      const keyPair = await kp();
      const pub = await getAddressFromPublicKey(keyPair.publicKey);
      
      if (process.env.RPC_ENDPOINT == null || process.env.RPC_ENDPOINT == "http://localhost:8899") {
        await api.requestAirdrop(pub, lamports(1000000000n)).send();  
        await new Promise((resolve) => setTimeout(resolve, 1000));
      }
      const result = await Deploy(
        {
          imageUrl: "https://bonsol-public-images.s3.us-east-1.amazonaws.com/simple-20b9db715f989e3f57842787badafae101ce0b16202491bac1a3aebf573da0ba",
          imageId: SIMPLE_IMAGE_ID,
          imageSize: 266608n,
          programName: "simple6",
          deployer: pub,
          inputs: [
            ProgramInputType.Public,
            ProgramInputType.Private
          ]
        })

      const ctx = await api.getLatestBlockhash().send()
      const txn = pipe(
        createTransactionMessage({ version: 0 }),
        tx => appendTransactionMessageInstruction(result, tx),
        tx => setTransactionMessageLifetimeUsingBlockhash(ctx.value, tx),
        tx => setTransactionMessageFeePayer(pub, tx),
        tx => compileTransaction(tx),
      );
      const signed = await signTransaction([keyPair], txn)
      try {
        await api.sendTransaction(getBase64EncodedWireTransaction(signed), { skipPreflight: false, encoding: 'base64' }).send()
        await api.confirmTransaction(getSignatureFromTransaction(signed), { commitment: "confirmed" }).send()
      } catch (e) {
        if (isSolanaError(e, SOLANA_ERROR__JSON_RPC__SERVER_ERROR_SEND_TRANSACTION_PREFLIGHT_FAILURE)) {
          console.log(e.context)
        }
      }
      
      await pipe(
        api.getTransaction(getSignatureFromTransaction(signed), {
          commitment: "confirmed",
          maxSupportedTransactionVersion: 0
        }).send(),
        async tx => {
          console.log("logs", (await tx)?.meta?.logMessages)
          return tx;
        })

    } else {
      
      let ea = parseBase64RpcAccount(depl, deployAccount.value)
      if (!ea.exists) {
        console.log("Image not deployed")
        return
      }
      let buf = new flatbuffers.ByteBuffer(ea.data as Uint8Array)
      let dp = DeployV1.getRootAsDeployV1(buf)
      console.log("deployed", dp.imageId())
      console.log("deployed", dp.programName())
      console.log("deployed", dp.url())
      console.log("deployed", dp.size())
      console.log("deployed", dp.ownerArray())
      console.log("deployed", dp.inputs)
    }
    //sleep
    await new Promise((resolve) => setTimeout(resolve, 1000))
  })

  it('should create valid execution requests', async () => {
    const keyPair = await kp()
    const pub = await getAddressFromPublicKey(keyPair.publicKey);
    if (process.env.RPC_ENDPOINT == null || process.env.RPC_ENDPOINT == "http://localhost:8899") {
      await api.requestAirdrop(pub, lamports(1000000000n)).send();
      await new Promise((resolve) => setTimeout(resolve, 1000));
    }
    //random uuid 
    const eid = randomBytes(5).toString();
    const input = JSON.stringify({ "attestation": "test" })
    const ht = await api.getBlockHeight().send()
    const result = await Execute(
      {
        executionId: eid,
        imageId: SIMPLE_IMAGE_ID,
        inputs: [
          {
            inputType: InputType.PublicData,
            input: Buffer.from(input, 'utf-8')
          },
          {
            inputType: InputType.Private,
            input: Buffer.from("https://echoserver.dev/server?response=N4IgFgpghgJhBOBnEAuA2mkBjA9gOwBcJCBaAgTwAcIQAaEIgDwIHpKAbKASzxAF0+9AEY4Y5VKArVUDCMzogYUAlBlFEBEAF96G5QFdkKAEwAGU1qA", 'utf-8')
          }
        ],
        tip: 1000000,
        expiry: ht + 800n, // 400s from now
        requester: pub
      }
    )

    const ctx = await api.getLatestBlockhash().send()
    await expect(async () => {
      const txn = pipe(
        createTransactionMessage({ version: 0 }),
        tx => setTransactionMessageFeePayer(pub, tx),
        tx => setTransactionMessageLifetimeUsingBlockhash(ctx.value, tx),
        tx => appendTransactionMessageInstruction(result, tx),
        tx => compileTransaction(tx),
      );

      const signed = await signTransaction([keyPair], txn)
      try {
        await api.sendTransaction(getBase64EncodedWireTransaction(signed), { skipPreflight: false, encoding: 'base64' }).send()
      } catch (e) {
       
          console.log(e)
      }
    
      await pipe(
        api.getTransaction(getSignatureFromTransaction(signed), {
          commitment: "confirmed",
          maxSupportedTransactionVersion: 0
        }).send(),
        async tx => {
          console.log("logs",((await tx)?.meta?.logMessages))
          return tx;
        }
      )
    }).not.toThrow()
    await new Promise((resolve) => setTimeout(resolve, 1000))
  })
})
