# Upgrade a Zk Program

Zk programs are immutable and can only be changed by redeploying the program. This means that if you want to upgrade a zk program you will need to redeploy the program.
The benefits of immutability out weigh the costs of redeploying the program and managing the upgrade process.

## Updating a Solana Program to use the new Zk Program ID
Your solana program will most likley have a constant that is used to identify the zk program. You will need to update this constant to the new zk program id.
This could cause an issue if your users have inflight proofs that are using the old zk program id. Our recommendation is to is to add another constant and check the slot that the execution request was created in. If the slot is older than the slot of your upgrade then you can use the new zk program id, otherwise you can allow the old zk program id to be used.

This will become very important especially if the two zkprogram versions emit different output data. In the future Bonsol may offer abstractions to help with this.