mod double_trait;
mod dummies;
mod dummy_impl;

use syn::{Error, ItemTrait, parse_macro_input};

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
