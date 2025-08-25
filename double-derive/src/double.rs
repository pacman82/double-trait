mod dummy_impl;
mod trait_impl;

use quote::quote;
use syn::{Ident, ItemTrait};

use self::trait_impl::trait_impl;
use crate::double_trait::double_trait;

/// The main implementation of [`crate::double`]. This function is not annotated with
/// `#[proc_macro_attribute]` so it can exist in unit tests. It uses only APIs build on top of
/// [`proc_macro2`] in order to be unit testable.
pub fn expand(
    double_trait_name: Ident,
    org_trait: ItemTrait,
) -> syn::Result<proc_macro2::TokenStream> {
    let double_trait = double_trait(double_trait_name.clone(), org_trait.clone())?;
    let trait_impl = trait_impl(double_trait_name.clone(), org_trait.clone());
    let dummy_impl = dummy_impl::dummy_impl(double_trait_name, org_trait.clone());

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

        #dummy_impl
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
                fn foobar (&self) {}
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

            pub trait MyTraitDummy {
                fn foobar() { println!("Hello Default!") }
            }

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
                async fn foobar (&self) {}
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
