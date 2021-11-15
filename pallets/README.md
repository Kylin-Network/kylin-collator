# Kylin OCW Modules

Kylin OCW repo contains the pallets for PoC.

**CAUTION! PLEASE DONOT USE THIS REPO DIRECTLY**

### Test account
Alice
```bash
$ subkey inspect "bottom drive obey lake curtain smoke basket hold race lonely fit walk" --scheme ed25519
Secret phrase `bottom drive obey lake curtain smoke basket hold race lonely fit walk` is account:
  Secret seed:      0xfac7959dbfe72f052e5a0c3c8d6530f202b02fd8f9f5ca3580ec8deb7797479e
  Public key (hex): 0x345071da55e5dccefaaa440339415ef9f2663338a38f7da0df21be5ab4e055ef
  Account ID:       0x345071da55e5dccefaaa440339415ef9f2663338a38f7da0df21be5ab4e055ef
  SS58 Address:     5DFJF7tY4bpbpcKPJcBTQaKuCDEPCpiz8TRjpmLeTtweqmXL

$ subkey inspect "bottom drive obey lake curtain smoke basket hold race lonely fit walk" --scheme  sr25519
Secret phrase `bottom drive obey lake curtain smoke basket hold race lonely fit walk` is account:
  Secret seed:      0xfac7959dbfe72f052e5a0c3c8d6530f202b02fd8f9f5ca3580ec8deb7797479e
  Public key (hex): 0x46ebddef8cd9bb167dc30878d7113b7e168e6f0646beffd77d69d39bad76b47a
  Account ID:       0x46ebddef8cd9bb167dc30878d7113b7e168e6f0646beffd77d69d39bad76b47a
  SS58 Address:     5DfhGyQdFobKM8NsWvEeAKk5EQQgYe9AydgJ7rMB6E1EqRzV

```

### OCW transaction's Q&A
```bash
WARN (offchain call) Error submitting a transaction to the pool: Pool(UnknownTransaction(UnknownTransaction::NoUnsignedValidator))
```
UnsigneTransactiond, must the validator can submit, you need use validator's key in OCW.

```bash
WARN (offchain call) Error submitting a transaction to the pool: Pool(InvalidTransaction(InvalidTransaction::Payment)) 
```
fee is not enough, please charge money.
