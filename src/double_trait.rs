use quote::quote;
use syn::{Ident, ItemTrait, ReturnType, TraitItem, TraitItemFn, Type, parse2};

/// Generate a double trait which mirrors the original trait's methods and provides default
/// implementations using `unimplemented!()`.
pub fn double_trait(double_trait_name: Ident, org_trait: ItemTrait) -> ItemTrait {
    let items = org_trait
        .items
        .into_iter()
        .filter_map(transform_trait_item)
        .collect();
    ItemTrait {
        ident: double_trait_name.clone(),
        items,
        ..org_trait
    }
}

fn transform_trait_item(trait_item: TraitItem) -> Option<TraitItem> {
    // We are only interessted in transforming functions
    if let TraitItem::Fn(fn_item) = trait_item {
        transform_function(fn_item).map(TraitItem::Fn)
    } else {
        Some(trait_item)
    }
}

// Filter method which already have a default implementation
fn transform_function(mut fn_item: TraitItemFn) -> Option<TraitItemFn> {
    if fn_item.default.is_some() {
        return None;
    }

    let is_impl_future = if let ReturnType::Type(_rarrow, ty) = &fn_item.sig.output {
        if let Type::ImplTrait(ref _impl_trait) = **ty {
            // Technically, not every impl is a "impl Future", but for now we assume that.        
            true
        } else {
            false
        }
    } else {
        false
    };

    let default_impl = if is_impl_future {
        // If the method returns an impl Future, we provide a default implementation using an async
        // block, so that the compiler won't complain about not being able to infer the type of
        // `impl Future`.
        parse2(quote! {{ async { unimplemented!() }} }).unwrap()
    } else {
        // Otherwise, we provide a default implementation using unimplemented!
        parse2(quote! {{ unimplemented!() }}).unwrap()
    };

    fn_item.default = Some(default_impl);

    Some(fn_item)
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
        let double_trait = double_trait(double_trait_name, org_trait);

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
