import { Cluster, LuzidSdk } from '@luzid/sdk'
import {
  appendTransactionInstruction,
  createTransaction,
  signTransaction,
  lamports,
  setTransactionFeePayer,
  getBase64EncodedWireTransaction,
  setTransactionLifetimeUsingBlockhash,
  createSolanaRpcSubscriptions,
  createDefaultTransactionSender,
  createSolanaRpc, generateKeyPair, createDefaultRpcTransport, address, getAddressFromPublicKey, createDefaultAirdropRequester, signature, getSignatureFromTransaction
} from '@solana/web3.js';

import { pipe } from '@solana/functional';
import { createDefaultRpcSubscriptionsTransport } from '@solana/web3.js';
import { InputType, ProgramInputType } from 'bonsol-schemas';
import { Execute, Deploy, deploymentAddress } from '../src/';

describe('BonsolProgram', () => {
  const SIMPLE_IMAGE_ID = "1133b1185fa60cb4deb91ab7d9368f1539753c0541f544806656c5b00c294df7";

  const tsp = createDefaultRpcTransport({ url: Cluster.Development.apiUrl })
  const ssp = createDefaultRpcSubscriptionsTransport({
    url: Cluster.Development.apiUrl.replace('http', 'ws').replace("8899", "8900")
  })
  const api = createSolanaRpc({ transport: tsp });
  const sub = createSolanaRpcSubscriptions({ transport: ssp })
  const sender = createDefaultTransactionSender({
    rpc: api,
    rpcSubscriptions: sub
  })
  

  beforeEach(async () => {
    const depl = await deploymentAddress(SIMPLE_IMAGE_ID);
    console.log(depl)
    const deployAccount = await api.getAccountInfo(depl, { commitment: "confirmed", encoding: "base64" }).send();
    if (!deployAccount.value) {
      //deploy
      const keyPair = await generateKeyPair();
      const pub = await getAddressFromPublicKey(keyPair.publicKey);
      const airdrop = createDefaultAirdropRequester({
        rpc: api,
        rpcSubscriptions: sub
      })
      await airdrop({
        commitment: "confirmed",
        lamports: lamports(1000000000n),
        recipientAddress: pub
      })
      const result = await Deploy(
        {
          imageUrl: "https://elfserver.solana",
          imageId: SIMPLE_IMAGE_ID,
          programName: "simple",
          deployer: pub,
          inputs: [
            ProgramInputType.Public,
            ProgramInputType.Private
          ]
        })
     
      const ctx = await api.getLatestBlockhash().send()
      const x = pipe(
        createTransaction({ version: 0 }),
        tx => setTransactionFeePayer(pub, tx),
        tx => setTransactionLifetimeUsingBlockhash(ctx.value, tx),
        tx => appendTransactionInstruction(result, tx),
        async tx => signTransaction([keyPair], tx),
        async tx => {
          await sender(await tx, { commitment: "confirmed", skipPreflight: true });
          return tx;
        },
        async tx => api.getTransaction(getSignatureFromTransaction(await tx), {
          commitment: "confirmed",
          maxSupportedTransactionVersion: 0
        }).send(),
        async tx => {
          console.log((await tx)?.meta?.logMessages)
          return tx;
        }
      );
      await x;
    }
  })

  it('should create valid execution requests', async () => {
    const keyPair = await generateKeyPair();


    const pub = await getAddressFromPublicKey(keyPair.publicKey);
    const airdrop = createDefaultAirdropRequester({
      rpc: api,
      rpcSubscriptions: sub
    })
    await airdrop({
      commitment: "confirmed",
      lamports: lamports(1000000000n),
      recipientAddress: pub
    })
    const eid = "test-execution-id"
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
        expiry: ht + 800n, // 400s from now
        requester: pub
      }
    )
   
    const ctx = await api.getLatestBlockhash().send()
    expect(async () => {
      const x = pipe(
        createTransaction({ version: 0 }),
        tx => setTransactionFeePayer(pub, tx),
        tx => setTransactionLifetimeUsingBlockhash(ctx.value, tx),
        tx => appendTransactionInstruction(result, tx),
        async tx => signTransaction([keyPair], tx),
        async tx => {
          await sender(await tx, { commitment: "confirmed"});
          return tx;
        },
        async tx => api.getTransaction(getSignatureFromTransaction(await tx), {
          commitment: "confirmed",
          maxSupportedTransactionVersion: 0
        }).send(),
        async tx => {
          console.log((await tx)?.meta?.logMessages)
          return tx;
        }
      );
      await x;
    }).not.toThrow()

  })
})
