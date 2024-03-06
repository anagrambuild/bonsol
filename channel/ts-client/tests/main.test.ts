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
import { BonsolProgram } from "../src/index";
import { randomBytes } from 'crypto';
import { pipe } from '@solana/functional';
import { createDefaultRpcSubscriptionsTransport } from '@solana/web3.js';
import { ExecutionInputType } from 'bonsol-schemas';


describe('BonsolProgram', () => {

  it('should create valid execution requests', async () => {
    const keyPair = await generateKeyPair();
    const tsp = createDefaultRpcTransport({ url: Cluster.Development.apiUrl })
    const ssp = createDefaultRpcSubscriptionsTransport({
      url: Cluster.Development.apiUrl.replace('http', 'ws').replace("8899", "8900")
    })
    const api = createSolanaRpc({ transport: tsp });
    const sub = createSolanaRpcSubscriptions({ transport: ssp })
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
    const eid = randomBytes(16).toString('hex')
    const input = JSON.stringify({ "attestation": "test" })
    const result = await BonsolProgram.Execute(
      {
        executionId: eid,
        imageId: "ffddd818c78b130ba84aacd1cc69e075e40053c4ca14f1b0272bf196c204d2d0",
        inputType: ExecutionInputType.DATA,
        input: Buffer.from(input,'utf-8'),
        requester: pub
      }
    )
    const s = createDefaultTransactionSender({
      rpc: api,
      rpcSubscriptions: sub
    }) 
    const ctx = await api.getLatestBlockhash().send()
    expect(async () => {
      const x = pipe(
        createTransaction({ version: 0 }),
        tx => setTransactionFeePayer(pub, tx),
        tx => setTransactionLifetimeUsingBlockhash(ctx.value, tx),
        tx => appendTransactionInstruction(result, tx),
        async tx => signTransaction([keyPair], tx),
        async tx => {
          await s(await tx, { commitment: "confirmed" });
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
