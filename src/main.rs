use bevy::{
    input::mouse::{MouseScrollUnit, MouseWheel},
    math::vec3,
    prelude::*,
};
use std::f32::consts::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (light_temperature_update, light_switch_update, wall_update).chain(),
        )
        .run();
}
#[derive(Component)]
struct ColorTemperature {
    value: f32,
}

#[derive(Component)]
struct LightSwitch;

#[derive(Component)]
struct WallTile {
    x: usize,
    y: usize,
}

#[derive(Resource)]
struct WallTilePalette {
    materials: Vec<Handle<StandardMaterial>>,
    seed: u64,
}

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // wall
    const WALL_SIZE_X: f32 = 18.0;
    const WALL_SIZE_Y: f32 = 5.0;
    const TILE_SIZE: f32 = 0.2;

    let mesh = meshes.add(shape::Plane::from_size(TILE_SIZE).into());
    commands.insert_resource(WallTilePalette {
        materials: [
            Color::hex("#0C356A").unwrap(),
            Color::hex("#0174BE").unwrap(),
            Color::hex("#FFC436").unwrap(),
        ]
        .into_iter()
        .map(|color| materials.add(color.into()))
        .collect(),
        seed: rand::random::<u64>(),
    });

    for x in 0..=(WALL_SIZE_X / TILE_SIZE) as usize {
        for y in 0..=(WALL_SIZE_Y / TILE_SIZE) as usize {
            commands
                .spawn(PbrBundle {
                    mesh: mesh.clone(),
                    // material:
                    transform: Transform::from_rotation(Quat::from_rotation_x(FRAC_PI_2))
                        .with_translation(vec3(
                            x as f32 * TILE_SIZE - WALL_SIZE_X / 2.0,
                            y as f32 * TILE_SIZE - WALL_SIZE_Y / 2.0,
                            0.0,
                        )),
                    ..default()
                })
                .insert(WallTile { x, y });
        }
    }

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
    commands
        .spawn(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cylinder {
                radius: 0.15,
                height: 0.3,
                resolution: 32,
                ..Default::default()
            })),
            material: switch_material.clone(),
            transform: Transform::from_rotation(Quat::from_rotation_x(FRAC_PI_2)),
            ..default()
        })
        .insert(LightSwitch);
    commands
        .spawn(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cylinder {
                radius: 0.03,
                height: 0.32,
                resolution: 32,
                ..Default::default()
            })),
            material: materials.add(Color::GRAY.into()),
            transform: Transform::from_xyz(0.1, 0.0, 0.0)
                .with_rotation(Quat::from_rotation_x(FRAC_PI_2)),
            ..default()
        })
        .insert(ColorTemperature { value: 0.5 })
        .insert(LightSwitch);

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
    mut query_light: Query<&mut DirectionalLight>,
    mut query_switch: Query<&mut Transform, With<LightSwitch>>,
) {
    if mouse_input.just_released(MouseButton::Middle) {
        for mut light in query_light.iter_mut() {
            if light.illuminance > 0.0 {
                light.illuminance = 0.0;
            } else {
                light.illuminance = 5000.0;
            }
        }
    }
    for mut switch in query_switch.iter_mut() {
        switch.translation.z = if mouse_input.pressed(MouseButton::Middle) {
            -0.05
        } else {
            0.0
        };
    }
}

fn light_temperature_update(
    mut scroll_events: EventReader<MouseWheel>,
    mut query_light: Query<&mut DirectionalLight>,
    mut query_switch: Query<&mut Transform, With<ColorTemperature>>,
    mut query_temperature: Query<&mut ColorTemperature>,
) {
    let mut query_temperature = query_temperature.single_mut();

    for event in scroll_events.read() {
        query_temperature.value += match event.unit {
            MouseScrollUnit::Line => event.y,
            MouseScrollUnit::Pixel => {
                println!("pixel {}", event.y);
                event.y * 10.0
            }
        } * 0.05;
    }
    query_temperature.value = f32::clamp(query_temperature.value, 0.0, 1.0);

    for mut switch in query_switch.iter_mut() {
        let angle = TAU * query_temperature.value * 0.8;
        switch.translation = Vec3::new(0.1 * angle.cos(), 0.1 * angle.sin(), 0.0);
    }

    for mut light in query_light.iter_mut() {
        light.color = color_temperature_to_rgb(3000.0 + query_temperature.value * 4000.0)
            .extend(1.0)
            .into();
    }
}

fn color_temperature_to_rgb(temperature: f32) -> Vec3 {
    // Values from: http://blenderartists.org/forum/showthread.php?270332-OSL-Goodness&p=2268693&viewfull=1#post2268693
    let m = if temperature <= 6500.0 {
        Mat3::from_cols(
            vec3(0.0, -2902.1955373783176, -8257.7997278925690),
            vec3(0.0, 1669.5803561666639, 2575.2827530017594),
            vec3(1.0, 1.3302673723350029, 1.8993753891711275),
        )
    } else {
        Mat3::from_cols(
            vec3(1745.0425298314172, 1216.6168361476490, -8257.7997278925690),
            vec3(-2666.3474220535695, -2173.1012343082230, 2575.2827530017594),
            vec3(0.55995389139931482, 0.70381203140554553, 1.8993753891711275),
        )
    };
    let temperature = temperature.clamp(1000.0, 40000.0);
    Vec3::lerp(
        (m.col(0) / (Vec3::splat(temperature.clamp(1000.0, 40000.0)) + m.col(1)) + m.col(2))
            .clamp(Vec3::ZERO, Vec3::ONE),
        Vec3::ONE,
        smoothstep(1000.0, 0.0, temperature),
    )
}

fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

fn wall_update(
    mut wall_tiles: Query<(&WallTile, &mut Handle<StandardMaterial>)>,
    palette: Res<WallTilePalette>,
) {
    use rand::Rng;
    use rand::SeedableRng;

    // Set random seed
    let mut rng = rand::rngs::StdRng::seed_from_u64(palette.seed);

    const PATTERN_SIZE: usize = 5;
    // Clever Anke stuff: Make last color less likely than the others
    let pattern = (0..PATTERN_SIZE * PATTERN_SIZE)
        .map(|_| (rng.gen::<usize>() % (palette.materials.len() * 2 - 1)) % palette.materials.len())
        .collect::<Vec<_>>();

    let pattern_index = |tile: &WallTile| {
        let x = ((tile.x % (PATTERN_SIZE * 2 - 2)) as i32 - PATTERN_SIZE as i32 + 2).abs() as usize;
        let y = ((tile.y % (PATTERN_SIZE * 2 - 2)) as i32 - PATTERN_SIZE as i32 + 2).abs() as usize;
        x + y * PATTERN_SIZE
    };

    for (tile, mut material) in wall_tiles.iter_mut() {
        *material = palette.materials[pattern[pattern_index(tile)]].clone();
    }
}
