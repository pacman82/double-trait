use quote::quote;
use syn::ItemTrait;

/// The main implementation of [`crate::dummies`]. This function is not annotated with
/// `#[proc_macro_attribute]` so it can exist in unit tests. It uses only APIs build on top of
/// [`proc_macro2`] in order to be unit testable.
pub fn expand(org_trait: ItemTrait) -> syn::Result<proc_macro2::TokenStream> {
    let token_stream = quote! {
        #org_trait
    };
    Ok(token_stream)
}

#[cfg(test)]
mod tests {}
