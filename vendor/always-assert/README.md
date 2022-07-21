# always-assert

Recoverable assertions, inspired by [the use of `assert()` in SQLite](https://www.sqlite.org/assert.html).

```rust
use always_assert::never;

fn apply_transaction(&mut self, tx: Transaction) -> Result<(), TransactionAborted> {
    let delta = self.compute_delta(&tx);

    if never!(!self.check_internal_invariant(&delta)) {
        // Ok, something in this transaction messed up our internal state.
        // This really shouldn't be happening, and this signifies a bug.
        // Luckily, we can recover by just rejecting the transaction.
        return abort_transaction(tx);
    }
    self.commit(delta);
    Ok(())
}
```
