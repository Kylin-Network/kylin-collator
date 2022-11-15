# Distribution

The Kylin-distribution pallet enables the granting of tokens to a set of users.

## The idea

An Distribution provides the means to distribute a set amount of tokens over some 
amount of time to a set of accounts. Once a claim is made, users will have a portion 
of their Distributionped funds deposited into their local accounts (e.g. PICHU 
accounts). Users will be able to continue to preform validated claims until they 
have claimed all of their funds, or the Distribution has ended.
It supports multiple concurrent distributions, instead of a single one.

## Signing & Verifying Claims

[TO DO]

## Gas & Fees

When a creator adds recipients to an Distribution, they can indicate that specific 
users will have their claims funded. If this is true, users will not pay fees 
associated with the `claim` transaction.

## Workflow

Distributions can be created by any user who is capable of providing the required 
`Stake` needed to create an Distribution. Once created, the account address 
associated with the creation transaction will be able to utilize the life cycle 
transactions of this pallet. An Distribution has three life cycle states: created, 
enabled, and disabled.

### Created

During the Created state, the Distribution can be manipulated with the life cycle 
transactions (`enable_distribution`, `disable_distribution`, `add_recipient`, 
`remove_recipient`). During this state, claims can not be made by recipients. 

If the Distribution was created with a `start_at`, it will automatically transition 
to enabled once that point in time has passed. If no `start_at` was provided, 
the distribution can be manually enabled by the creator.

### Enabled

Once enabled, recipients can begin claiming funds from the Distribution. Funds will 
become available to users according to the `vesting_schedule` provided at 
creation and the vesting window size provided when each recipient is added.

While the Distribution is enabled, the creator can still use the life cycle 
transactions in a limited fashion. The most notable limitation is that, once a 
recipient has started claiming funds, they cannot be removed from the Distribution.

Once there are no more remaining unclaimed funds (via claiming or removing of 
inactive recipients) the Distribution will automatically transition to the disabled 
state. The disabled state can also be manually triggered with the 
`disable_distribution` transaction.

### Disabled

Once an Distribution has been disabled, it will be removed from pallet storage along 
with other related information.

