use quote::quote;
use syn::{
    Ident, ItemTrait,
    parse::{Parse, ParseStream, Result},
    parse_macro_input,
};

#[proc_macro_attribute]
pub fn double(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    // Convert `proc_macro::TokenStreams` from and to input proc_marco2::TokenStream in order to
    // enable better testability of `double_impl`
    let attr = parse_macro_input!(attr as Attr);
    let item = parse_macro_input!(item as ItemTrait);

    let output = double_impl(attr, item);

    proc_macro::TokenStream::from(output)
}

fn double_impl(attr: Attr, item: ItemTrait) -> proc_macro2::TokenStream {
    let double_trait_name = attr.name;
    let visibility = item.vis;
    quote! {
        #visibility trait #double_trait_name {}
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

#[cfg(test)]
mod tests {

    use super::{Attr, double_impl};
    use quote::quote;
    use syn::{ItemTrait, parse2};

    #[test]
    fn generate_double_trait() {
        let attr = quote! {
            MyTraitDummy
        };
        let attr: Attr = parse2(attr).unwrap();
        let item = quote! {
            trait MyTrait {}
        };
        let item: ItemTrait = parse2(item).unwrap();
        let output = double_impl(attr, item);

        let expected = quote! {
            trait MyTraitDummy {}
        };
        assert_eq!(expected.to_string(), output.to_string());
    }

    #[test]
    fn forward_visibility() {
        // Given a public trait
        let attr = quote! {
            MyTraitDummy
        };
        let attr: Attr = parse2(attr).unwrap();
        let item = quote! {
            pub trait MyTrait {}
        };
        let item: ItemTrait = parse2(item).unwrap();
        let output = double_impl(attr, item);

        // Then the generated trait should be public, too
        let expected = quote! {
            pub trait MyTraitDummy {}
        };
        assert_eq!(expected.to_string(), output.to_string());
    }
}
