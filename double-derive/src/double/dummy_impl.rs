use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    Ident, ImplItem, ImplItemType, ItemTrait, Token, TraitItem, Visibility, spanned::Spanned,
};

/// Implemntation of double trait for `Dummy` type.
pub fn dummy_impl(double_trait_name: Ident, org_trait: ItemTrait) -> TokenStream {
    let items = org_trait.items.into_iter().filter_map(transform_trait_item);
    quote! {
        impl #double_trait_name for double_trait::Dummy{
            #(#items)*
        }
    }
}

// We provide a dummy implementation for associated types. We do this in a dummy impl, because at
// the time of writing this, default types in traits are not supported by stable Rust.
fn transform_trait_item(item: TraitItem) -> Option<ImplItem> {
    if let TraitItem::Type(ty_item) = item {
        let span = ty_item.span();
        let impl_item = ImplItemType {
            attrs: ty_item.attrs,
            vis: Visibility::Inherited,
            defaultness: None,
            type_token: ty_item.type_token,
            ident: ty_item.ident,
            generics: ty_item.generics,
            eq_token: Token![=](span),
            ty: syn::parse_quote! { double_trait::Dummy },
            semi_token: Token![;](span),
        };
        Some(ImplItem::Type(impl_item))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use quote::quote;
    use syn::{Ident, ItemTrait, parse2};

    use super::dummy_impl;

    #[test]
    fn provide_default_implementation_for_associated_types() {
        // Given an original trait with an associated type
        let (double_trait_name, org_trait) = given(
            quote! { DoubleTrait },
            quote! {
                trait OriginalTrait {
                    type AssociatedType;
                }
            },
        );

        // When generating the dummy implementation
        let dummy_impl = dummy_impl(double_trait_name, org_trait);

        // Then the dummy implementation should provide a default type for the associated type
        let actual = quote! { #dummy_impl };
        let expected = quote! {
            impl DoubleTrait for double_trait::Dummy {
                type AssociatedType = double_trait::Dummy;
            }
        };
        assert_eq!(actual.to_string(), expected.to_string());
    }

    fn given(attr: proc_macro2::TokenStream, item: proc_macro2::TokenStream) -> (Ident, ItemTrait) {
        let attr: Ident = parse2(attr).unwrap();
        let item: ItemTrait = parse2(item).unwrap();
        (attr, item)
    }
}
