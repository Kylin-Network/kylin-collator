# Airdrop

The Kylin-airdrop pallet enables the granting of tokens to a set of users.

## The idea

An Airdrop provides the means to distribute a set amount of tokens over some 
amount of time to a set of accounts. Once a claim is made, users will have a portion 
of their Airdropped funds deposited into their local accounts (e.g. PICHU 
accounts). Users will be able to continue to preform validated claims until they 
have claimed all of their funds, or the Airdrop has ended.
It supports multiple concurrent airdrops, instead of a single one.

## Signing & Verifying Claims

The Airdrop pallet supports remote accounts from Ethereum
To verify account ownershipAirdrop will 
preform validation on signatures natively produced by the Etherum chain. 
For Ethereum `msg` is expected to be the account ID.

Transactions with the `claim` extrinsic are expected to be unsigned. While users 
will sign part of the transaction payload, the transaction itself will be 
unsigned. To prevent transaction spamming, unsigned transactions are validated 
to ensure they have a signed payload and otherwise relevant information for an 
active Aairdrop.

## Gas & Fees

When a creator adds recipients to an Airdrop, they can indicate that specific 
users will have their claims funded. If this is true, users will not pay fees 
associated with the `claim` transaction.

## Workflow

Airdrops can be created by any user who is capable of providing the required 
`Stake` needed to create an Airdrop. Once created, the account address 
associated with the creation transaction will be able to utilize the life cycle 
transactions of this pallet. An Airdrop has three life cycle states: created, 
enabled, and disabled.

### Created

During the Created state, the Airdrop can be manipulated with the life cycle 
transactions (`enable_airdrop`, `disable_airdrip`, `add_recipient`, 
`remove_recipient`). During this state, claims can not be made by recipients. 

If the Airdrop was created with a `start_at`, it will automatically transition 
to enabled once that point in time has passed. If no `start_at` was provided, 
the airdrop can be manually enabled by the creator.

### Enabled

Once enabled, recipients can begin claiming funds from the Airdrop. Funds will 
become available to users according to the `vesting_schedule` provided at 
creation and the vesting window size provided when each recipient is added.

While the Airdrop is enabled, the creator can still use the life cycle 
transactions in a limited fashion. The most notable limitation is that, once a 
recipient has started claiming funds, they cannot be removed from the Airdrop.

Once there are no more remaining unclaimed funds (via claiming or removing of 
inactive recipients) the Airdrop will automatically transition to the disabled 
state. The disabled state can also be manually triggered with the 
`disable_airdrop` transaction.

### Disabled

Once an Airdrop has been disabled, it will be removed from pallet storage along 
with other related information.
