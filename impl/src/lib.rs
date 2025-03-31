#![warn(missing_docs)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::redundant_closure_for_method_calls)]
#![doc = include_str!("../README.md")]

use proc_macro2::TokenStream as TokenStream2;
use quote::{
    ToTokens,
    quote,
};
use syn::{
    Token,
    TraitItemFn,
    parse::{
        Parse,
        ParseStream,
    },
    parse_quote,
    punctuated::Punctuated,
};

#[derive(Clone)]
struct Args(Punctuated<syn::FnArg, Token![,]>);

impl Parse for Args {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self(Punctuated::parse_terminated(input)?))
    }
}

/// Wraps a trait impl fn token stream and turns it into a Bevy system
pub struct WrapImplSystem {
    args: Args,
    func: TraitItemFn,
}

impl WrapImplSystem {
    /// Create a new [`WrapImplSystem`]
    /// 
    /// # Panics
    /// - If `func` does not represent a [`TraitItemFn`]
    pub fn new(args: TokenStream2, func: TokenStream2) -> Self {
        Self {
            args: syn::parse2(args).unwrap(),
            func: syn::parse2(func).expect("this attribute macro only works on trait fns"),
        }
    }

    /// Allows the returned system to accept system input.
    #[must_use]
    pub fn with_input(mut self) -> Self {
        self.func.attrs.push(parse_quote!{ #[with_input] });
        self
    }

    /// Makes the returned system implement `ReadOnlySystem`.
    #[must_use]
    pub fn readonly(mut self) -> Self {
        self.func.attrs.push(parse_quote!{ #[readonly] });
        self
    }

    /// Boxes the returned system.
    #[must_use]
    pub fn boxed(mut self) -> Self {
        self.func.attrs.push(parse_quote!{ #[boxed] });
        self
    }
}

impl ToTokens for WrapImplSystem {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let Args(mut args) = self.args.clone();
        let mut input = self.func.clone();

        let mut with_input = false;
        let mut readonly = false;
        let mut boxed = false;

        let mut attrs = vec![];

        for attr in std::mem::take(&mut input.attrs) {
            let Some(ident) = attr.meta.path().get_ident() else {
                attrs.push(attr);
                continue;
            };

            match &*ident.to_string() {
                "with_input" => {
                    with_input = true;
                }
                "readonly" => {
                    readonly = true;
                }
                "boxed" => {
                    boxed = true;
                }
                _ => {
                    attrs.push(attr);
                }
            }
        }

        input.attrs = attrs;

        if input.default.is_some() {
            std::mem::swap(&mut args, &mut input.sig.inputs);
        }

        let sys_out = input.sig.output;

        let out = if let syn::ReturnType::Type(_, ty) = sys_out.clone() {
            *ty
        } else {
            parse_quote! { () }
        };

        let sys_in = if with_input {
            *match args.first().expect("Expected SystemInput argument") {
                syn::FnArg::Receiver(receiver) => receiver.ty.clone(),
                syn::FnArg::Typed(pat_type) => pat_type.ty.clone(),
            }
        } else {
            parse_quote! { () }
        };

        let bound: syn::TypeParamBound = if readonly {
            parse_quote! {
                ::bevy::ecs::system::ReadOnlySystem<In = #sys_in, Out = #out>
            }
        } else {
            parse_quote! {
                ::bevy::ecs::system::System<In = #sys_in, Out = #out>
            }
        };

        input.sig.output = if boxed {
            parse_quote! {
                -> ::std::boxed::Box<dyn #bound>
            }
        } else {
            parse_quote! {
                -> impl #bound
            }
        };

        if let Some(body) = &mut input.default {
            let inner = quote! {
                ::bevy::ecs::system::IntoSystem::into_system(move |#args| #sys_out #body)
            };

            if boxed {
                *body = parse_quote! {{ ::std::boxed::Box::new(#inner) }};
            } else {
                *body = parse_quote! {{ #inner }};
            }
        }

        input.to_tokens(tokens);
    }
}
