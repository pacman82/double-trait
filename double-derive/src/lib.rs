mod double;
mod dummies;

use syn::{Error, Ident, ItemTrait, parse_macro_input};

/// Generates a trait which replicates the original trait method for method. It does implement the
/// original trait for each of its implementations, by means of forwarding the method calls. The
/// utility comes from the fact that the generated trait has default implementations for each method
/// using `unimplemented!()`, which makes it useful for testing purposes.
///
/// If a test requires an implementation of an original trait `Org` yet would only invoke one of its
/// methods, implementing the mirrored method on an implementation of the generated trait `OrgDummy`
/// is sufficient. The other methods would not be inovked in the test, so their default
/// implementation using `unimplemented!()` would not be reached.
///
/// The argument passed to the attribute is used as the name of the generated trait.
///
/// * Existing default implementations are respected and not overridden.
/// * Visibility of the generated trait is the same as the original trait.
/// * `async` methods are supported
/// * Methods returning `impl` Traits are not supported, with the exception of `impl Future`.
/// * Generated double trait is implemented for `Dummy`.
///
/// # Example
///
/// Basic usage allows creating test stubs for traits, without worrying about implementing methods
/// not called in test code
///
/// ```no_run
/// use double_trait::double;
///
/// #[double(MyTraitDouble)]
/// trait MyTrait {
///    fn answer(&self) -> i32;
///
///    fn some_other_method(&self);
/// }
///
/// struct MyStub;
///
/// impl MyTraitDouble for MyStub {
///     fn answer(&self) -> i32 {
///         42
///     }
/// }
///
/// assert_eq!(42, MyTrait::answer(&MyStub));
/// ```
///
/// Then interacting with the `async_trait` crate, make sure to put the `#[async_trait]` attribute
/// on top.
///
/// ```no_run
/// use double_trait::double;
/// use async_trait::async_trait;
///
/// #[async_trait]
/// #[double(MyTraitDouble)]
/// trait MyTrait {
///     async fn answer(&self) -> i32;
/// }
/// ```
#[proc_macro_attribute]
pub fn double(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let double_name = parse_macro_input!(attr as Ident);
    let item = parse_macro_input!(item as ItemTrait);

    let output = double::expand(double_name, item).unwrap_or_else(Error::into_compile_error);

    proc_macro::TokenStream::from(output)
}

/// Generates a "dummy" implementation for each method in a trait using `unimplemented!()`. The main
/// use case is to greate specialized test doubles for implementing the trait without worrying the
/// need to explicitly implement methods, which are not invoked by the test.
///
/// * Existing default implementations are respected and not overridden.
/// * `async` methods are supported
/// * Methods returning `impl` Traits are not supported, with the exception of `impl Future` and
///   `impl Iterator`.
/// * Dummy implements the trait
#[proc_macro_attribute]
pub fn dummies(
    _attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    // let double_name = parse_macro_input!(attr as Ident);
    let item = parse_macro_input!(item as ItemTrait);

    let output = dummies::expand(item).unwrap_or_else(Error::into_compile_error);

    proc_macro::TokenStream::from(output)
}
