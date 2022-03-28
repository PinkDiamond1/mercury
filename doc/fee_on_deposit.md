# Mercury wallet: pay on deposit

Mercury wallet and service currently has an open deposit model: deposits are free and permissionless (i.e. no authentication required to generate a shared key deposit addresses) and the mercury server fee (as a fixed percentage of the coin value) is collected in the withdrawal transaction as a UTXO paid to a fixed, specified bitcoin address. This has the advantage of making the deposit process low friction and user friendly, but has some disadvantages:

The withdrawal transaction fee output is typically a small fraction of the coin value and for the smallest coin values is close to the dust limit (i.e. these outputs may not be spendable in a high tx fee environment). The on-chain mercury fee explicitly labels all withdrawn coins as mercury statechain withdrawals, which is a privacy concern for many users.

The alternative that mitigates these issues is to charge the fee up-front, via a LN invoice or separate bitcoin payment, before the shared key deposit address is generated. In this approach, a user would specify in the wallet that they wanted to deposit a specific amount into a statecoin, and instead of performing a shared key generation with the server, would request a LN invoice or payment address for the mercury fee from the server, which would be returned to the wallet and displayed to the user.

The user would then copy this invoice (by C&P or QR code) or address into a third party LN or bitcoin wallet and pay the fee. A LN node running on the mercury server back end would then verify that the payment had been made, and enable the wallet to continue with the deposit keygen and deposit process. This coin would be labeled as ‘fee paid’ by the wallet and server, and not be subject to an on-chain fee payment on withdrawal. 

## Current process:

The permissionless deposit protocol is currently protected from attack by a proof-of-work system. The first step in any deposit is the `deposit_init` POST request. This contains the user wallet generated public `proof_key`, and the server returns both a unique `user_id` and a `challenge` nonce for the proof-of-work computation. The challenge is then stored in the `usersession` table. 

The wallet then completes the solution for the PoW and returns this to the server with the `user_id` in the first shared key generation request (`/ecdsa/keygen/first`). The server then verifies the proof-of-work (against the stored challenge) and performs `keygen_first`. The PoW protects the expensive (via Lockbox) `keygen_first` computations from abuse/attack. 

## Proposed process:

The server is configured to have access (via an RPC connection) to an LN node. When `deposit_init` is called, instead of issuing a PoW challenge, the mercury server generates 1. A bitcoin deposit address (using a deterministic derivation - each new deposit will recieve a unique address). This can be generated within the server logic using rust-bitcoin tools. 
2. A LN invoice generated by the LN node (via an RPC call). 

Both of these are then returned to the wallet in the `UserID` object, and stored in the `usersession` table in two new columns with the `user_id`. 

The wallet then displays the invoice and address on the deposit page, and begins attempting `/ecdsa/keygen/first` with just the `user_id` every 2-3 seconds. 

The mercury server `keygen_first` is then repeatedly called. Each time this is called for a given `user_id`, two external RPC calls are made: 1. to the LN node and 2. The electrum server (with the invoice and address retreieved from the `usersession` table). If either the deposit has been made to the address (with one confirmation) or the LN payment received, then `keygen_first` is completed - otherwise a "Payment not received" error is returned. While the wallet is receiving this error, it should display "Awaiting fee payment" on the deposit page. 

Once `keygen_first` is complete, `keygen_second` comletes and the wallet displays the deposit address. 

### Deposit amounts:

The above assumes a single deposit fee. To support a fee as a percentage of the deposit amount, the deposit amount must be selected in advance. The deposit/fee amount can be included along with the proof_key in the `deposit_init` call, and added to the `usersession` DB table. The amount is then verified against this in the confirmation RPC calls to the electrum server and LN node. 

Nothing prevents a user from depositing a different amount (and cheating the % fee) - however they can be prevented from joining a swap group by having the `verify_tx_confirmed` function verify the amount of the confirmed deposit tx against the fee/amount in the `usersession` table. 

## Auto-deposit:

Auto-deposit is where the wallet itself manages coin deposit process automatically, enabling the creation of many coins in a single operation. 