use quote::{quote, quote_spanned};
use syn::{
    AngleBracketedGenericArguments, Block, GenericArgument, Ident, PathArguments, ReturnType,
    TraitItemFn, Type, TypeParamBound, parse2, spanned::Spanned,
};

/// Since we ignore all arguments in the body the return type alone decides what the body of the
/// default implementation is.
#[derive(Debug, PartialEq, Eq)]
pub enum DefaultBodyStrategy {
    /// If the function does not return, we want the default implementation to be empty, rather than
    /// using `unimplemented!()`.
    Empty,
    /// Indicates that the return type is an impl Future. We want to know this, so we can wrap
    /// `unimplemented!()` in an async block.
    ImplFuture {
        /// The associated Output type of the Future
        output: Option<Box<DefaultBodyStrategy>>,
    },
    ImplIterator {
        /// The associated Item type of the Iterator
        item: Option<Box<DefaultBodyStrategy>>,
    },
    ImplStream {
        /// The associated Item type of the Stream
        _item: Option<Box<DefaultBodyStrategy>>,
    },
    Result {
        // The `Ok` type of the Result
        ok: Box<DefaultBodyStrategy>,
    },
    Option,
    UnknownImpl,
    Other,
}

impl DefaultBodyStrategy {
    pub fn default_body(
        &self,
        fn_item: &TraitItemFn,
        double_trait_name: Ident,
        fn_name: Ident,
    ) -> Block {
        match self {
            DefaultBodyStrategy::ImplFuture { output } => {
                // Treat missing Output type like other, i.e. use unimplemented!() in the async
                // block
                let output = output.as_deref().unwrap_or(&DefaultBodyStrategy::Other);
                let inner = output.default_body(fn_item, double_trait_name, fn_name);
                // If the method returns an impl Future, we provide a default implementation using
                // an async block, so that the compiler won't complain about not being able to infer
                // the type of `impl Future`.
                parse2(quote! {{ async #inner }}).unwrap()
            }
            DefaultBodyStrategy::ImplIterator { item } => {
                // If the method returns an impl Iterator, we provide a default implementation using
                // an iterator returning no elements.

                let item = item.as_deref().unwrap_or(&DefaultBodyStrategy::Other);
                let inner = item.default_body(fn_item, double_trait_name, fn_name);

                // We are constructing an empty interator, but we still want to be able to infer an
                // element type from `#inner` if possible.
                parse2(quote! {{
                    #[allow(unreachable_code)]
                    std::iter::from_fn(move || {
                        if false {
                            Some(#inner)
                        } else {
                            None
                        }
                    })
                }})
                .unwrap()
            }
            DefaultBodyStrategy::ImplStream { _item: _ } => {
                if cfg!(feature = "stream") {
                    parse2(quote! {{
                        futures_util::stream::empty()
                    }})
                    .unwrap()
                } else {
                    parse2(quote_spanned! {
                        fn_item.sig.output.span() => {
                            compile_error!(
                                "impl Stream is only supported if the `stream` feature of \
                                double-trait is activated."
                            )
                        }
                    })
                    .unwrap()
                }
            }
            DefaultBodyStrategy::Other => {
                // Otherwise, we provide a default implementation using unimplemented!
                // We can unwrap here, this body should always compile
                parse2(quote! {{
                    let double_trait_name = stringify!(#double_trait_name);
                    let fn_name = stringify!(#fn_name);
                    unimplemented!("{double_trait_name}::{fn_name}")
                }})
                .unwrap()
            }
            DefaultBodyStrategy::Empty => {
                // If the function does not return anything, we provide an empty default
                // implementation to avoid using `unimplemented!()`.
                parse2(quote! { { } }).unwrap()
            }
            DefaultBodyStrategy::Option => {
                // If the function does not return anything, we provide an empty default
                // implementation to avoid using `unimplemented!()`.
                parse2(quote! { { None } }).unwrap()
            }
            DefaultBodyStrategy::Result { ok } => {
                // If the method returns a Result, we provide a default implementation as if it were
                // infalliable, wrapped in `Ok`.

                let inner = ok.default_body(fn_item, double_trait_name, fn_name);

                // We are constructing an empty interator, but we still want to be able to infer an
                // element type from `#inner` if possible.
                parse2(quote! {{
                    let inner = #inner;
                    #[allow(unreachable_code)]
                    Ok(inner)
                }})
                .unwrap()
            }
            DefaultBodyStrategy::UnknownImpl => parse2(quote_spanned! {
                fn_item.sig.output.span() => {
                    compile_error!(
                        "impl Trait is currently not supported by double-trait. Apart from the \
                        special cases of `impl Future` and `impl Stream`."
                    )
                }
            })
            .unwrap(),
        }
    }
}

pub fn default_body_strategy(output: &ReturnType) -> DefaultBodyStrategy {
    if let ReturnType::Type(_rarrow, ty) = output {
        type_info(ty)
    } else {
        DefaultBodyStrategy::Empty
    }
}

fn type_info(ty: &Type) -> DefaultBodyStrategy {
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
                    let output = assoctiated_type(&first_path_segment.arguments, "Output");
                    // If the first trait bound is Future, we assume that this is an impl Future.
                    DefaultBodyStrategy::ImplFuture {
                        output: output.map(|ty| Box::new(type_info(ty))),
                    }
                }
                "Iterator" => {
                    let item = assoctiated_type(&first_path_segment.arguments, "Item");
                    DefaultBodyStrategy::ImplIterator {
                        item: item.map(|ty| Box::new(type_info(ty))),
                    }
                }
                "Stream" => {
                    let item = assoctiated_type(&first_path_segment.arguments, "Item");
                    DefaultBodyStrategy::ImplStream {
                        _item: item.map(|ty| Box::new(type_info(ty))),
                    }
                }
                _ => DefaultBodyStrategy::UnknownImpl,
            }
        }
        Type::Tuple(ref tuple_type) => {
            if tuple_type.elems.is_empty() {
                DefaultBodyStrategy::Empty
            } else {
                DefaultBodyStrategy::Other
            }
        }
        Type::Path(ref type_path) => {
            let Some(last) = type_path.path.segments.last() else {
                return DefaultBodyStrategy::Other;
            };
            if last.ident.to_string() == "Option" {
                return DefaultBodyStrategy::Option;
            }
            if last.ident.to_string() != "Result" {
                return DefaultBodyStrategy::Other;
            }
            let PathArguments::AngleBracketed(ref generic_arguments) = last.arguments else {
                return DefaultBodyStrategy::Other;
            };
            let Some(generic_argument) = generic_arguments.args.first() else {
                return DefaultBodyStrategy::Other;
            };
            let GenericArgument::Type(ok) = generic_argument else {
                return DefaultBodyStrategy::Other;
            };
            DefaultBodyStrategy::Result {
                ok: Box::new(type_info(ok)),
            }
        }
        _ => DefaultBodyStrategy::Other,
    }
}

/// Find the associated output type of an impl Future trait. E.g. the `i64` in impl Future<Output=i64>.
fn assoctiated_type<'a>(
    future_trait_args: &'a PathArguments,
    associated: &str,
) -> Option<&'a Type> {
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
        // Find the associated type
        .find(|at| at.ident == associated)
        // Return the type of the associated type
        .map(|at| &at.ty)
}

