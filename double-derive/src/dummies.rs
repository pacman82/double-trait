use quote::quote;
use syn::ItemTrait;

use crate::{double_trait::double_trait, dummy_impl::dummy_impl};

/// The main implementation of [`crate::dummies`]. This function is not annotated with
/// `#[proc_macro_attribute]` so it can exist in unit tests. It uses only APIs build on top of
/// [`proc_macro2`] in order to be unit testable.
pub fn expand(org_trait: ItemTrait) -> syn::Result<proc_macro2::TokenStream> {
    let trait_with_dummies = double_trait(org_trait.ident.clone(), org_trait.clone())?;
    let dummy_impl = dummy_impl(org_trait.ident.clone(), org_trait);

    let token_stream = quote! {
        #trait_with_dummies

        #dummy_impl
    };
    Ok(token_stream)
}

#[cfg(test)]
mod tests {

    use quote::quote;
    use syn::{ItemTrait, parse2};

    use super::expand;

    #[test]
    fn dummies_for_empty_trait() {
        // Given an empty trait
        let empty_trait = given(quote! {
            trait MyTrait {}
        });

        // When expanded with `dummies`
        let output = expand(empty_trait).unwrap();

        // Then it will be unchanged
        let expected = quote! {
            trait MyTrait{}

            impl MyTrait for double_trait::Dummy {}
        };
        assert_eq!(expected.to_string(), output.to_string())
    }

    #[test]
    fn forward_visibility() {
        // Given a public trait
        let org_trait = given(quote! { pub trait MyTrait {} });

        // When generating the dummy
        let output = expand(org_trait).unwrap();

        // Then the generated trait should be public, too
        let expected = quote! {
            pub trait MyTrait {}

            impl MyTrait for double_trait::Dummy {}
        };
        assert_eq!(expected.to_string(), output.to_string());
    }

    fn given(item: proc_macro2::TokenStream) -> ItemTrait {
        let item: ItemTrait = parse2(item).unwrap();
        item
    }
}
