mod default_body;

use self::default_body::default_body_strategy;

use syn::{
    FnArg, Ident, ItemTrait, Pat, PatWild, Token, TraitItem, TraitItemFn, punctuated::Punctuated,
    spanned::Spanned, token::Comma,
};

/// Generate a double trait which mirrors the original trait's methods and provides default
/// implementations using `unimplemented!()`.
pub fn double_trait(org_trait: ItemTrait) -> syn::Result<ItemTrait> {
    let items = org_trait
        .items
        .into_iter()
        .map(|item| transform_trait_item(item, org_trait.ident.clone()))
        .collect::<syn::Result<_>>()?;
    Ok(ItemTrait { items, ..org_trait })
}

fn transform_trait_item(trait_item: TraitItem, double_trait_name: Ident) -> syn::Result<TraitItem> {
    // We are only interessted in transforming functions
    let transformed_trait_item = match trait_item {
        TraitItem::Fn(fn_item) => TraitItem::Fn(transform_function(fn_item, double_trait_name)?),
        _ => {
            // If it is not a function, we forward the original Item
            trait_item
        }
    };
    Ok(transformed_trait_item)
}

// Give methods a default implementation, if they do not have one already.
fn transform_function(
    mut fn_item: TraitItemFn,
    double_trait_name: Ident,
) -> syn::Result<TraitItemFn> {
    if fn_item.default.is_some() {
        return Ok(fn_item);
    }

    // We are stripping parameter names in order to avoid warnings regarding unused variables, since
    // our default implementation is not making use of any arguments.
    strip_parameter_names(&mut fn_item.sig.inputs);

    let return_type_info = default_body_strategy(&fn_item.sig.output);
    let fn_name = fn_item.sig.ident.clone();

    let default_impl = return_type_info.default_body(&fn_item, double_trait_name, fn_name);

    fn_item.default = Some(default_impl);

    Ok(fn_item)
}

