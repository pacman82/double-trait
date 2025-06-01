use quote::quote;
use syn::{ItemImpl, ItemTrait, parse2};

use crate::Attr;

pub fn trait_impl(attr: Attr, item: ItemTrait) -> ItemImpl {
    let double_trait_name = attr.name;
    let org_trait_name = item.ident;

    let impl_ = quote! {
        impl<T> #org_trait_name for T where T: #double_trait_name {

        }
    };

    let impl_ = parse2(impl_).unwrap();

    ItemImpl { ..impl_ }
}
