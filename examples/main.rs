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
}

struct Cactus;

impl Initializable for Cactus {
    #[system]
    fn init(_transforms: Query<&Transform>) {
        let _b = 17;
    }

    #[system_config]
    #[in_base_set(CoreSet::PostUpdate)]
    #[before(apply_system_buffers)]
    fn init_config(_query: Query<&Transform>) {}

    #[system_app_config]
    fn init_app_config() {}

    #[system]
    fn generic<C: Component>(_query: Query<&C>) {}
}

fn main() {
    todo!()
}
