# predicates-rs

> An implementation of **boolean-valued predicate functions** in Rust.

[![Build Status](https://dev.azure.com/assert-rs/assert-rs/_apis/build/status/predicates-rs?branchName=master)](https://dev.azure.com/assert-rs/assert-rs/_build/latest?definitionId=1&branchName=master)
[![Documentation](https://img.shields.io/badge/docs-master-blue.svg)](https://docs.rs/predicates)
![License](https://img.shields.io/crates/l/predicates.svg)
[![Crates.io](https://img.shields.io/crates/v/predicates.svg?maxAge=2592000)](https://crates.io/crates/predicates)

[Changelog](https://github.com/assert-rs/predicates-rs/blob/master/CHANGELOG.md)


## Usage

First, add this to your `Cargo.toml`:

```toml
[dependencies]
predicates = "1.0.8"
```

Next, add this to your crate:

```rust
extern crate predicates;

use predicates::prelude::*;
```

For more information on using predicates, look at the
[documentation](https://docs.rs/predicates)


## License

`predicates-rs` is distributed under the terms of both the MIT license and the
Apache License (Version 2.0).

See LICENSE-APACHE, and LICENSE-MIT for details.


## Credits

Big thanks to [futures-rs](https://github.com/alexcrichton/futures-rs), whose
slick API design informed a lot of decisions made on the API design of this
library.
