use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    FnArg, Ident, ImplItem, ImplItemFn, ItemImpl, ItemTrait, PatType, TraitItem, TraitItemFn,
    Visibility, parse2,
};

pub fn trait_impl(double_trait_name: Ident, org_trait: ItemTrait) -> ItemImpl {
    let items = org_trait
        .items
        .into_iter()
        .filter_map(|trait_item| map_methods(trait_item, &double_trait_name))
        .collect();

    let org_trait_name = org_trait.ident;
    let impl_ = quote! {
        impl<T> #org_trait_name for T where T: #double_trait_name {

        }
    };

    let impl_ = parse2(impl_).unwrap();

    ItemImpl { items, ..impl_ }
}

fn map_methods(trait_item: TraitItem, double_trait_name: &Ident) -> Option<ImplItem> {
    // We are only interessted in transforming functions
    if let TraitItem::Fn(fn_item) = trait_item {
        let trait_item_fn = function_with_forwarding(fn_item, double_trait_name);
        Some(ImplItem::Fn(trait_item_fn))
    } else {
        None
    }
}

// Filter method which already have a default implementation
fn function_with_forwarding(fn_item: TraitItemFn, double_trait_name: &Ident) -> ImplItemFn {
    let fn_name = fn_item.sig.ident.clone();
    let inputs = fn_item
        .sig
        .inputs
        .clone()
        .into_iter()
        .map(parameter_to_argument);
    ImplItemFn {
        attrs: Vec::new(),
        vis: Visibility::Inherited,
        defaultness: None,
        sig: fn_item.sig,
        block: parse2(quote! {{ #double_trait_name::#fn_name(#(#inputs,)*) }}).unwrap(),
    }
}

fn parameter_to_argument(input: FnArg) -> TokenStream {
    match input {
        FnArg::Receiver(_) => quote! { self },
        FnArg::Typed(PatType { pat, .. }) => {
            quote! { # pat}
        }
    }
}

#[cfg(test)]
mod tests {

    use super::{Ident, trait_impl};
    use quote::quote;
    use syn::{ItemTrait, parse2};

    #[test]
    fn forward_self() {
        // Given a method with a default implementation in the original trait
        let (attr, item) = given(
            quote! { MyTraitDummy },
            quote! {
                pub trait MyTrait {
                    fn foobar(&mut self);
                }
            },
        );

        // When generating the dummy
        let output = trait_impl(attr, item);

        // Then the generated trait should not overide the existing default
        let output = quote! { #output };
        let expected = quote! {
            impl<T> MyTrait for T where T: MyTraitDummy {
                fn foobar(&mut self) { MyTraitDummy::foobar(self,) }
            }
        };
        assert_eq!(expected.to_string(), output.to_string());
    }

    #[test]
    fn forward_non_self_parameter() {
        // Given a method with a default implementation in the original trait
        let (attr, item) = given(
            quote! { MyTraitDummy },
            quote! {
                pub trait MyTrait {
                    fn foobar(x: i32);
                }
            },
        );

        // When generating the dummy
        let output = trait_impl(attr, item);

        // Then the generated trait should not overide the existing default
        let output = quote! { #output };
        let expected = quote! {
            impl<T> MyTrait for T where T: MyTraitDummy {
                fn foobar(x: i32) { MyTraitDummy::foobar(x,) }
            }
        };
        assert_eq!(expected.to_string(), output.to_string());
    }

    #[test]
    fn forward_multiple_arguments() {
        // Given a method with a default implementation in the original trait
        let (attr, item) = given(
            quote! { MyTraitDummy },
            quote! {
                pub trait MyTrait {
                    fn foobar(one: i32, two: i32);
                }
            },
        );

        // When generating the dummy
        let output = trait_impl(attr, item);

        // Then the generated trait should not overide the existing default
        let output = quote! { #output };
        let expected = quote! {
            impl<T> MyTrait for T where T: MyTraitDummy {
                fn foobar(one: i32, two: i32) { MyTraitDummy::foobar(one, two,) }
            }
        };
        assert_eq!(expected.to_string(), output.to_string());
    }

    fn given(attr: proc_macro2::TokenStream, item: proc_macro2::TokenStream) -> (Ident, ItemTrait) {
        let attr: Ident = parse2(attr).unwrap();
        let item: ItemTrait = parse2(item).unwrap();
        (attr, item)
    }
}
