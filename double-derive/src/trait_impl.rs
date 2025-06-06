use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    parse2, spanned::Spanned, FnArg, Ident, ImplItem, ImplItemFn, ImplItemType, ItemImpl, ItemTrait, PatType, Token, TraitItem, TraitItemFn, TraitItemType, Visibility
};

pub fn trait_impl(double_trait_name: Ident, org_trait: ItemTrait) -> ItemImpl {
    let items = org_trait
        .items
        .into_iter()
        .filter_map(|trait_item| forward_items(trait_item, &double_trait_name))
        .collect();

    let org_trait_name = org_trait.ident;
    let impl_ = quote! {
        impl<T> #org_trait_name for T where T: #double_trait_name {

        }
    };

    let impl_ = parse2(impl_).unwrap();

    ItemImpl { items, ..impl_ }
}

fn forward_items(trait_item: TraitItem, double_trait_name: &Ident) -> Option<ImplItem> {
    // We are only interessted in transforming functions
    match trait_item {
        TraitItem::Fn(fn_item) => {
            let trait_item_fn = forward_methods(fn_item, double_trait_name);
            Some(ImplItem::Fn(trait_item_fn))
        }
        TraitItem::Type(ty_item) => {
            let impl_item_type = forward_type(ty_item, double_trait_name);
            Some(ImplItem::Type(impl_item_type))
        }
        _ => None,
    }
}

// Forward associated types from Double for original trait
fn forward_type(ty_item: TraitItemType, double_trait_name: &Ident) -> ImplItemType {
    let span = ty_item.span();
    let ident = ty_item.ident.clone();
    let ty = syn::parse_quote! { <Self as #double_trait_name>::#ident };
    let impl_item = ImplItemType {
        attrs: ty_item.attrs,
        vis: Visibility::Inherited,
        defaultness: None,
        type_token: ty_item.type_token,
        ident,
        generics: ty_item.generics,
        eq_token: Token![=](span),
        ty,
        semi_token: Token![;](span),
    };
    impl_item
}

// Forward implementation from Double for original trait
fn forward_methods(fn_item: TraitItemFn, double_trait_name: &Ident) -> ImplItemFn {
    let fn_name = fn_item.sig.ident.clone();
    let async_invocation = if fn_item.sig.asyncness.is_some() {
        quote! { .await }
    } else {
        quote! {}
    };
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
        block: parse2(
            quote! {{ <Self as #double_trait_name>::#fn_name(#(#inputs,)*)#async_invocation }},
        )
        .unwrap(),
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
                fn foobar(&mut self) {
                    <Self as MyTraitDummy>::foobar(self,)
                }
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
                fn foobar(x: i32) { <Self as MyTraitDummy>::foobar(x,) }
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
                fn foobar(one: i32, two: i32) {
                    <Self as MyTraitDummy>::foobar(one, two,)
                }
            }
        };
        assert_eq!(expected.to_string(), output.to_string());
    }

    #[test]
    fn forward_async() {
        // Given a method with a default implementation in the original trait
        let (attr, item) = given(
            quote! { MyTraitDummy },
            quote! {
                pub trait MyTrait {
                    async fn foobar(&mut self);
                }
            },
        );

        // When generating the dummy
        let output = trait_impl(attr, item);

        // Then the generated trait should not overide the existing default
        let output = quote! { #output };
        let expected = quote! {
            impl<T> MyTrait for T where T: MyTraitDummy {
                async fn foobar(&mut self) { <Self as MyTraitDummy>::foobar(self,).await }
            }
        };
        assert_eq!(expected.to_string(), output.to_string());
    }

    #[test]
    fn forward_type() {
        // Given a method with an associated type in the original trait
        let (attr, item) = given(
            quote! { MyTraitDummy },
            quote! {
                trait MyTrait {
                    type AssociatedType;
                }
            },
        );

        // When generating the dummy
        let output = trait_impl(attr, item);

        // Then the generated trait impl should forward the associated type
        let output = quote! { #output };
        let expected = quote! {
            impl<T> MyTrait for T where T: MyTraitDummy {
                type AssociatedType = <Self as MyTraitDummy>::AssociatedType;
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
