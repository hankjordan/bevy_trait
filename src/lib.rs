#![warn(missing_docs)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::redundant_closure_for_method_calls)]
#![doc = include_str!("../README.md")]

use bevy_trait_impl::WrapImplSystem;
use proc_macro::TokenStream;
use quote::ToTokens;

/// Attribute to use a Trait fn like it's a [`System`](https://docs.rs/bevy/latest/bevy/ecs/system/trait.System.html).
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
/// Makes the fn return an object that impls [`ReadOnlySystem`](https://docs.rs/bevy/latest/bevy/ecs/system/trait.ReadOnlySystem.html).
///
/// ### `#[with_input]`
/// Makes the fn return a system trait object that accepts [`SystemInput`](https://docs.rs/bevy/latest/bevy/ecs/system/trait.SystemInput.html).
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
    WrapImplSystem::new(args.into(), func.into())
        .into_token_stream()
        .into()
}

/// Attribute to use a Trait fn like it's a [`System`](https://docs.rs/bevy/latest/bevy/ecs/system/trait.System.html), with [`SystemInput`](https://docs.rs/bevy/latest/bevy/ecs/system/trait.SystemInput.html).
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
    WrapImplSystem::new(args.into(), func.into())
        .with_input()
        .into_token_stream()
        .into()
}

/// Attribute to use a Trait fn like it's a [`ReadOnlySystem`](https://docs.rs/bevy/latest/bevy/ecs/system/trait.ReadOnlySystem.html).
///
/// See [`macro@system`] for usage and examples
///
/// Alias of
/// ```ignore
/// #[system]
/// #[readonly]
/// ```
#[proc_macro_attribute]
pub fn readonly_system(args: TokenStream, func: TokenStream) -> TokenStream {
    WrapImplSystem::new(args.into(), func.into())
        .readonly()
        .into_token_stream()
        .into()
}

/// Attribute to use a Trait fn like it's a [`ReadOnlySystem`](https://docs.rs/bevy/latest/bevy/ecs/system/trait.ReadOnlySystem.html), with [`SystemInput`](https://docs.rs/bevy/latest/bevy/ecs/system/trait.SystemInput.html).
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
    WrapImplSystem::new(args.into(), func.into())
        .readonly()
        .with_input()
        .into_token_stream()
        .into()
}

/// Attribute to use a Trait fn like it's a boxed [`System`](https://docs.rs/bevy/latest/bevy/ecs/system/trait.System.html).
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
    WrapImplSystem::new(args.into(), func.into())
        .boxed()
        .into_token_stream()
        .into()
}

/// Attribute to use a Trait fn like it's a boxed [`System`](https://docs.rs/bevy/latest/bevy/ecs/system/trait.System.html), with [`SystemInput`](https://docs.rs/bevy/latest/bevy/ecs/system/trait.SystemInput.html).
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
    WrapImplSystem::new(args.into(), func.into())
        .boxed()
        .with_input()
        .into_token_stream()
        .into()
}

/// Attribute to use a Trait fn like it's a boxed [`ReadOnlySystem`](https://docs.rs/bevy/latest/bevy/ecs/system/trait.ReadOnlySystem.html).
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
    WrapImplSystem::new(args.into(), func.into())
        .boxed()
        .readonly()
        .into_token_stream()
        .into()
}

/// Attribute to use a Trait fn like it's a boxed [`ReadOnlySystem`](https://docs.rs/bevy/latest/bevy/ecs/system/trait.ReadOnlySystem.html), with [`SystemInput`](https://docs.rs/bevy/latest/bevy/ecs/system/trait.SystemInput.html).
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
    WrapImplSystem::new(args.into(), func.into())
        .boxed()
        .readonly()
        .with_input()
        .into_token_stream()
        .into()
}
