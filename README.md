# Bevy_trait

[![][img_bevy]][bevy] [![][img_version]][crates] [![][img_doc]][doc] [![][img_license]][license] [![][img_tracking]][tracking] [![][img_downloads]][crates]

Macros for creating Traits in Bevy.

## System

Turn a trait fn into a Bevy system

```ignore
trait Interactive {
    #[system]
    fn update(damage: f32);
}

/*
// Desugars to
trait Interactive {
    fn update(damage: f32) -> impl System;
}
*/

#[derive(Component)]
struct Health(f32);

#[derive(Component, Copy, Clone)]
struct Cactus;

impl Interactive for Cactus {
    #[system(damage: f32)]
    fn update(
        cacti: Query<&GlobalTransform, With<Cactus>>,
        creatures: Query<(&GlobalTransform, &mut Health), Without<Cactus>>,
    ) {
        // This is a normal Bevy system and accepts SystemParams as such.
        for cactus_gtf in &cacti {
            info!("Damage {:?}", damage); // You can also use params passed into the System builder.

            // ...
        }
    }
}
 
fn run() {
    let system = Cactus::update(42); // This is a System ...
 
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Update, system) // ... that you can add to an App
        .run();
}
```

## Compatibility

Since `bevy_trait` does not rely on `bevy` directly, it is typically compatible across many different versions.

| Bevy Version                           | Crate Version |
| -------------------------------------- | ------------- |
| `0.15`                                 | `0.3`         |
| `0.10`, `0.11`, `0.12`, `0.13`, `0.14` | `0.1`, `0.2`  |

## License

`bevy_trait` is dual-licensed under MIT and Apache-2.0.

[img_bevy]: https://img.shields.io/badge/Bevy-0.15-blue
[img_version]: https://img.shields.io/crates/v/bevy_trait.svg
[img_doc]: https://docs.rs/bevy_trait/badge.svg
[img_license]: https://img.shields.io/badge/license-MIT%2FApache-blue.svg
[img_downloads]: https://img.shields.io/crates/d/bevy_trait.svg
[img_tracking]: https://img.shields.io/badge/Bevy%20tracking-released%20version-lightblue
[bevy]: https://crates.io/crates/bevy/0.15.3
[crates]: https://crates.io/crates/bevy_trait
[doc]: https://docs.rs/bevy_trait/
[license]: https://github.com/hankjordan/bevy_trait#license
[tracking]: https://github.com/bevyengine/bevy/blob/main/docs/plugins_guidelines.md#main-branch-tracking
