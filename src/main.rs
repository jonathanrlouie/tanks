use bevy::{
    diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin},
    prelude::*,
    render::camera::Camera,
    sprite::collide_aabb::{collide, Collision},
};

const SHOW_FPS: bool = true;

fn main() {
    App::build()
        .insert_resource(CursorPosition { pos: Vec2::ZERO })
        .add_plugins(DefaultPlugins)
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_state(GameState::Playing)
        .add_startup_system(setup_cameras.system())
        .add_startup_system(setup_text.system())
        .add_system_set(SystemSet::on_enter(GameState::Playing).with_system(setup.system()))
        .add_system_set(
            SystemSet::on_update(GameState::Playing)
                .with_system(player_movement_system.system())
                .with_system(player_shoot_system.system())
                .with_system(bullet_movement_system.system())
                .with_system(cursor_position_system.system())
                .with_system(bullet_cleanup_system.system())
                .with_system(bullet_collision_system.system())
                .with_system(brown_tank_shoot_system.system())
                .with_system(playing_system.system()),
        )
        .add_system_set(
            SystemSet::on_enter(GameState::Lose).with_system(lose_setup_system.system()),
        )
        .add_system_set(SystemSet::on_update(GameState::Lose).with_system(lose_system.system()))
        .add_system_set(SystemSet::on_exit(GameState::Lose).with_system(teardown_system.system()))
        .add_system_set(SystemSet::on_enter(GameState::Win).with_system(win_setup_system.system()))
        .add_system_set(SystemSet::on_update(GameState::Win).with_system(win_system.system()))
        .add_system_set(
            SystemSet::on_exit(GameState::Win)
                .with_system(blank_text_system.system())
                .with_system(teardown_system.system()),
        )
        .add_system(text_update_system.system())
        .run()
}

struct UiElement;
struct FpsText;
struct WinText;

enum Level {
    L1,
    L2,
}

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
enum GameState {
    Win,
    Lose,
    Playing,
}

struct CursorPosition {
    pos: Vec2,
}

struct GameTimer(Timer);

enum Collider {
    Wall,
    Player,
    Bullet,
    Enemy,
}

struct Player {
    speed: f32,
}

struct Bullet {
    velocity: Vec3,
}

struct RicochetLimit(u32);

struct RicochetCount(u32);

struct BulletOwner(Entity);

struct BulletLimit(u8);

struct Enemy;

struct BrownTank;

fn setup_cameras(mut commands: Commands) {
    // game camera
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());

    // UI camera needed to render text
    commands.spawn_bundle(UiCameraBundle::default());
}

