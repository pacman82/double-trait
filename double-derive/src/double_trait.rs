use quote::{quote, quote_spanned};
use syn::{
    AngleBracketedGenericArguments, Block, FnArg, GenericArgument, Ident, ItemTrait, Pat, PatWild,
    PathArguments, ReturnType, Token, TraitItem, TraitItemFn, Type, TypeParamBound, parse2,
    punctuated::Punctuated, spanned::Spanned, token::Comma,
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

    let return_type_info = return_type_info(&fn_item.sig.output);
    let fn_name = fn_item.sig.ident.clone();

    let default_impl = default_impl(&fn_item, double_trait_name, &return_type_info, fn_name);

    fn_item.default = Some(default_impl);

    Ok(fn_item)
}

fn default_impl(
    fn_item: &TraitItemFn,
    double_trait_name: Ident,
    return_type_info: &ReturnTypeInfo,
    fn_name: Ident,
) -> Block {
    match return_type_info {
        ReturnTypeInfo::ImplFuture { output } => {
            // Treat missing Output type like other, i.e. use unimplemented!() in the async block
            let output = output.as_deref().unwrap_or(&ReturnTypeInfo::Other);
            let inner = default_impl(fn_item, double_trait_name, output, fn_name);
            // If the method returns an impl Future, we provide a default implementation using an
            // async block, so that the compiler won't complain about not being able to infer the
            // type of `impl Future`.
            parse2(quote! {{ async #inner }}).unwrap()
        }
        ReturnTypeInfo::ImplIterator => {
            // If the method returns an impl Iterator, we provide a default implementation using an
            // empty array, so that the compiler won't complain about not being able to infer the
            // type of `impl Iterator`.
            parse2(quote! {{ [].into_iter() }}).unwrap()
        }
        ReturnTypeInfo::Other => {
            // Otherwise, we provide a default implementation using unimplemented!
            // We can unwrap here, this body should always compile
            parse2(quote! {{
                let double_trait_name = stringify!(#double_trait_name);
                let fn_name = stringify!(#fn_name);
                unimplemented!("{double_trait_name}::{fn_name}")
            }})
            .unwrap()
        }
        ReturnTypeInfo::Empty => {
            // If the function does not return anything, we provide an empty default implementation
            // to avoid using `unimplemented!()`.
            parse2(quote! { { } }).unwrap()
        }
        ReturnTypeInfo::UnknownImpl => parse2(quote_spanned! {
            fn_item.sig.output.span() => {
                compile_error!(
                    "impl Trait is currently not supported by double-trait. Apart from the \
                    special case of impl Future."
            )}
        })
        .unwrap(),
    }
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

fn return_type_info(output: &ReturnType) -> ReturnTypeInfo {
    if let ReturnType::Type(_rarrow, ty) = output {
        type_info(ty)
    } else {
        ReturnTypeInfo::Empty
    }
}

fn type_info(ty: &Type) -> ReturnTypeInfo {
    match *ty {
        Type::ImplTrait(ref impl_trait) => {
            let mut trait_bounds = impl_trait.bounds.iter().filter_map(|b| match b {
                TypeParamBound::Trait(trait_bound) => Some(trait_bound),
                TypeParamBound::Lifetime(_)
                | TypeParamBound::PreciseCapture(_)
                | TypeParamBound::Verbatim(_)
                | _ => None,
            });
            let first_trait_bound = trait_bounds
                .next()
                .expect("At least one trait bound expected in impl trait.");
            let first_path_segment = first_trait_bound
                .path
                .segments
                .first()
                .expect("There must be at least one path segment in trait bound");
            let identifier = &first_path_segment.ident.to_string();
            match identifier.as_str() {
                "Future" => {
                    let output = associated_output_type(&first_path_segment.arguments);
                    // If the first trait bound is Future, we assume that this is an impl Future.
                    ReturnTypeInfo::ImplFuture {
                        output: output.map(|ty| Box::new(type_info(ty))),
                    }
                }
                "Iterator" => ReturnTypeInfo::ImplIterator,
                _ => ReturnTypeInfo::UnknownImpl,
            }
        }
        Type::Tuple(ref tuple_type) => {
            if tuple_type.elems.is_empty() {
                ReturnTypeInfo::Empty
            } else {
                ReturnTypeInfo::Other
            }
        }
        _ => ReturnTypeInfo::Other,
    }
}

/// Find the associated output type of an impl Future trait. E.g. the `i64` in impl Future<Output=i64>.
fn associated_output_type(future_trait_args: &PathArguments) -> Option<&Type> {
    let PathArguments::AngleBracketed(AngleBracketedGenericArguments { args, .. }) =
        future_trait_args
    else {
        return None;
    };
    args.iter()
        // Only look at associated types
        .filter_map(|arg| {
            let GenericArgument::AssocType(at) = arg else {
                return None;
            };
            Some(at)
        })
        // Find the associated type named "Output"
        .find(|at| at.ident == "Output")
        // Return the type of the associated type
        .map(|at| &at.ty)
}

#[derive(Debug)]
enum ReturnTypeInfo {
    /// If the function does not return, we want the default implementation to be empty, rather than
    /// using `unimplemented!()`.
    Empty,
    /// Indicates that the return type is an impl Future. We want to know this, so we can wrap
    /// `unimplemented!()` in an async block.
    ImplFuture {
        output: Option<Box<ReturnTypeInfo>>,
    },
    ImplIterator,
    UnknownImpl,
    Other,
}

#[cfg(test)]
mod tests {
    use super::{ReturnTypeInfo, double_trait, return_type_info};
    use quote::quote;
    use syn::{ItemTrait, ReturnType, parse2};

    #[test]
    fn return_type_info_unit() {
        let rt: ReturnType = parse2(quote! {-> () }).unwrap();
        assert!(matches!(return_type_info(&rt), ReturnTypeInfo::Empty));
    }

    #[test]
    fn return_type_info_i34() {
        let rt: ReturnType = parse2(quote! {-> i32 }).unwrap();
        assert!(matches!(return_type_info(&rt), ReturnTypeInfo::Other));
    }

    #[test]
    fn return_type_info_impl_future_i32() {
        let rt: ReturnType = parse2(quote! {-> impl Future<Output = i32> }).unwrap();
        let ReturnTypeInfo::ImplFuture {
            output: Some(output),
        } = return_type_info(&rt)
        else {
            panic!("Expected ReturnTypeInfo::ImplFuture with Some output");
        };
        assert!(matches!(*output, ReturnTypeInfo::Other));
    }

    #[test]
    fn return_type_info_impl_future_unit() {
        let rt: ReturnType = parse2(quote! {-> impl Future<Output = ()> }).unwrap();
        let ReturnTypeInfo::ImplFuture {
            output: Some(output),
        } = return_type_info(&rt)
        else {
            panic!("Expected ReturnTypeInfo::ImplFuture with Some output");
        };
        assert!(matches!(*output, ReturnTypeInfo::Empty));
    }

    #[test]
    fn return_type_info_impl_future_impl_iterator_i32() {
        let rt: ReturnType =
            parse2(quote! {-> impl Future<Output = impl Iterator<Item=i32>> }).unwrap();
        let ReturnTypeInfo::ImplFuture {
            output: Some(output),
        } = return_type_info(&rt)
        else {
            panic!("Expected ReturnTypeInfo::ImplFuture with Some output");
        };
        assert!(matches!(*output, ReturnTypeInfo::ImplIterator));
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
                    [].into_iter()
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
                    special case of impl Future."
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
