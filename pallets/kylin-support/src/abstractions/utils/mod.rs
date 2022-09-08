pub mod decrement;
pub mod increment;
pub mod start_at;

/// Helper macro to create a type that can be used in [`IncrementToMax`][increment::IncrementToMax].
///
/// # Usage
///
/// ```rust,ignore
/// #[pallet::config]
/// pub trait Config: frame_system::Config {
///     type SomeType: Copy + Zero + SafeAdd + One + TypeInfo + Member + FullCodec;
///
///     #[pallet::constant]
///     type SomeTypeMaxValue: Get<Self::SomeType>;
/// }
///
/// #[pallet::error]
/// #[derive(PartialEqNoBound)]
/// pub enum Error<T> {
///     SomeTypeTooLarge
/// }
///
/// #[pallet::storage]
/// #[allow(clippy::disallowed_type)] // counter
/// pub type Counter_ZeroInit_ToMax<T: Config> = StorageValue<
///     _,
///     T::SomeType,
///     ValueQuery,
///     Counter<
///         ZeroInit,
///         IncrementToMax<
///             T::SomeTypeMaxValue,
///             SomeTypeTooLarge,
///             Error<T>,
///         >,
///         SafeDecrement,
///     >,
/// >;
///
/// error_to_pallet_error!(
///      SomeTypeTooLarge,
/// );
/// ```
///
/// Note that this assumes that the pallet's `Error` and `Config` types are in scope and not
/// renamed.
#[macro_export]
macro_rules! error_to_pallet_error {
    ($($name:ident,)+) => {
        $(
            #[derive(::core::fmt::Debug, ::core::default::Default, ::core::cmp::PartialEq)]
            pub struct $name;

            impl<T: Config> From<$name> for Error<T> {
                fn from(_: $name) -> Error<T> {
                    Error::<T>::$name
                }
            }
        )+
    };
}

#[allow(clippy::disallowed_types)]
pub type ValueQuery = frame_support::pallet_prelude::ValueQuery;
