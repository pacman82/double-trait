use quote::quote;
use syn::{Error, Ident, ItemTrait, parse_macro_input};

mod double_trait;
mod trait_impl;

use self::{double_trait::double_trait, trait_impl::trait_impl};

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

    let output = expand(double_name, item).unwrap_or_else(Error::into_compile_error);

    proc_macro::TokenStream::from(output)
}

/// The main implementation of [`crate::double`]. This function is not annotated with
/// `#[proc_macro_attribute]` so it can exist in unit tests. It uses only APIs build on top of
/// [`proc_macro2`] in order to be unit testable.
fn expand(double_trait_name: Ident, org_trait: ItemTrait) -> syn::Result<proc_macro2::TokenStream> {
    let double_trait = double_trait(double_trait_name.clone(), org_trait.clone())?;
    let trait_impl = trait_impl(double_trait_name.clone(), org_trait.clone());

    // We generate three items as part of our output.
    // 1. The orginal trait, which we put in the output unaltered.
    // 2. The double trait, we genarate, which mirrors the original traits methods and provides
    //    default implementations using `unimplemented!()`.
    // 3. An implementation of the original trait for all types which implement the double trait.
    //    This is done by forwarding the method calls to the double trait.
    let token_stream = quote! {
        #org_trait

        #double_trait

        #trait_impl

        impl #double_trait_name for double_trait::Dummy{}
    };
    Ok(token_stream)
}

#[cfg(test)]
mod tests {

    use super::{Ident, expand};
    use quote::quote;
    use syn::{ItemTrait, parse2};

    #[test]
    fn generate_double_trait() {
        let (attr, item) = given(quote! { MyTraitDummy }, quote! { trait MyTrait {} });

        let output = expand(attr, item).unwrap();

        let expected = quote! {
            trait MyTrait {}

            trait MyTraitDummy {}

            impl<T> MyTrait for T where T: MyTraitDummy {}

            impl MyTraitDummy for double_trait::Dummy {}
        };
        assert_eq!(expected.to_string(), output.to_string());
    }

    #[test]
    fn forward_visibility() {
        // Given a public trait
        let (attr, item) = given(quote! { MyTraitDummy }, quote! { pub trait MyTrait {} });

        // When generating the dummy
        let output = expand(attr, item).unwrap();

        // Then the generated trait should be public, too
        let expected = quote! {
            pub trait MyTrait {}

            pub trait MyTraitDummy {}

            impl<T> MyTrait for T where T: MyTraitDummy {}

            impl MyTraitDummy for double_trait::Dummy {}
        };
        assert_eq!(expected.to_string(), output.to_string());
    }

    #[test]
    fn forward_method() {
        // Given a trait with a method
        let (attr, item) = given(
            quote! { MyTraitDummy },
            quote! {
                trait MyTrait {
                    fn foobar(&self);
                }
            },
        );

        // When generating the dummy
        let output = expand(attr, item).unwrap();

        // Then the generated trait should contain that method, too
        let expected = quote! {
            trait MyTrait {
                fn foobar(&self);
            }

            trait MyTraitDummy {
                fn foobar (&self) { unimplemented!() }
            }

            impl<T> MyTrait for T where T: MyTraitDummy {
                fn foobar(&self) { <Self as MyTraitDummy>::foobar(self,) }
            }

            impl MyTraitDummy for double_trait::Dummy {}
        };
        assert_eq!(expected.to_string(), output.to_string());
    }

    #[test]
    fn respect_existing_default_impl() {
        // Given a method with a default implementation in the original trait
        let (attr, item) = given(
            quote! { MyTraitDummy },
            quote! {
                pub trait MyTrait {
                    fn foobar() { println!("Hello Default!") }
                }
            },
        );

        // When generating the dummy
        let output = expand(attr, item).unwrap();

        // Then the generated trait should not overide the existing default
        let expected = quote! {
            pub trait MyTrait {
                fn foobar() { println!("Hello Default!") }
            }

            pub trait MyTraitDummy {}

            impl<T> MyTrait for T where T: MyTraitDummy {
                fn foobar() { <Self as MyTraitDummy>::foobar() }
            }

            impl MyTraitDummy for double_trait::Dummy {}
        };
        assert_eq!(expected.to_string(), output.to_string());
    }

    #[test]
    fn forward_async_method() {
        // Given a trait with a method
        let (attr, item) = given(
            quote! { MyTraitDummy },
            quote! {
                trait MyTrait {
                    async fn foobar(&self);
                }
            },
        );

        // When generating the dummy
        let output = expand(attr, item).unwrap();

        // Then the generated trait should contain that method, too
        let expected = quote! {
            trait MyTrait {
                async fn foobar(&self);
            }

            trait MyTraitDummy {
                async fn foobar (&self) { unimplemented!() }
            }

            impl<T> MyTrait for T where T: MyTraitDummy {
                async fn foobar(&self) { <Self as MyTraitDummy>::foobar(self,).await }
            }

            impl MyTraitDummy for double_trait::Dummy {}
        };
        assert_eq!(expected.to_string(), output.to_string());
    }

    fn given(attr: proc_macro2::TokenStream, item: proc_macro2::TokenStream) -> (Ident, ItemTrait) {
        let attr: Ident = parse2(attr).unwrap();
        let item: ItemTrait = parse2(item).unwrap();
        (attr, item)
    }
}
