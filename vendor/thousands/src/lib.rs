#![doc(html_root_url = "https://docs.rs/thousands/0.2.0")]
//! Provides a trait, [`Separable`], for formatting numbers with
//! separators between the digits. Typically this will be used to add
//! commas or spaces every three digits from the right, but can be
//! configured via a [`SeparatorPolicy`].
//!
//! # Examples
//!
//! The simplest way to use the library is with trait [`Separable`]’s method
//! [`separate_with_commas`] method, which does what it sounds like:
//!
//! ```
//! use thousands::Separable;
//!
//! assert_eq!(   12345  .separate_with_commas(),  "12,345" );
//! assert_eq!( (-12345) .separate_with_commas(), "-12,345" );
//! assert_eq!(    9876.5.separate_with_commas(),   "9,876.5" );
//! ```
//!
//! There are also methods [`separate_with_spaces`], [`separate_with_dots`], and
//! [`separate_with_underscores`], in case you, your culture, or your file
//! format prefer those separators.
//!
//! However, it's also possible to pass a policy for different behavior:
//!
//! ```
//! use thousands::{Separable, SeparatorPolicy, digits};
//!
//! let policy = SeparatorPolicy {
//!     separator: ",",
//!     groups:    &[3, 2],
//!     digits:    digits::ASCII_DECIMAL,
//! };
//!
//! assert_eq!( 1234567890.separate_by_policy(policy), "1,23,45,67,890" );
//! ```
//!
//! # Usage
//!
//! It’s [on crates.io](https://crates.io/crates/thousands), so you can add
//!
//! ```toml
//! [dependencies]
//! thousands = "0.2.0"
//! ```
//!
//! to your `Cargo.toml`.
//!
//! This crate supports Rust version 1.22 and newer.
//!
//! [`Separable`]: trait.Separable.html
//! [`SeparatorPolicy`]: struct.SeparatorPolicy.html
//! [`separate_with_commas`]: trait.Separable.html#method.separate_with_commas
//! [`separate_with_spaces`]: trait.Separable.html#method.separate_with_spaces
//! [`separate_with_dots`]: trait.Separable.html#method.separate_with_dots
//! [`separate_with_underscores`]: trait.Separable.html#method.separate_with_underscores

/// Collections of digits.
///
/// These are used for constructing [SeparatorPolicy](struct.SeparatorPolicy.html)s.
pub mod digits;

/// Predefined policies.
pub mod policies;
pub use policies::SeparatorPolicy;

mod traits;
pub use traits::Separable;

// Contains blanket impl<T: Display> Separable for T.
mod display;

mod helpers;
