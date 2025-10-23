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
///   `impl Iterator`. One way to deal with this, is to give them an explicit default implementation
///   in the test case. E.g.,
///
///   ```
///   # trait Answer {}
///   # struct DummyAnswer;
///   # impl Answer for DummyAnswer {}
///
///   #[cfg_attr(test, double_trait::dummies)]
///   trait MyTrait {
///     #[cfg(not(test))]
///     fn answer(&self) -> impl Answer;
///
///     // `dummies` can not interfere a type for `impl Answer`, so we provide a default impl here.
///     #[cfg(test)]
///     fn answer(&self) -> impl Answer {
///         DummyAnswer
///     }
///
///     // ... other methods ...
///   }
///   ```
/// * Dummy implements the trait
#[proc_macro_attribute]
pub fn dummies(
    _attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let item = parse_macro_input!(item as ItemTrait);

    let output = dummies::expand(item).unwrap_or_else(Error::into_compile_error);

    proc_macro::TokenStream::from(output)
}
