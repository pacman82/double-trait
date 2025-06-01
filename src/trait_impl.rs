use quote::quote;
use syn::{parse2, Ident, ImplItem, ImplItemFn, ItemImpl, ItemTrait, TraitItem, TraitItemFn, Visibility};

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

    ItemImpl { 
        items,
        ..impl_
    }
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
    let inputs = fn_item.sig.inputs.clone().into_iter();
    ImplItemFn {
        attrs: Vec::new(),
        vis: Visibility::Inherited,
        defaultness: None,
        sig: fn_item.sig,
        block: parse2(quote! {{ #double_trait_name::#fn_name(#(#inputs)*) }}).unwrap(),
    }
}