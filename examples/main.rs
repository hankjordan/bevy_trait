use bevy::prelude::*;
use bevy_trait::*;

trait Initializable {
    #[system]
    fn init();

    #[system]
    fn empty() {}

    #[system]
    fn default(_entities: Query<Entity>) {
        let _a = 25;
    }

    #[system]
    fn generic<C: Component>();

    #[system]
    fn needs_build(data: i32);

    #[system]
    fn build_generic<C: Component + std::fmt::Debug>(component: C);

    fn desugared_system() -> impl System;

    fn desugared_boxed_system() -> bevy::ecs::system::BoxedSystem;
}

struct Cactus;

impl Initializable for Cactus {
    #[system]
    fn init(_transforms: Query<&Transform>) {
        info!("Init!");
    }

    #[system]
    fn generic<C: Component>(_query: Query<&C>) {}

    #[system(data: i32)]
    fn needs_build(query: Query<&Transform, With<Visibility>>) {
        info!("Data: {:?}", data);

        for tf in &query {
            info!("Transform {:?}", tf);
        }
    }

    #[system(component: C)]
    fn build_generic<C: Component + std::fmt::Debug>(query: Query<&C>) {
        info!("Component: {:?}", component);

        for other in &query {
            info!("Other: {:?}", other);
        }
    }

    fn desugared_system() -> impl System {
        bevy::ecs::system::IntoSystem::into_system(|tfs: Query<&Transform>| {
            for tf in &tfs {
                info!("Transform {:?}", tf)
            }
        })
    }

    fn desugared_boxed_system() -> bevy::ecs::system::BoxedSystem {
        Box::new(bevy::ecs::system::IntoSystem::into_system(
            |tfs: Query<&Transform>| {
                for tf in &tfs {
                    info!("Transform {:?}", tf)
                }
            },
        ))
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, Cactus::init())
        .add_systems(
            Update,
            (Cactus::needs_build(100), Cactus::generic::<Transform>()),
        )
        .run();
}
