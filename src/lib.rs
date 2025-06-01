use quote::quote;
use syn::{
    Ident, ItemTrait, TraitItem, TraitItemFn,
    parse::{Parse, ParseStream, Result},
    parse_macro_input, parse2,
};

#[proc_macro_attribute]
pub fn double(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let attr = parse_macro_input!(attr as Attr);
    let item = parse_macro_input!(item as ItemTrait);

    let output = double_impl(attr, item);

    proc_macro::TokenStream::from(output)
}

fn double_impl(attr: Attr, item: ItemTrait) -> proc_macro2::TokenStream {
    let items = item
        .items
        .into_iter()
        .filter_map(transform_trait_item)
        .collect();
    let double_trait = ItemTrait {
        ident: attr.name,
        items,
        ..item
    };

    quote! {
        #double_trait
    }
}

struct Attr {
    name: Ident,
}

impl Parse for Attr {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Attr {
            name: input.parse()?,
        })
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

#[cfg(test)]
mod tests {

    use super::{Attr, double_impl};
    use quote::quote;
    use syn::{ItemTrait, parse2};

    #[test]
    fn generate_double_trait() {
        let (attr, item) = given(quote! { MyTraitDummy }, quote! { trait MyTrait {} });

        let output = double_impl(attr, item);

        let expected = quote! {
            trait MyTraitDummy {}
        };
        assert_eq!(expected.to_string(), output.to_string());
    }

    #[test]
    fn forward_visibility() {
        // Given a public trait
        let (attr, item) = given(quote! { MyTraitDummy }, quote! { pub trait MyTrait {} });

        // When generating the dummy
        let output = double_impl(attr, item);

        // Then the generated trait should be public, too
        let expected = quote! {
            pub trait MyTraitDummy {}
        };
        assert_eq!(expected.to_string(), output.to_string());
    }

    #[test]
    fn forward_method() {
        // Given a trait with a method
        let (attr, item) = given(
            quote! { MyTraitDummy },
            quote! {
                pub trait MyTrait {
                    fn foobar();
                }
            },
        );

        // When generating the dummy
        let output = double_impl(attr, item);

        // Then the generated trait should contain that method, too
        let expected = quote! {
            pub trait MyTraitDummy {
                fn foobar () { unimplemented!() }
            }
        };
        assert_eq!(expected.to_string(), output.to_string());
    }

    #[test]
    fn respect_existing_default_impl() {
        // Given a method with a default implementation in the original trait
        let (attr, item) = given(
            quote! { MyTraitDummy },
            quote! {
                pub trait MyTrait {
                    fn foobar() { println!("Hello Default!") }
                }
            },
        );

        // When generating the dummy
        let output = double_impl(attr, item);

        // Then the generated trait should not overide the existing default
        let expected = quote! {
            pub trait MyTraitDummy {}
        };
        assert_eq!(expected.to_string(), output.to_string());
    }

    fn given(attr: proc_macro2::TokenStream, item: proc_macro2::TokenStream) -> (Attr, ItemTrait) {
        let attr: Attr = parse2(attr).unwrap();
        let item: ItemTrait = parse2(item).unwrap();
        (attr, item)
    }
}
