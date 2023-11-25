use bevy::{math::vec3, prelude::*};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, light_switch_update)
        .run();
}

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // wall
    commands.spawn(PbrBundle {
        mesh: meshes.add(shape::Plane::from_size(10.0).into()),
        material: materials.add(Color::AQUAMARINE.into()),
        transform: Transform::from_rotation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)),
        ..default()
    });

    let switch_material = materials.add(Color::WHITE.into());
    // switch
    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Box::from_corners(
            vec3(-0.2, -0.3, 0.0),
            vec3(0.2, 0.3, 0.05),
        ))),
        material: switch_material.clone(),
        ..default()
    });
    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Cylinder {
            radius: 0.15,
            height: 0.3,
            resolution: 32,
            ..Default::default()
        })),
        material: switch_material.clone(),
        transform: Transform::from_rotation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)),
        ..default()
    });
    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Cylinder {
            radius: 0.03,
            height: 0.32,
            resolution: 32,
            ..Default::default()
        })),
        material: materials.add(Color::BLACK.into()),
        transform: Transform::from_xyz(0.1, 0.0, 0.0)
            .with_rotation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)),
        ..default()
    });

    // light
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            color: Color::WHITE,
            illuminance: 5000.0,
            shadows_enabled: true,
            ..Default::default()
        },
        transform: Transform::from_rotation(Quat::from_euler(
            EulerRot::XYZ,
            -std::f32::consts::TAU * 0.15,
            -std::f32::consts::TAU / 16.0,
            0.0,
        )),
        ..default()
    });
    // camera
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(-0.5, 1.0, 4.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
}

fn light_switch_update(
    mouse_input: Res<Input<MouseButton>>,
    mut query: Query<&mut DirectionalLight>,
) {
    if mouse_input.just_released(MouseButton::Middle) {
        for mut light in query.iter_mut() {
            if light.illuminance > 0.0 {
                light.illuminance = 0.0;
            } else {
                light.illuminance = 5000.0;
            }
        }
    }
}
