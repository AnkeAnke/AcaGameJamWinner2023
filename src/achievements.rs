use bevy::{
    math::vec3,
    prelude::*,
    sprite::Anchor,
    text::{BreakLineOn, Text2dBounds},
    utils::Instant,
};
use bevy_hanabi::prelude::*;
use std::collections::VecDeque;

const ACHIEVEMENT_CARD_HEIGHT: f32 = 100.0;

#[derive(Resource)]
pub struct AchievementStyle {
    pub text_style: TextStyle,
    pub particle_style: Handle<EffectAsset>,
    pub sound: Handle<AudioSource>,
}

#[derive(Component)]
pub struct Achievement {
    spawn_time: Instant,
    index: usize,
}

// #[derive(Event)]
pub struct AchievementToBeAdded {
    pub text: String,
}

#[derive(Resource, Default)]
pub struct AchievementQueue {
    pub queue: VecDeque<AchievementToBeAdded>,
    pub num_achieved_achievements: usize,
    pub was_dimmer_used: bool,
    pub was_achievement_achieved: bool,
    pub time_flies_achieved: bool,
}

pub fn setup_achievements(
    mut commands: Commands,
    mut effects: ResMut<Assets<EffectAsset>>,
    asset_server: Res<AssetServer>,
) {
    let mut gradient = Gradient::new();
    gradient.add_key(0.0, Vec4::new(1.0, 0.9, 1.0, 1.0));
    gradient.add_key(1.0, Vec4::new(0.5, 0.5, 1.0, 0.0));

    let writer = ExprWriter::new();

    let age = writer.lit(0.).expr();
    let init_age = SetAttributeModifier::new(Attribute::AGE, age);

    let lifetime = writer.lit(5.).expr();
    let init_lifetime = SetAttributeModifier::new(Attribute::LIFETIME, lifetime);

    let init_pos = SetPositionCircleModifier {
        center: writer.lit(Vec3::ZERO).expr(),
        axis: writer.lit(Vec3::Z).expr(),
        radius: writer.lit(ACHIEVEMENT_CARD_HEIGHT * 0.5).expr(),
        dimension: ShapeDimension::Surface,
    };

    let init_vel = SetVelocityCircleModifier {
        center: writer.lit(Vec3::ZERO).expr(),
        axis: writer.lit(Vec3::Z).expr(),
        speed: writer.lit(20.0).expr(),
    };

    let spawner = Spawner::once(30.0.into(), true);
    let effect = effects.add(
        EffectAsset::new(4096, spawner, writer.finish())
            .with_simulation_space(SimulationSpace::Local)
            .with_name("2d")
            .init(init_pos)
            .init(init_vel)
            .init(init_age)
            .init(init_lifetime)
            .render(SizeOverLifetimeModifier {
                gradient: Gradient::constant(Vec2::splat(10.0)),
                screen_space_size: false,
            })
            .render(ColorOverLifetimeModifier { gradient }),
    );

    commands.insert_resource(AchievementStyle {
        text_style: TextStyle {
            font: asset_server.load("embedded://aca_gamejam_winner2023/PublicPixel-z84yD.ttf"),
            font_size: 20.0,
            color: Color::hex("#FFF0CE").unwrap(),
        },
        sound: asset_server.load("embedded://aca_gamejam_winner2023/achievement.ogg"),
        particle_style: effect,
    });
}

pub fn achievement_update(
    mut commands: Commands,
    achievement_style: Res<AchievementStyle>,
    query_ortho: Query<&OrthographicProjection>,
    mut achievement_queue: ResMut<AchievementQueue>,
    mut achievements: Query<(&mut Transform, &Achievement, Entity)>,
) {
    let mut shortest_lifetime = None;
    for (_, achievement, entity) in achievements.iter_mut() {
        let age = achievement.spawn_time.elapsed().as_secs_f32();
        if age > 5.0 {
            commands.entity(entity).despawn_recursive();
        }
        if achievement.index == achievement_queue.num_achieved_achievements {
            shortest_lifetime = Some(age);
        }
    }
    let ortho = query_ortho.single();

    let lowest_stack_position = shortest_lifetime.map_or(0.0, |t| ((t - 1.0) / 1.0).min(0.0));
    for (mut transform, achievement, _) in achievements.iter_mut() {
        let stack_position = lowest_stack_position
            + achievement_queue.num_achieved_achievements as f32
            - achievement.index as f32;
        transform.translation = achievement_position(ortho.area, stack_position);
    }

    if lowest_stack_position >= 0.0 {
        if let Some(event) = achievement_queue.queue.pop_front() {
            if !achievement_queue.was_achievement_achieved {
                achievement_queue.was_achievement_achieved = true;
                achievement_queue.queue.push_back(AchievementToBeAdded {
                    text: "Got it!".to_string(),
                });
            }

            achievement_queue.num_achieved_achievements += 1;
            spawn_achievement(
                &mut commands,
                achievement_style.as_ref(),
                ortho.area,
                achievement_queue.num_achieved_achievements,
                &event.text,
            );
        }
    }
}

fn achievement_position(screen_area: Rect, stack_position: f32) -> Vec3 {
    vec3(
        screen_area.max.x,
        screen_area.min.y + stack_position * ACHIEVEMENT_CARD_HEIGHT,
        0.0,
    )
}

fn spawn_achievement(
    commands: &mut Commands,
    achievement_style: &AchievementStyle,
    screen_area: Rect,
    achievement_index: usize,
    text: &str,
) {
    let box_size = Vec2::new(300.0, ACHIEVEMENT_CARD_HEIGHT);
    commands
        .spawn(SpriteBundle {
            sprite: Sprite {
                color: Color::hex("#232D3F").unwrap(),
                custom_size: Some(Vec2::new(box_size.x, box_size.y)),
                anchor: Anchor::BottomRight,
                ..default()
            },
            transform: Transform::from_translation(achievement_position(screen_area, -1.0)),
            ..default()
        })
        .with_children(|builder| {
            builder.spawn(Text2dBundle {
                text: Text {
                    sections: vec![TextSection::new(
                        text.to_string(),
                        achievement_style.text_style.clone(),
                    )],
                    alignment: TextAlignment::Left,
                    linebreak_behavior: BreakLineOn::WordBoundary,
                },
                text_2d_bounds: Text2dBounds {
                    // Wrap text in the rectangle
                    size: box_size,
                },
                // ensure the text is drawn on top of the box
                transform: Transform::from_xyz(-box_size.x * 0.5, box_size.y * 0.5, 1.0),
                ..default()
            });
            builder
                .spawn(ParticleEffectBundle {
                    effect: ParticleEffect::new(achievement_style.particle_style.clone()),
                    transform: Transform::from_xyz(-box_size.x * 0.5, box_size.y * 0.5, 0.9),
                    ..default()
                })
                .insert(Name::new("effect:2d"));
        })
        .insert(Achievement {
            spawn_time: Instant::now(),
            index: achievement_index,
        });

    commands.spawn(AudioBundle {
        source: achievement_style.sound.clone(),
        settings: PlaybackSettings::DESPAWN,
    });
}