fn strip_parameter_names(input: &mut Punctuated<FnArg, Comma>) {
    for arg in input {
        // We are only interested in pattern type. No need to transform `self`
        if let FnArg::Typed(pat_type) = arg {
            *pat_type.pat = Pat::Wild(PatWild {
                attrs: Vec::new(),
                underscore_token: Token![_](pat_type.span()),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::double_trait;
    use quote::quote;
    use syn::{ItemTrait, parse2};

    #[test]
    fn default_impl_for_method_returning_result_unit() {
        // Given
        let org_trait = given(quote! {
            trait MyTrait {
                fn method(&self) -> Result<(), MyError>;
            }
        });

        // When
        let double_trait = double_trait(org_trait).unwrap();

        // Then
        let actual = quote! { #double_trait };
        let expected = quote! {
            trait MyTrait {
                fn method(&self) -> Result<(), MyError> {
                    let inner = {};
                    # [allow (unreachable_code)]
                    Ok(inner)
                }
            }
        };
        assert_eq!(actual.to_string(), expected.to_string());
    }

    #[test]
    fn default_impl_for_method_returning_result_any_type() {
        // Given
        let org_trait = given(quote! {
            trait MyTrait {
                fn method(&self) -> Result<MyOk, MyError>;
            }
        });

        // When
        let double_trait = double_trait(org_trait).unwrap();

        // Then
        let actual = quote! { #double_trait };
        let expected = quote! {
            trait MyTrait {
                fn method(&self) -> Result<MyOk, MyError> {
                    let inner = {
                        let double_trait_name = stringify!(MyTrait);
                        let fn_name = stringify!(method);
                        unimplemented ! ("{double_trait_name}::{fn_name}")
                    };
                    # [allow (unreachable_code)]
                    Ok(inner)
                }
            }
        };
        assert_eq!(actual.to_string(), expected.to_string());
    }

    #[test]
    fn default_impl_for_method_with_impl_future_output_unit() {
        // Given an original trait with a method returning an impl Future
        let org_trait = given(quote! {
            trait MyTrait {
                fn method(&self) -> impl Future<Output = ()>;
            }
        });

        // When generating the double trait
        let double_trait = double_trait(org_trait).unwrap();

        // Then the double trait should have a default implementation for the method which uses
        // an async block
        let actual = quote! { #double_trait };
        let expected = quote! {
            trait MyTrait {
                fn method(&self) -> impl Future<Output = ()> {
                    async { }
                }
            }
        };
        assert_eq!(actual.to_string(), expected.to_string());
    }

    #[test]
    fn default_impl_for_method_with_impl_future_output_i32() {
        // Given an original trait with a method returning an impl Future
        let org_trait = given(quote! {
            trait MyTrait {
                fn method(&self) -> impl Future<Output = i32>;
            }
        });

        // When generating the double trait
        let double_trait = double_trait(org_trait).unwrap();

        // Then the double trait should have a default implementation for the method which uses
        // an async block
        let actual = quote! { #double_trait };
        let expected = quote! {
            trait MyTrait {
                fn method(&self) -> impl Future<Output = i32> {
                    async {
                        let double_trait_name = stringify!(MyTrait);
                        let fn_name = stringify!(method);
                        unimplemented!("{double_trait_name}::{fn_name}")
                    }
                }
            }
        };
        assert_eq!(expected.to_string(), actual.to_string());
    }

    #[test]
    fn default_impl_for_method_with_impl_iterator_return() {
        // Given an original trait with a method returning an impl Iterator
        let org_trait = given(quote! {
            trait MyTrait {
                fn method(&self) -> impl Iterator<Item = String>;
            }
        });

        // When generating the double trait
        let double_trait = double_trait(org_trait).unwrap();

        // Then the double trait should have a default implementation for the method which uses
        // an empty array iterator
        let actual = quote! { #double_trait };
        let expected = quote! {
            trait MyTrait {
                fn method(&self) -> impl Iterator<Item = String> {
                    #[allow(unreachable_code)]
                    std::iter::from_fn(move | | {
                        if false {
                            Some({
                                let double_trait_name = stringify!(MyTrait);
                                let fn_name = stringify!(method);
                                unimplemented!("{double_trait_name}::{fn_name}")
                            })
                        } else {
                            None
                        }
                    })
                }
            }
        };
        assert_eq!(actual.to_string(), expected.to_string());
    }

    #[test]
    fn empty_default_implementation_if_function_does_not_return_anything() {
        // Given
        let org_trait = given(quote! {
            trait MyTrait {
                fn method(x: i32);
            }
        });

        // When
        let double_trait = double_trait(org_trait).unwrap();

        // Then
        let actual = quote! { #double_trait };
        let expected = quote! {
            trait MyTrait {
                fn method(_: i32) {}
            }
        };
        assert_eq!(actual.to_string(), expected.to_string());
    }

    #[test]
    fn empty_default_implementation_if_function_returns_option() {
        // Given
        let org_trait = given(quote! {
            trait MyTrait {
                fn method() -> Option<i32>;
            }
        });

        // When
        let double_trait = double_trait(org_trait).unwrap();

        // Then
        let actual = quote! { #double_trait };
        let expected = quote! {
            trait MyTrait {
                fn method() -> Option<i32> {
                    None
                }
            }
        };
        assert_eq!(actual.to_string(), expected.to_string());
    }

    #[test]
    fn default_implementation_for_function_with_i32_result() {
        // Given an original trait with a method returning an i32
        let org_trait = given(quote! {
            trait MyTrait {
                fn method(x: i32) -> i32;
            }
        });

        // When generating the double trait
        let double_trait = double_trait(org_trait).unwrap();

        // Then the double trait should have a default implementation with unimplemented!() which
        // uses the trait and function name in the error message.
        let actual = quote! { #double_trait };
        let expected = quote! {
            trait MyTrait {
                fn method(_: i32) -> i32 {
                    let double_trait_name = stringify!(MyTrait);
                    let fn_name = stringify!(method);
                    unimplemented!("{double_trait_name}::{fn_name}")
                }
            }
        };
        assert_eq!(actual.to_string(), expected.to_string());
    }

    #[test]
    fn compiler_error_for_unknown_return_impl() {
        // Given an original trait with a method returning an impl to an unsupported trait
        let org_trait = given(quote! {
            trait MyTrait {
                fn method() -> impl UnsupportedTrait;
            }
        });

        // When generating the double trait
        let double_trait = double_trait(org_trait).unwrap();

        // Then the double trait should have a default implementation which generates a nice compile
        // error.
        let actual = quote! { #double_trait };
        let expected = quote! {
            trait MyTrait {
                fn method() -> impl UnsupportedTrait {
                    compile_error!(
                        "impl Trait is currently not supported by double-trait. Apart from the \
                        special cases of `impl Future` and `impl Stream`."
                    )
                }
            }
        };
        assert_eq!(actual.to_string(), expected.to_string());
    }

    #[cfg(feature = "stream")]
    #[test]
    fn default_impl_stream_with_stream_feature_activated() {
        // Given an original trait with a method returning an impl to an unsupported trait
        let org_trait = given(quote! {
            trait MyTrait {
                fn method() -> impl Stream;
            }
        });

        // When generating the double trait
        let double_trait = double_trait(org_trait).unwrap();

        // Then the double trait should have a default implementation which generates a nice compile
        // error.
        let actual = quote! { #double_trait };
        let expected = quote! {
            trait MyTrait {
                fn method() -> impl Stream {
                    futures_util::stream::empty()
                }
            }
        };
        assert_eq!(actual.to_string(), expected.to_string());
    }

    #[cfg(not(feature = "stream"))]
    #[test]
    fn compiler_error_for_impl_stream_with_stream_feature_deactivated() {
        // Given an original trait with a method returning an impl to an unsupported trait
        let org_trait = given(quote! {
            trait MyTrait {
                fn method() -> impl Stream;
            }
        });

        // When generating the double trait
        let double_trait = double_trait(org_trait).unwrap();

        // Then the double trait should have a default implementation which generates a nice compile
        // error.
        let actual = quote! { #double_trait };
        let expected = quote! {
            trait MyTrait {
                fn method() -> impl Stream {
                    compile_error!(
                        "impl Stream is only supported if the `stream` feature of \
                                double-trait is activated."
                    )
                }
            }
        };
        assert_eq!(actual.to_string(), expected.to_string());
    }

    #[test]
    fn strip_parameter_names_from_default_implementation() {
        // Given an original trait with a method returning an impl Future
        let org_trait = given(quote! {
            trait MyTrait {
                fn method(x: i32) -> i32;
            }
        });

        // When generating the double trait
        let double_trait = double_trait(org_trait).unwrap();

        // Then the double trait should have a default implementation for the method which uses
        // an async block
        let actual = quote! { #double_trait };
        let expected = quote! {
            trait MyTrait {
                fn method(_: i32) -> i32{
                    let double_trait_name = stringify!(MyTrait);
                    let fn_name = stringify!(method);
                    unimplemented!("{double_trait_name}::{fn_name}")
                }
            }
        };
        assert_eq!(actual.to_string(), expected.to_string());
    }

    fn given(item: proc_macro2::TokenStream) -> ItemTrait {
        parse2(item).unwrap()
    }
}
