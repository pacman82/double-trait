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
    let attr = parse_macro_input!(attr as Attr);
    let item = parse_macro_input!(item as ItemTrait);

    let output = double_impl(attr, item);

    proc_macro::TokenStream::from(output)
}

fn double_impl(attr: Attr, item: ItemTrait) -> proc_macro2::TokenStream {
    let double_trait = ItemTrait {
        ident: attr.name,
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
                fn foobar ();
            }
        };
        assert_eq!(expected.to_string(), output.to_string());
    }

    fn given(attr: proc_macro2::TokenStream, item: proc_macro2::TokenStream) -> (Attr, ItemTrait) {
        let attr: Attr = parse2(attr).unwrap();
        let item: ItemTrait = parse2(item).unwrap();
        (attr, item)
    }
}
