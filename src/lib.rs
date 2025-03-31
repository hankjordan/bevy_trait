#![warn(missing_docs)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::redundant_closure_for_method_calls)]
#![doc = include_str!("../README.md")]

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{
    quote,
    ToTokens,
};
use syn::{
    parse::{
        Parse,
        ParseStream,
    },
    parse_quote,
    punctuated::Punctuated,
    Token,
    TraitItemFn,
};

#[derive(Clone)]
struct Args(Punctuated<syn::FnArg, Token![,]>);

impl Parse for Args {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self(Punctuated::parse_terminated(input)?))
    }
}

struct IntoSystem {
    args: Args,
    func: TraitItemFn,
}

impl IntoSystem {
    fn new(args: TokenStream, func: TokenStream) -> Self {
        Self {
            args: syn::parse(args).unwrap(),
            func: syn::parse(func).expect("this attribute macro only works on trait fns"),
        }
    }
}

impl ToTokens for IntoSystem {
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

/// Attribute to use a Trait fn like it's a [`System`](bevy::ecs::system::System).
///
/// ### `#[system]`
/// Defines a system builder fn without parameters.
///
/// ### `#[system(arg: T, ...)]`
/// Add args to the macro to add parameters to the builder fn.
///
/// # Attributes
///
/// ### `#[boxed]`
/// Makes the fn box the returned system trait object.
///
/// ### `#[readonly]`
/// Makes the fn return an object that impls [`ReadOnlySystem`](bevy::ecs::system::ReadOnlySystem).
///
/// ### `#[with_input]`
/// Makes the fn return a system trait object that accepts [`SystemInput`](bevy::ecs::system::SystemInput).
///
/// # Examples
/// ```
/// # use bevy::prelude::*;
/// # use bevy_trait::*;
/// trait Interactive {
///     #[system]
///     fn update();
/// }
///
/// #[derive(Component)]
/// struct Health(f32);
///
/// #[derive(Component)]
/// struct Cactus;
///
/// impl Interactive for Cactus {
///     #[system]
///     fn update(
///         cacti: Query<&GlobalTransform, With<Cactus>>,
///         creatures: Query<(&GlobalTransform, &mut Health), Without<Cactus>>,
///     ) {
///         // This is a normal Bevy system and accepts SystemParams as such.
///         for cactus_gtf in &cacti {
///             // ...
///         }
///     }
/// }
///
/// fn run() {
///     let system = Cactus::update(); // This is a System ...
///
///     App::new()
///         .add_plugins(DefaultPlugins)
///         .add_systems(Update, system) // ... that you can add to an App
///         .run();
/// }
/// ```
/// You can also pass arguments into the system builder.
/// ```
/// # use bevy::prelude::*;
/// # use bevy_trait::*;
/// trait Interactive {
///     #[system]
///     fn update(&self, name: String);
/// }
///
/// #[derive(Component)]
/// struct Health(f32);
///
/// #[derive(Component, Copy, Clone)]
/// struct Cactus(u32);
///
/// impl Interactive for Cactus {
///     // We need to take &self to be able to turn cactus into a dyn object, but we don't have to use it
///     #[system(&self, name: String)]
///     fn update(
///         cacti: Query<&GlobalTransform, With<Cactus>>,
///         creatures: Query<(&GlobalTransform, &mut Health), Without<Cactus>>,
///     ) {
///         // This is a normal Bevy system and accepts SystemParams as such.
///         for gtf in &cacti {
///             info!("Cactus GlobalTransform: {:?}, name: {:?}", gtf, name);
///             // ...
///         }
///     }
/// }
///
/// fn run() {
///     let cactus = Cactus(42);
///     let system = cactus.update("A cactus".into()); // This is a System ...
///
///     App::new()
///         .add_plugins(DefaultPlugins)
///         .add_systems(Update, system) // ... that you can add to an App
///         .run();
/// }
/// ```
#[proc_macro_attribute]
pub fn system(args: TokenStream, func: TokenStream) -> TokenStream {
    IntoSystem::new(args, func).into_token_stream().into()
}

/// Attribute to use a Trait fn like it's a [`System`](bevy::ecs::system::System), with [`SystemInput`](bevy::ecs::system::SystemInput).
///
/// See [`macro@system`] for usage and examples
///
/// Alias of
/// ```ignore
/// #[system]
/// #[with_input]
/// ```
#[proc_macro_attribute]
pub fn system_with_input(args: TokenStream, func: TokenStream) -> TokenStream {
    let mut input = IntoSystem::new(args, func);
    input.func.attrs.push(parse_quote! { #[with_input] });
    input.into_token_stream().into()
}

/// Attribute to use a Trait fn like it's a [`ReadOnlySystem`](bevy::ecs::system::ReadOnlySystem).
///
/// See [`system`](attr.system) for usage and examples
///
/// Alias of
/// ```ignore
/// #[system]
/// #[readonly]
/// ```
#[proc_macro_attribute]
pub fn readonly_system(args: TokenStream, func: TokenStream) -> TokenStream {
    let mut input = IntoSystem::new(args, func);
    input.func.attrs.push(parse_quote! { #[readonly] });
    input.into_token_stream().into()
}

/// Attribute to use a Trait fn like it's a [`ReadOnlySystem`](bevy::ecs::system::ReadOnlySystem), with [`SystemInput`](bevy::ecs::system::SystemInput).
///
/// See [`macro@system`] for usage and examples
///
/// Alias of
/// ```ignore
/// #[system]
/// #[readonly]
/// #[with_input]
/// ```
#[proc_macro_attribute]
pub fn readonly_system_with_input(args: TokenStream, func: TokenStream) -> TokenStream {
    let mut input = IntoSystem::new(args, func);
    input.func.attrs.push(parse_quote! { #[readonly] });
    input.func.attrs.push(parse_quote! { #[with_input] });
    input.into_token_stream().into()
}

/// Attribute to use a Trait fn like it's a boxed [`System`](bevy::ecs::system::System).
///
/// See [`macro@system`] for usage and examples
///
/// Alias of
/// ```ignore
/// #[system]
/// #[boxed]
/// ```
#[proc_macro_attribute]
pub fn boxed_system(args: TokenStream, func: TokenStream) -> TokenStream {
    let mut input = IntoSystem::new(args, func);
    input.func.attrs.push(parse_quote! { #[boxed] });
    input.into_token_stream().into()
}

/// Attribute to use a Trait fn like it's a boxed [`System`](bevy::ecs::system::System), with [`SystemInput`](bevy::ecs::system::SystemInput).
///
/// See [`macro@system`] for usage and examples
///
/// Alias of
/// ```ignore
/// #[system]
/// #[boxed]
/// #[with_input]
/// ```
#[proc_macro_attribute]
pub fn boxed_system_with_input(args: TokenStream, func: TokenStream) -> TokenStream {
    let mut input = IntoSystem::new(args, func);
    input.func.attrs.push(parse_quote! { #[boxed] });
    input.func.attrs.push(parse_quote! { #[with_input] });
    input.into_token_stream().into()
}

/// Attribute to use a Trait fn like it's a boxed [`ReadOnlySystem`](bevy::ecs::system::ReadOnlySystem).
///
/// See [`macro@system`] for usage and examples
///
/// Alias of
/// ```ignore
/// #[system]
/// #[boxed]
/// #[readonly]
/// ```
#[proc_macro_attribute]
pub fn boxed_readonly_system(args: TokenStream, func: TokenStream) -> TokenStream {
    let mut input = IntoSystem::new(args, func);
    input.func.attrs.push(parse_quote! { #[boxed] });
    input.func.attrs.push(parse_quote! { #[readonly] });
    input.into_token_stream().into()
}

/// Attribute to use a Trait fn like it's a boxed [`ReadOnlySystem`](bevy::ecs::system::ReadOnlySystem), with [`SystemInput`](bevy::ecs::system::SystemInput).
///
/// See [`macro@system`] for usage and examples
///
/// Alias of
/// ```ignore
/// #[system]
/// #[boxed]
/// #[readonly]
/// #[with_input]
/// ```
#[proc_macro_attribute]
pub fn boxed_readonly_system_with_input(args: TokenStream, func: TokenStream) -> TokenStream {
    let mut input = IntoSystem::new(args, func);
    input.func.attrs.push(parse_quote! { #[boxed] });
    input.func.attrs.push(parse_quote! { #[readonly] });
    input.func.attrs.push(parse_quote! { #[with_input] });
    input.into_token_stream().into()
}
