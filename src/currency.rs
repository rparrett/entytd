use bevy::prelude::*;

pub struct CurrencyPlugin;
impl Plugin for CurrencyPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Currency>();
    }
}

#[derive(Resource, Default)]
pub struct Currency {
    pub metal: u32,
    pub crystal: u32,
}
