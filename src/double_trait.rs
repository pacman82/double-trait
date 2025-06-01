use quote::quote;
use syn::{parse2, Ident, ItemTrait, TraitItem, TraitItemFn};

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

    fn_item.default = Some(parse2(quote! {{ unimplemented!() }}).unwrap());

    Some(fn_item)
}
