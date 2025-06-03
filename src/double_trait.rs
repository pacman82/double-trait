use quote::quote;
use syn::{Ident, ItemTrait, ReturnType, TraitItem, TraitItemFn, Type, parse2, spanned::Spanned};

/// Generate a double trait which mirrors the original trait's methods and provides default
/// implementations using `unimplemented!()`.
pub fn double_trait(double_trait_name: Ident, org_trait: ItemTrait) -> syn::Result<ItemTrait> {
    let items = org_trait
        .items
        .into_iter()
        .filter_map(|item| transform_trait_item(item).transpose())
        .collect::<syn::Result<_>>()?;
    Ok(ItemTrait {
        ident: double_trait_name.clone(),
        items,
        ..org_trait
    })
}

fn transform_trait_item(trait_item: TraitItem) -> syn::Result<Option<TraitItem>> {
    // We are only interessted in transforming functions
    let transformed_trait_item = if let TraitItem::Fn(fn_item) = trait_item {
        transform_function(fn_item)?.map(TraitItem::Fn)
    } else {
        // If it is not a function, we forward the original Item
        Some(trait_item)
    };
    Ok(transformed_trait_item)
}

// Filter method which already have a default implementation
fn transform_function(mut fn_item: TraitItemFn) -> syn::Result<Option<TraitItemFn>> {
    if fn_item.default.is_some() {
        return Ok(None);
    }

    let is_impl_future = is_maybe_impl_future(&fn_item.sig.output);

    let default_impl =
        if is_impl_future {
            // If the method returns an impl Future, we provide a default implementation using an async
            // block, so that the compiler won't complain about not being able to infer the type of
            // `impl Future`.
            // This `quote!` is falliable, because we do not know for sure that the impl is a future
            parse2(quote! {{ async { unimplemented!() }} })
            .map_err(|_| syn::Error::new(
                fn_item.sig.output.span(),
                "impl Trait is currently not supported by double-derive. Apart from the special \
                case of impl Future."))?
        } else {
            // Otherwise, we provide a default implementation using unimplemented!
            // We can unwrap here, this body should always compile
            parse2(quote! {{ unimplemented!() }}).unwrap()
        };

    fn_item.default = Some(default_impl);

    Ok(Some(fn_item))
}

fn is_maybe_impl_future(output: &ReturnType) -> bool {
    if let ReturnType::Type(_rarrow, ty) = output {
        if let Type::ImplTrait(ref _impl_trait) = **ty {
            // Technically, not every impl is a "impl Future", but for now we assume that.
            true
        } else {
            false
        }
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::double_trait;
    use quote::quote;
    use syn::{Ident, ItemTrait, parse2};

    #[test]
    fn default_impl_for_method_with_impl_future_return() {
        // Given an original trait with a method returning an impl Future
        let (double_trait_name, org_trait) = given(
            quote! { DoubleTrait },
            quote! {
                trait OriginalTrait {
                    fn method(&self) -> impl Future<Output = ()>;
                }
            },
        );

        // When generating the double trait
        let double_trait = double_trait(double_trait_name, org_trait).unwrap();

        // Then the double trait should have a default implementation for the method which uses
        // an async block
        let actual = quote! { #double_trait };
        let expected = quote! {
            trait DoubleTrait {
                fn method(&self) -> impl Future<Output = ()> {
                    async { unimplemented!() }
                }
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