#[cfg(test)]
mod tests {
    use super::{DefaultBodyStrategy, default_body_strategy};
    use quote::quote;
    use syn::{ReturnType, parse2};

    #[test]
    fn return_type_info_unit() {
        let rt: ReturnType = parse2(quote! {-> () }).unwrap();
        assert!(matches!(
            default_body_strategy(&rt),
            DefaultBodyStrategy::Empty
        ));
    }

    #[test]
    fn return_type_info_i32() {
        let rt: ReturnType = parse2(quote! {-> i32 }).unwrap();
        assert!(matches!(
            default_body_strategy(&rt),
            DefaultBodyStrategy::Other
        ));
    }

    #[test]
    fn return_type_info_option_i32() {
        let rt: ReturnType = parse2(quote! {-> Option<i32> }).unwrap();
        assert!(matches!(
            default_body_strategy(&rt),
            DefaultBodyStrategy::Option
        ));
    }

    #[test]
    fn return_type_info_impl_future_i32() {
        let rt: ReturnType = parse2(quote! {-> impl Future<Output = i32> }).unwrap();
        let DefaultBodyStrategy::ImplFuture {
            output: Some(output),
        } = default_body_strategy(&rt)
        else {
            panic!("Expected ReturnTypeInfo::ImplFuture with Some output");
        };
        assert!(matches!(*output, DefaultBodyStrategy::Other));
    }

    #[test]
    fn return_type_info_result_unit() {
        let rt: ReturnType = parse2(quote! {-> Result<(), MyError> }).unwrap();
        let DefaultBodyStrategy::Result { ok } = default_body_strategy(&rt) else {
            panic!("Expected ReturnTypeInfo::Result");
        };
        assert!(matches!(*ok, DefaultBodyStrategy::Empty));
    }

    #[test]
    fn return_type_info_result_vec() {
        let rt: ReturnType = parse2(quote! {-> Result<Vec<i32>, MyError> }).unwrap();
        let rti = default_body_strategy(&rt);
        let expected = DefaultBodyStrategy::Result {
            ok: Box::new(DefaultBodyStrategy::Other),
        };
        assert_eq!(expected, rti);
    }

    #[test]
    fn return_type_info_impl_future_unit() {
        let rt: ReturnType = parse2(quote! {-> impl Future<Output = ()> }).unwrap();
        let DefaultBodyStrategy::ImplFuture {
            output: Some(output),
        } = default_body_strategy(&rt)
        else {
            panic!("Expected ReturnTypeInfo::ImplFuture with Some output");
        };
        assert!(matches!(*output, DefaultBodyStrategy::Empty));
    }

    #[test]
    fn return_type_info_impl_future_impl_iterator_i32() {
        let rt: ReturnType =
            parse2(quote! {-> impl Future<Output = impl Iterator<Item=i32>> }).unwrap();
        let DefaultBodyStrategy::ImplFuture {
            output: Some(output),
        } = default_body_strategy(&rt)
        else {
            panic!("Expected ReturnTypeInfo::ImplFuture with Some output");
        };
        assert!(matches!(
            *output,
            DefaultBodyStrategy::ImplIterator { item: Some(_) }
        ));
    }
}