fn setup_text(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Row,
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                ..Default::default()
            },
            material: materials.add(Color::NONE.into()),
            ..Default::default()
        })
        .insert(UiElement)
        .with_children(|parent| {
            // FPS text
            parent
                .spawn_bundle(TextBundle {
                    style: Style {
                        size: Size::new(Val::Percent(15.0), Val::Percent(100.0)),
                        ..Default::default()
                    },
                    text: Text {
                        sections: vec![
                            TextSection {
                                value: "FPS: ".to_string(),
                                style: TextStyle {
                                    font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                                    font_size: 60.0,
                                    color: if SHOW_FPS { Color::WHITE } else { Color::NONE },
                                },
                            },
                            TextSection {
                                value: "".to_string(),
                                style: TextStyle {
                                    font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                                    font_size: 60.0,
                                    color: if SHOW_FPS { Color::WHITE } else { Color::NONE },
                                },
                            },
                        ],
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .insert(UiElement)
                .insert(FpsText);

            parent
                .spawn_bundle(NodeBundle {
                    style: Style {
                        size: Size::new(Val::Percent(70.0), Val::Percent(100.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..Default::default()
                    },
                    material: materials.add(Color::NONE.into()),
                    ..Default::default()
                })
                .insert(UiElement)
                .with_children(|p2| {
                    // Win text
                    p2.spawn_bundle(TextBundle {
                        text: Text {
                            sections: vec![TextSection {
                                value: "Mission complete!".to_string(),
                                style: TextStyle {
                                    font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                                    font_size: 60.0,
                                    color: Color::NONE,
                                },
                            }],
                            ..Default::default()
                        },
                        ..Default::default()
                    })
                    .insert(WinText)
                    .insert(UiElement);
                });

            // Empty node to evenly split the UI
            parent
                .spawn_bundle(NodeBundle {
                    style: Style {
                        size: Size::new(Val::Percent(15.0), Val::Percent(100.0)),
                        ..Default::default()
                    },
                    material: materials.add(Color::NONE.into()),
                    ..Default::default()
                })
                .insert(UiElement);
        });
}

fn setup(
    commands: Commands,
    asset_server: Res<AssetServer>,
    materials: ResMut<Assets<ColorMaterial>>,
) {
    let mut creator = Creator {
        commands,
        asset_server,
        materials,
    };

    // player
    creator.create_player(0.0, 0.0);

    // create walls
    creator.create_wall(32.0, 64.0);
    creator.create_wall(-32.0, 64.0);
    creator.create_wall(32.0, -64.0);

    // create enemies
    creator.create_brown_tank(-100.0, 100.0);
    creator.create_brown_tank(200.0, 150.0);
}

struct Creator<'a> {
    commands: Commands<'a>,
    asset_server: Res<'a, AssetServer>,
    materials: ResMut<'a, Assets<ColorMaterial>>,
}

impl<'a> Creator<'a> {
    fn create_player(&mut self, x: f32, y: f32) {
        let texture_handle = self.asset_server.load("player.png");
        self.commands
            .spawn_bundle(SpriteBundle {
                material: self.materials.add(texture_handle.into()),
                transform: Transform::from_xyz(x, y, 0.0),
                ..Default::default()
            })
            .insert(Player { speed: 100.0 })
            .insert(BulletLimit(5))
            .insert(Collider::Player);
    }

    fn create_wall(&mut self, x: f32, y: f32) {
        let texture_handle = self.asset_server.load("wall.png");
        self.commands
            .spawn_bundle(SpriteBundle {
                material: self.materials.add(texture_handle.into()),
                transform: Transform::from_xyz(x, y, 0.0),
                ..Default::default()
            })
            .insert(Collider::Wall);
    }

    fn create_brown_tank(&mut self, x: f32, y: f32) {
        let texture_handle = self.asset_server.load("enemy_brown.png");
        self.commands
            .spawn_bundle(SpriteBundle {
                material: self.materials.add(texture_handle.into()),
                transform: Transform::from_xyz(x, y, 0.0),
                ..Default::default()
            })
            .insert(BulletLimit(1))
            .insert(BrownTank)
            .insert(Enemy)
            .insert(Collider::Enemy);
    }
}

#[allow(clippy::type_complexity)]
fn player_movement_system(
    time: Res<Time>,
    keyboard_input: Res<Input<KeyCode>>,
    mut queries: QuerySet<(
        Query<(&Transform, &Sprite), With<Player>>,
        Query<(&Collider, &Transform, &Sprite)>,
        Query<(&Player, &mut Transform)>,
    )>,
) {
    let mut collisions: Vec<Collision> = vec![];
    if let Ok((player_transform, player_sprite)) = queries.q0().single() {
        for (collider, transform, sprite) in queries.q1().iter() {
            let collision = collide(
                player_transform.translation,
                player_sprite.size,
                transform.translation,
                sprite.size,
            );

            if let Some(collision) = collision {
                match *collider {
                    Collider::Wall | Collider::Enemy => collisions.push(collision),
                    _ => (),
                }
            }
        }
    }

    if let Ok((player, mut player_transform)) = queries.q2_mut().single_mut() {
        let mut direction: Vec2 = Vec2::ZERO;
        if keyboard_input.pressed(KeyCode::A) {
            direction -= Vec2::X;
        }

        if keyboard_input.pressed(KeyCode::D) {
            direction += Vec2::X;
        }

        if keyboard_input.pressed(KeyCode::W) {
            direction += Vec2::Y;
        }

        if keyboard_input.pressed(KeyCode::S) {
            direction -= Vec2::Y;
        }

        let normalized_direction = direction.try_normalize().unwrap_or(Vec2::ZERO);
        let translation = &mut player_transform.translation;

        let mut stop_x = false;
        let mut stop_y = false;

        for collision in collisions {
            match collision {
                Collision::Left => stop_x = normalized_direction.x > 0.0,
                Collision::Right => stop_x = normalized_direction.x < 0.0,
                Collision::Top => stop_y = normalized_direction.y < 0.0,
                Collision::Bottom => stop_y = normalized_direction.y > 0.0,
            }
        }

        if !stop_x {
            translation.x += time.delta_seconds() * normalized_direction.x * player.speed;
        }

        if !stop_y {
            translation.y += time.delta_seconds() * normalized_direction.y * player.speed;
        }
    }
}

fn cursor_position_system(windows: Res<Windows>, mut cursor_position: ResMut<CursorPosition>) {
    if let Some(cursor_pos) = calculate_cursor_position(windows) {
        cursor_position.pos = cursor_pos;
    }
}

fn calculate_cursor_position(windows: Res<Windows>) -> Option<Vec2> {
    let window = windows.get_primary()?;
    let cursor_position = window.cursor_position()?;
    Some(Vec2::new(
        cursor_position.x - window.width() / 2.0,
        cursor_position.y - window.height() / 2.0,
    ))
}

fn bullet_cleanup_system(
    mut commands: Commands,
    windows: Res<Windows>,
    query: Query<(Entity, &Transform), With<Bullet>>,
) {
    if let Some(window) = windows.get_primary() {
        for (entity, transform) in query.iter() {
            if transform.translation.x > window.width() / 2.0
                || transform.translation.x < 0.0 - window.width() / 2.0
                || transform.translation.y > window.height() / 2.0
                || transform.translation.y < 0.0 - window.height() / 2.0
            {
                commands.entity(entity).despawn();
            }
        }
    }
}

fn player_shoot_system(
    mut commands: Commands,
    mouse_input: Res<Input<MouseButton>>,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    query: Query<(Entity, &BulletLimit, &Transform), With<Player>>,
    bullet_query: Query<&BulletOwner, With<Bullet>>,
    cursor_position: Res<CursorPosition>,
) {
    if let Ok((player_entity, bullet_limit, player_transform)) = query.single() {
        if mouse_input.just_pressed(MouseButton::Left)
            && bullet_query
                .iter()
                .filter(|owner| owner.0 == player_entity)
                .count()
                < bullet_limit.0.into()
        {
            if let Some(bullet_direction) = Vec3::new(
                cursor_position.pos.x - player_transform.translation.x,
                cursor_position.pos.y - player_transform.translation.y,
                0.0,
            )
            .try_normalize()
            {
                let texture_handle = asset_server.load("bullet.png");
                let bullet_transform = Transform::from_xyz(
                    player_transform.translation.x,
                    player_transform.translation.y,
                    0.0,
                );
                let sprite_bundle = SpriteBundle {
                    material: materials.add(texture_handle.into()),
                    transform: bullet_transform,
                    ..Default::default()
                };

                // bullet
                commands
                    .spawn_bundle(sprite_bundle)
                    .insert(Bullet {
                        velocity: 150.0 * bullet_direction,
                    })
                    .insert(BulletOwner(player_entity))
                    .insert(RicochetLimit(1))
                    .insert(RicochetCount(0))
                    .insert(Collider::Bullet);
            }
        }
    }
}

fn bullet_movement_system(time: Res<Time>, mut query: Query<(&Bullet, &mut Transform)>) {
    for (bullet, mut transform) in query.iter_mut() {
        transform.translation += time.delta_seconds() * bullet.velocity;
    }
}

fn bullet_collision_system(
    mut commands: Commands,
    mut bullet_query: Query<(
        Entity,
        &mut Bullet,
        &BulletOwner,
        &RicochetLimit,
        &mut RicochetCount,
        &Transform,
        &Sprite,
    )>,
    collider_query: Query<(Entity, &Collider, &Transform, &Sprite)>,
) {
    for (
        bullet_entity,
        mut bullet,
        bullet_owner,
        ricochet_limit,
        mut ricochet_count,
        bullet_transform,
        bullet_sprite,
    ) in bullet_query.iter_mut()
    {
        let velocity = &mut bullet.velocity;
        for (collider_entity, collider, transform, sprite) in collider_query.iter() {
            let collision = collide(
                bullet_transform.translation,
                bullet_sprite.size,
                transform.translation,
                sprite.size,
            );

            if let Some(collision) = collision {
                match *collider {
                    Collider::Bullet => commands.entity(collider_entity).despawn(),
                    Collider::Enemy | Collider::Player => {
                        if !(bullet_owner.0 == collider_entity && ricochet_count.0 < 1) {
                            commands.entity(bullet_entity).despawn();
                            commands.entity(collider_entity).despawn()
                        }
                    }
                    Collider::Wall => {
                        let mut reflect_x = false;
                        let mut reflect_y = false;

                        match collision {
                            Collision::Left => reflect_x = velocity.x > 0.0,
                            Collision::Right => reflect_x = velocity.x < 0.0,
                            Collision::Top => reflect_y = velocity.y < 0.0,
                            Collision::Bottom => reflect_y = velocity.y > 0.0,
                        }

                        if ricochet_count.0 >= ricochet_limit.0 && (reflect_x || reflect_y) {
                            commands.entity(bullet_entity).despawn();
                        } else {
                            if reflect_x {
                                velocity.x = -velocity.x;
                            }

                            if reflect_y {
                                velocity.y = -velocity.y;
                            }

                            if reflect_x || reflect_y {
                                ricochet_count.0 += 1;
                            }
                        }
                    }
                }
            }
        }
    }
}

fn brown_tank_shoot_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    brown_tank_query: Query<(Entity, &BulletLimit, &Transform), With<BrownTank>>,
    player_query: Query<&Transform, With<Player>>,
    bullet_query: Query<&BulletOwner, With<Bullet>>,
) {
    for (tank_entity, bullet_limit, tank_transform) in brown_tank_query.iter() {
        if let Ok(player_transform) = player_query.single() {
            if bullet_query
                .iter()
                .filter(|owner| owner.0 == tank_entity)
                .count()
                < bullet_limit.0.into()
            {
                if let Some(bullet_direction) = Vec3::new(
                    player_transform.translation.x - tank_transform.translation.x,
                    player_transform.translation.y - tank_transform.translation.y,
                    0.0,
                )
                .try_normalize()
                {
                    let texture_handle = asset_server.load("bullet.png");
                    let bullet_transform = Transform::from_xyz(
                        tank_transform.translation.x,
                        tank_transform.translation.y,
                        0.0,
                    );
                    let sprite_bundle = SpriteBundle {
                        material: materials.add(texture_handle.into()),
                        transform: bullet_transform,
                        ..Default::default()
                    };

                    commands
                        .spawn_bundle(sprite_bundle)
                        .insert(Bullet {
                            velocity: 150.0 * bullet_direction,
                        })
                        .insert(BulletOwner(tank_entity))
                        .insert(RicochetLimit(1))
                        .insert(RicochetCount(0))
                        .insert(Collider::Bullet);
                }
            }
        }
    }
}

fn playing_system(
    player_query: Query<&Player>,
    enemy_query: Query<&Enemy>,
    mut game_state: ResMut<State<GameState>>,
) {
    if enemy_query.iter().count() == 0 {
        game_state
            .set(GameState::Win)
            .expect("Error: Failed to push Win state");
    } else if player_query.single().is_err() {
        game_state
            .set(GameState::Lose)
            .expect("Error: Failed to push Lose state")
    }
}

fn teardown_system(
    mut commands: Commands,
    entities: Query<Entity, (Without<Camera>, Without<UiElement>)>,
) {
    for entity in entities.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

fn lose_setup_system(mut commands: Commands) {
    commands
        .spawn()
        .insert(GameTimer(Timer::from_seconds(4.0, false)));
}

fn lose_system(
    time: Res<Time>,
    mut query: Query<&mut GameTimer>,
    mut game_state: ResMut<State<GameState>>,
) {
    if let Ok(mut timer) = query.single_mut() {
        if timer.0.tick(time.delta()).just_finished() {
            game_state
                .set(GameState::Playing)
                .expect("Error: Failed to set Playing state");
        }
    }
}

fn win_setup_system(mut commands: Commands, mut query: Query<&mut Text, With<WinText>>) {
    commands
        .spawn()
        .insert(GameTimer(Timer::from_seconds(4.0, false)));

    if let Ok(mut text) = query.single_mut() {
        text.sections[0].style.color = Color::WHITE;
    }
}

fn win_system(
    time: Res<Time>,
    mut query: Query<&mut GameTimer>,
    mut game_state: ResMut<State<GameState>>,
) {
    if let Ok(mut timer) = query.single_mut() {
        if timer.0.tick(time.delta()).just_finished() {
            game_state
                .set(GameState::Playing)
                .expect("Error: Failed to set Playing state");
        }
    }
}

fn text_update_system(diagnostics: Res<Diagnostics>, mut query: Query<&mut Text, With<FpsText>>) {
    for mut text in query.iter_mut() {
        if let Some(fps) = diagnostics.get(FrameTimeDiagnosticsPlugin::FPS) {
            if let Some(average) = fps.average() {
                text.sections[1].value = format!("{:.2}", average);
            }
        }
    }
}

fn blank_text_system(mut query: Query<&mut Text, With<WinText>>) {
    if let Ok(mut text) = query.single_mut() {
        text.sections[0].style.color = Color::NONE;
    }
}
