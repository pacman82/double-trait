// Reexport the double macro from our derive crate
pub use double_derive::dummies;

/// A general purpose test Dummy. Implements any interface annotadet with the [`dummies`] macro as
/// well as a number of traits from the `std` namespace.
///
/// [`Dummy`] will implement any annotated trait trivially. For a trait `MyTrait` annotated with
/// [`dummies`] the implementation is:
///
/// ```no_run
/// # use double_trait::Dummy;
/// # trait MyTrait {}
/// impl MyTrait for Dummy {}
/// ```
///
/// This works, because after being annotated with [`dummies`] every trait method has a default
/// implementation. Even if it is just panicing.
///
/// ```no_run
/// use double_trait::{dummies, Dummy};
///
/// #[dummies]
/// trait OrgTrait {
///     fn answer(&self) -> i32;
/// }
///
/// OrgTrait::answer(&Dummy); // Compiles, but raises panic with `unimplemented!()`
/// ```
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Dummy;
