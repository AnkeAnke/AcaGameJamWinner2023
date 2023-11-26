use achievements::{
    achievement_update, setup_achievements, AchievementQueue, AchievementToBeAdded,
};
use bevy::{
    core_pipeline::clear_color::ClearColorConfig,
    input::mouse::{MouseScrollUnit, MouseWheel},
    math::vec3,
    prelude::*,
};
use bevy_hanabi::prelude::*;
use std::{
    f32::consts::*,
    path::{Path, PathBuf},
};

mod achievements;

// https://github.com/bevyengine/bevy/pull/10383
#[doc(hidden)]
pub fn _embedded_asset_path(
    crate_name: &str,
    src_prefix: &Path,
    file_path: &Path,
    asset_path: &Path,
) -> PathBuf {
    let mut maybe_parent = file_path.parent();
    let after_src = loop {
        let Some(parent) = maybe_parent else {
            panic!("Failed to find src_prefix {src_prefix:?} in {file_path:?}")
        };
        if parent.ends_with(src_prefix) {
            break file_path.strip_prefix(parent).unwrap();
        }
        maybe_parent = parent.parent();
    };
    let asset_path = after_src.parent().unwrap().join(asset_path);
    Path::new(crate_name).join(asset_path)
}

// Workaround https://github.com/bevyengine/bevy/issues/10377
macro_rules! embedded_asset {
    ($app: ident, $path: expr) => {{
        let embedded = $app
            .world
            .resource_mut::<bevy::asset::io::embedded::EmbeddedAssetRegistry>();
        let crate_name = module_path!().split(':').next().unwrap();
        let path =
            _embedded_asset_path(crate_name, "src".as_ref(), file!().as_ref(), $path.as_ref());
        let full_path = std::path::PathBuf::new();
        embedded.insert_asset(full_path, &path, include_bytes!($path));
    }};
}

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "AcaGameJam Winner 2023".to_string(),
            ..default()
        }),
        ..default()
    }))
    .add_plugins(HanabiPlugin)
    //.add_plugins((bevy::diagnostic::LogDiagnosticsPlugin::default(), bevy::diagnostic::FrameTimeDiagnosticsPlugin))
    .add_systems(Startup, (setup, setup_achievements))
    .add_systems(
        Update,
        (
            light_temperature_update,
            light_switch_update,
            wall_update,
            achievement_update,
            update_clock_hands,
            clock_achievement_check,
        ),
    );
    embedded_asset!(app, "./PublicPixel-z84yD.ttf");
    embedded_asset!(app, "./achievement.ogg");
    app.run();
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
    number_material: Handle<StandardMaterial>,
    seed: u64,
}

#[derive(Resource)]
struct Score {
    value: u32,
}

const WALL_SIZE_X: f32 = 18.0;
const WALL_SIZE_Y: f32 = 5.0;
const TILE_SIZE: f32 = 0.2;
const CLOCK_RADIUS: f32 = 0.4;
const CLOCK_MINUTE_HAND_LENGTH: f32 = CLOCK_RADIUS * 0.9;
const CLOCK_HOUR_HAND_LENGTH: f32 = CLOCK_RADIUS * 0.5;

#[derive(Component, Copy, Clone)]
enum ClockHand {
    Minute,
    Hour,
}

#[derive(Resource)]
struct StartupWallClockTime {
    time: chrono::DateTime<chrono::Local>,
}

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    // asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.insert_resource(Score { value: 0 });
    commands.insert_resource(AchievementQueue::default());

    // wall
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
        number_material: materials.add(Color::hex("#FFF0CE").unwrap().into()),
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

    // Clock
    commands
        .spawn(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cylinder {
                radius: CLOCK_RADIUS,
                height: 0.02,
                resolution: 64,
                ..Default::default()
            })),
            material: materials.add(Color::DARK_GRAY.into()),
            transform: Transform::from_xyz(-1.6, 0.9, 0.02)
                .with_rotation(Quat::from_rotation_x(FRAC_PI_2)),
            ..default()
        })
        .with_children(|builder| {
            let hand_material = materials.add(StandardMaterial {
                cull_mode: None,
                ..Color::GRAY.into()
            });
            builder.spawn(PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Cylinder {
                    radius: CLOCK_RADIUS * 1.1,
                    height: 0.01,
                    resolution: 64,
                    ..Default::default()
                })),
                material: hand_material.clone(),

                ..default()
            });

            // Minute hand
            builder
                .spawn(PbrBundle {
                    mesh: meshes.add(Mesh::from(shape::Plane::default())),
                    material: hand_material.clone(),
                    transform: clock_hand_transform(ClockHand::Minute),
                    ..default()
                })
                .insert(ClockHand::Minute);
            // Hour hand
            builder
                .spawn(PbrBundle {
                    mesh: meshes.add(Mesh::from(shape::Plane::default())),
                    material: hand_material.clone(),
                    transform: clock_hand_transform(ClockHand::Minute),
                    ..default()
                })
                .insert(ClockHand::Hour);
        });
    commands.insert_resource(StartupWallClockTime {
        time: chrono::Local::now(),
    });

    // light
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            color: Color::WHITE,
            illuminance: 0.0,
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

    // 2d camera
    let mut camera2d = Camera2dBundle {
        camera: Camera {
            order: 1,
            ..default()
        },
        camera_2d: Camera2d {
            clear_color: ClearColorConfig::None,
        },
        ..default()
    };
    // camera2d.projection.scale = 1.0;
    // camera2d.projection.scaling_mode = ScalingMode::FixedVertical(1.);
    commands.spawn(camera2d);
}

