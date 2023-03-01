#![warn(missing_docs)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::redundant_closure_for_method_calls)]
#![doc = include_str!("../README.md")]

use proc_macro::TokenStream;
use quote::{
    quote,
    ToTokens,
};
use syn::{
    parse_macro_input,
    parse_quote,
    punctuated::Punctuated,
    token::Comma,
    Block,
    FnArg,
    ImplItem,
    ImplItemMethod,
    Item,
    Meta,
    ReturnType,
    Stmt,
};

macro_rules! parse_system_input {
    ($i:ident) => {
        if let ImplItem::Method(input) = parse_macro_input!($i as ImplItem) {
            input
        } else {
            panic!("this attribute macro only works on trait methods")
        }
    };
}

fn impl_system<F>(input: ImplItemMethod, output: ReturnType, body: F) -> TokenStream
where
    F: FnOnce(&Punctuated<FnArg, Comma>, &Block) -> proc_macro2::TokenStream,
{
    let args = &input.sig.inputs;
    let block = &input.block;

    let body = if let Some(Stmt::Item(Item::Verbatim(item))) = block.stmts.get(0) {
        item.clone()
    } else {
        let mut body = body(args, block);

        for attr in input.attrs {
            let meta = attr.parse_meta().unwrap();
            let ident = meta.path().get_ident().unwrap();
            let tokens = ident.into_token_stream();
            let name = ident.to_string();

            let value = if let Meta::List(list) = meta {
                Some(list.nested.into_token_stream())
            } else {
                None
            };

            match &*name {
                "in_set" | "in_base_set" | "before" | "after" | "run_if" | "ambiguous_with"
                | "in_schedule" => {
                    body = quote! { #body.#tokens(#value) };
                }
                "no_default_base_set" | "ambiguous_with_all" | "on_startup" => {
                    body = quote! { #body.#tokens() };
                }
                _ => {}
            }
        }

        quote! { { #body } }
    };

    let mut sig = input.sig;

    sig.inputs.clear();
    sig.output = output;

    quote! {
        #sig #body
    }
    .into()
}

/// Attribute to turn a method of a Trait into a `BoxedSystem`.
///
/// Use this attribute when you want to prevent the implementer of your trait from defining scheduling metadata.
/// # Example
/// ```
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
/// ```
#[proc_macro_attribute]
pub fn system(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_system_input!(input);

    impl_system(
        input,
        parse_quote! { -> bevy::ecs::system::BoxedSystem },
        |args, block| {
            quote! { Box::new(bevy::ecs::system::IntoSystem::into_system(|#args| #block)) }
        },
    )
}

/// Attribute to turn a method of a Trait into a`SystemConfig`.
///
/// Use this attribute when you want the implementer of your trait to be able to define scheduling metadata.
/// # Example
/// ```
/// trait Interactive {
///     #[system_config]
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
///     #[system_config]
///     #[in_base_set(CoreSet::PreUpdate)] // Implementer can specify SystemSet
///     #[before(apply_system_buffers)] // ... and even relative ordering
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
/// ```
/// # Attributes
/// Add any of these attributes alongside `#[system_config]` to define schedule metadata for the system.
/// <br/>&nbsp;
/// ##  Trait `IntoSystemConfig`
/// Types that can be converted into a `SystemConfig`.
/// <br/>&nbsp;
/// ### &ensp; attr `#[in_set(set: impl SystemSet)]`
/// &emsp;&emsp; *See `IntoSystemConfig::in_set`.*
/// ### &ensp; attr `#[in_base_set(set: impl SystemSet)]`
/// &emsp; *See `IntoSystemConfig::in_base_set`.*
/// ### &ensp; attr `#[no_default_base_set]`
/// &emsp;&emsp; *See `IntoSystemConfig::no_default_base_set`.*
/// ### &ensp; attr `#[before(set: impl IntoSystemSet<_>)]`
/// &emsp;&emsp; *See `IntoSystemConfig::before`.*
/// ### &ensp; attr `#[after(set: impl IntoSystemSet<_>)]`
/// &emsp;&emsp; *See `IntoSystemConfig::after`.*
/// ### &ensp; attr `#[run_if(condition: impl Condition<_>)]`
/// &emsp;&emsp; *See `IntoSystemConfig::run_if`.*
/// ### &ensp; attr `#[ambiguous_with(set: impl IntoSystemSet<_>)]`
/// &emsp;&emsp; *See `IntoSystemConfig::ambiguous_with`.*
/// ### &ensp; attr `#[ambiguous_with_all]`
/// &emsp;&emsp; *See `IntoSystemConfig::ambiguous_with_all`.*
/// <br/>&nbsp;<br/>&nbsp;
#[proc_macro_attribute]
pub fn system_config(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_system_input!(input);

    impl_system(
        input,
        parse_quote! { -> bevy::ecs::schedule::SystemConfig },
        |args, block| {
            quote! { (|#args| #block).into_config() }
        },
    )
}

/// Attribute to turn a method of a Trait into a `SystemAppConfig`.
///
/// Use this attribute when you want the implementer of your trait to be able to define App-aware scheduling metadata.
/// # Example
/// ```
/// trait Interactive {
///     #[system_app_config]
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
///     #[system_app_config]
///     #[in_base_set(CoreSet::PreUpdate)] // Implementer can specify SystemSet
///     #[before(apply_system_buffers)] // ... relative ordering ...
///     #[in_schedule(CoreSchedule::Main)] // ... and even schedule
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
/// ```
/// # Attributes
/// Add any of these attributes alongside `#[system_app_config]` to define schedule metadata for the system.
/// <br/>&nbsp;
/// ##  Trait `IntoSystemConfig`
/// Types that can be converted into a `SystemConfig`.
/// <br/>&nbsp;
/// ### &ensp; attr `#[in_set(set: impl SystemSet)]`
/// &emsp;&emsp; *See `IntoSystemConfig::in_set`.*
/// ### &ensp; attr `#[in_base_set(set: impl SystemSet)]`
/// &emsp; *See `IntoSystemConfig::in_base_set`.*
/// ### &ensp; attr `#[no_default_base_set]`
/// &emsp;&emsp; *See `IntoSystemConfig::no_default_base_set`.*
/// ### &ensp; attr `#[before(set: impl IntoSystemSet<_>)]`
/// &emsp;&emsp; *See `IntoSystemConfig::before`.*
/// ### &ensp; attr `#[after(set: impl IntoSystemSet<_>)]`
/// &emsp;&emsp; *See `IntoSystemConfig::after`.*
/// ### &ensp; attr `#[run_if(condition: impl Condition<_>)]`
/// &emsp;&emsp; *See `IntoSystemConfig::run_if`.*
/// ### &ensp; attr `#[ambiguous_with(set: impl IntoSystemSet<_>)]`
/// &emsp;&emsp; *See `IntoSystemConfig::ambiguous_with`.*
/// ### &ensp; attr `#[ambiguous_with_all]`
/// &emsp;&emsp; *See `IntoSystemConfig::ambiguous_with_all`.*
/// <br/>&nbsp;<br/>&nbsp;
/// ## Trait `IntoSystemAppConfig`
/// Types that can be converted into a `SystemAppConfig`.
/// <br/>&nbsp;
/// ### &ensp; attr `#[in_schedule(schedule: impl ScheduleLabel)]`
/// &emsp;&emsp; *See `IntoSystemAppConfig::in_schedule`.*
/// ### &ensp; attr `#[on_startup]`
/// &emsp;&emsp; *See `IntoSystemAppConfig::on_startup`.*
#[proc_macro_attribute]
pub fn system_app_config(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_system_input!(input);

    impl_system(
        input,
        parse_quote! { -> bevy::app::SystemAppConfig },
        |args, block| {
            quote! { (|#args| #block).into_app_config() }
        },
    )
}
