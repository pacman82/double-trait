mod double_trait;
mod dummies;
mod dummy_impl;

use syn::{Error, ItemTrait, parse_macro_input};

/// Generates a "dummy" implementation for each method in a trait and implements the trait for `Dummy`.
///
/// This eases implementing test doubles in cases there the test does not require all the methods of
/// a trait. The compiler is happy as there is a default implementation and you can focus on
/// overwriting the behavior which is of interest to your test.
///
/// * Most default implementations will call `unimplemented!`.
/// * Existing default implementations are respected and not overridden.
/// * Methods returning `impl` Trait will not work unless they are specifically supproted by this
///   crate. One way to deal with this, is to give them an explicit default implementation in the
///   test case. E.g.,
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
///
/// * Associated types are implemented using `Dummy`.
/// * Async methods and methods returning `impl Future` are supported and inherit the default from
///   their sync counterparts.
/// * Methods returning `impl Iterator` are supported and will return an empty iterator.
/// * Methods returning `impl Stream` are supported if the `stream` feature is activated and will
///   return an empty Stream.
/// * Methods returning `Result`, will use the default behavior of the `Ok` type and wrap it in
///   `Ok`.
/// * Methods returning `Option` will return `None`.
/// * Methods returning `Vec` will return `Vec::new`.
///
#[proc_macro_attribute]
pub fn dummies(
    _attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let item = parse_macro_input!(item as ItemTrait);

    let output = dummies::expand(item).unwrap_or_else(Error::into_compile_error);

    proc_macro::TokenStream::from(output)
}