fn light_switch_update(
    mut score: ResMut<Score>,
    mouse_input: Res<Input<MouseButton>>,
    mut query_light: Query<&mut DirectionalLight>,
    mut query_switch: Query<&mut Transform, With<LightSwitch>>,
    mut achievement_queue: ResMut<AchievementQueue>,
) {
    if mouse_input.just_released(MouseButton::Middle) {
        for mut light in query_light.iter_mut() {
            if light.illuminance > 0.0 {
                light.illuminance = 0.0;
            } else {
                light.illuminance = 10000.0;
            }
            score.value += 1;

            if score.value == 1 {
                achievement_queue.queue.push_back(AchievementToBeAdded {
                    text: "Lights on".to_string(),
                });
            }

            if score.value == 100 {
                achievement_queue.queue.push_back(AchievementToBeAdded {
                    text: "But I wanted cookies...".to_string(),
                });
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
    mut achievement_queue: ResMut<AchievementQueue>,
) {
    let mut query_temperature = query_temperature.single_mut();

    for event in scroll_events.read() {
        if !achievement_queue.was_dimmer_used {
            achievement_queue.was_dimmer_used = true;
            achievement_queue.queue.push_back(AchievementToBeAdded {
                text: "So colorful *_*".to_string(),
            });
        }
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
    score: Res<Score>,
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

    let digit_patterns = include_str!("digits.txt")
        .chars()
        .filter_map(|c| match c {
            ' ' => Some(false),
            '\n' => None,
            '.' => Some(true),
            _ => unreachable!("Invalid digit pattern"),
        })
        .collect::<Vec<_>>();

    let score_str = score.value.to_string();

    for (tile, mut material) in wall_tiles.iter_mut() {
        *material = if is_digit_tile(tile, &score_str, &digit_patterns) {
            palette.number_material.clone()
        } else {
            palette.materials[pattern[pattern_index(tile)]].clone()
        };
    }
}

fn is_digit_tile(tile: &WallTile, digits: &str, digit_patterns: &[bool]) -> bool {
    const TOP_RIGHT_DIGIT_X: usize = (WALL_SIZE_X / TILE_SIZE / 2.0) as usize + 13;
    const TOP_RIGHT_DIGIT_Y: usize = (WALL_SIZE_Y / TILE_SIZE / 2.0) as usize + 7;
    const DIGIT_SIZE_X: usize = 3;
    const DIGIT_SIZE_Y: usize = 5;

    // Are we in the digit area?
    if tile.x > TOP_RIGHT_DIGIT_X
        || tile.y > TOP_RIGHT_DIGIT_Y
        || tile.y <= TOP_RIGHT_DIGIT_Y - DIGIT_SIZE_Y
    {
        return false;
    }

    // First determine in which digit we are
    let digit_idx = (TOP_RIGHT_DIGIT_X - tile.x) / (DIGIT_SIZE_X + 1);
    if digit_idx >= digits.len() {
        return false;
    }
    let digit_index = digits.len() - 1 - digit_idx;

    // Where inside this digit are we
    let digit_x = (TOP_RIGHT_DIGIT_X - tile.x) % (DIGIT_SIZE_X + 1);
    if digit_x == DIGIT_SIZE_X {
        return false; // We're in the space between digits!
    }
    let digit_x = DIGIT_SIZE_X - digit_x - 1;
    let digit_y = TOP_RIGHT_DIGIT_Y - tile.y;

    assert!(digit_x < DIGIT_SIZE_X);
    assert!(digit_y < DIGIT_SIZE_Y);

    let Some(current_char) = digits.chars().nth(digit_index) else {
        println!("Invalid digit idx: {}", digit_index);
        return false;
    };
    let Some(current_pattern_block) = current_char.to_digit(10) else {
        println!("Invalid digit index: {}", digit_index);
        return false;
    };
    digit_patterns[current_pattern_block as usize * (DIGIT_SIZE_X * DIGIT_SIZE_Y)
        + digit_x
        + digit_y * DIGIT_SIZE_X]
}

fn update_clock_hands(mut query: Query<(&mut Transform, &ClockHand)>) {
    for (mut transform, hand) in query.iter_mut() {
        *transform = clock_hand_transform(*hand);
    }
}

fn clock_achievement_check(
    start_time: Res<StartupWallClockTime>,
    mut achievement_queue: ResMut<AchievementQueue>,
) {
    if achievement_queue.time_flies_achieved {
        return;
    }

    use chrono::prelude::*;
    let local_time = Local::now();
    if local_time.minute() != start_time.time.minute() {
        achievement_queue.queue.push_back(AchievementToBeAdded {
            text: "Time flies when you're having fun".to_string(),
        });
        achievement_queue.time_flies_achieved = true;
    }
}

fn clock_hand_transform(hand: ClockHand) -> Transform {
    use chrono::prelude::*;
    let local_time = Local::now();

    match hand {
        ClockHand::Minute => {
            let minute_angle = local_time.minute() as f32 / 59.0 * (TAU * 59.0 / 60.0);
            Transform::from_rotation(Quat::from_rotation_y(-minute_angle))
                * Transform::from_scale(vec3(0.03, 1.0, CLOCK_MINUTE_HAND_LENGTH))
                    .with_translation(vec3(0.0, 0.03, -CLOCK_MINUTE_HAND_LENGTH / 2.0 + 0.015))
        }
        ClockHand::Hour => {
            let hour_angle = local_time.hour() as f32 / 11.0 * (TAU * 11.0 / 12.0);
            Transform::from_rotation(Quat::from_rotation_y(-hour_angle))
                * Transform::from_scale(vec3(0.03, 1.0, CLOCK_HOUR_HAND_LENGTH))
                    .with_translation(vec3(0.0, 0.04, -CLOCK_HOUR_HAND_LENGTH / 2.0 + 0.015))
        }
    }
}
