use quote::quote;
use syn::{parse2, ImplItem, ImplItemFn, ItemImpl, ItemTrait, TraitItem, TraitItemFn, Visibility};

use crate::Attr;

pub fn trait_impl(attr: Attr, org_trait: ItemTrait) -> ItemImpl {
    let items = org_trait
        .items
        .into_iter()
        .filter_map(map_methods)
        .collect();
    let double_trait_name = attr.name;
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

fn map_methods(trait_item: TraitItem) -> Option<ImplItem> {
    // We are only interessted in transforming functions
    if let TraitItem::Fn(fn_item) = trait_item {
        let trait_item_fn = function_with_forwarding(fn_item);
        Some(ImplItem::Fn(trait_item_fn))
    } else {
        None
    }
}

// Filter method which already have a default implementation
fn function_with_forwarding(fn_item: TraitItemFn) -> ImplItemFn {
    ImplItemFn {
        attrs: Vec::new(),
        vis: Visibility::Inherited,
        defaultness: None,
        sig: fn_item.sig,
        block: parse2(quote! {{ unimplemented!() }}).unwrap(),
    }
}