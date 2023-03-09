use bevy::prelude::*;
use bevy_trait::*;

trait Initializable {
    #[system]
    fn init();

    #[system_config]
    fn init_config();

    #[system_app_config]
    fn init_app_config();

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
}

struct Cactus;

impl Initializable for Cactus {
    #[system]
    fn init(_transforms: Query<&Transform>) {
        info!("Init!");
    }

    #[system_config]
    #[in_base_set(CoreSet::PostUpdate)]
    #[before(apply_system_buffers)]
    fn init_config(_query: Query<&Transform>) {}

    #[system_app_config]
    fn init_app_config() {}

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
}

fn main() {
    let cactus_init = Cactus::init();

    App::new()
        .add_plugins(DefaultPlugins)
        .add_system(cactus_init)
        .run();
}
