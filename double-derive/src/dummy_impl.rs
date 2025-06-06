use proc_macro2::TokenStream;
use quote::quote;
use syn::{Ident, ItemTrait, };

/// Implemntation of double trait for `Dummy` type.
pub fn dummy_impl(double_trait_name: Ident, _org_trait: ItemTrait) -> TokenStream {
    quote! {
        impl #double_trait_name for double_trait::Dummy{}
    }
}

