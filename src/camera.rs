use bevy::prelude::*;

pub struct CameraPlugin;
impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn);
        app.add_systems(Update, update);
    }
}

fn spawn(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn update(
    keys: Res<Input<KeyCode>>,
    mut query: Query<&mut Transform, With<Camera2d>>,
    time: Res<Time>,
) {
    let speed = 50.;

    let x = keys.pressed(KeyCode::Right) as i8 - keys.pressed(KeyCode::Left) as i8;
    let y = keys.pressed(KeyCode::Up) as i8 - keys.pressed(KeyCode::Down) as i8;
    let dir = Vec2::new(x as f32, y as f32).normalize_or_zero();

    if dir != Vec2::ZERO {
        for mut camera in &mut query {
            camera.translation += dir.extend(0.) * time.delta_seconds() * speed;
        }
    }
}
