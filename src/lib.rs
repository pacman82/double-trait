use quote::quote;
use syn::{
    Ident,
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
    // let attr = proc_macro2::TokenStream::from(attr);
    let item = proc_macro2::TokenStream::from(item);

    let output = double_impl(attr, item);

    proc_macro::TokenStream::from(output)
}

fn double_impl(attr: Attr, item: proc_macro2::TokenStream) -> proc_macro2::TokenStream {
    let double_trait_name = attr.name;
    quote! {
        pub trait #double_trait_name {}
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
    use syn::{Ident, parse2};

    #[test]
    fn make_double_can_annotate_trait() {
        let attr = quote! {
            MyTraitDummy
        };
        let attr: Attr = parse2(attr).unwrap();
        let item = quote! {
            trait MyTrait {}
        };
        let output = double_impl(attr, item);

        let expected = quote! {
            pub trait MyTraitDummy {}
        };
        assert_eq!(expected.to_string(), output.to_string());
    }
}
