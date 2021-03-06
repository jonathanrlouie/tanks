use bevy::{
    diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin},
    prelude::*,
    render::{
        camera::Camera,
        view::Visibility,
    },
    sprite::collide_aabb::{collide, Collision},
};

const SHOW_FPS: bool = true;

fn main() {
    App::new()
        .insert_resource(CursorPosition { pos: Vec2::ZERO })
        .insert_resource(CurrentLevel(Some(Level::L1)))
        .add_plugins(DefaultPlugins)
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_state(GameState::Playing)
        .add_startup_system(setup_cameras)
        .add_startup_system(setup_text)
        .add_system_set(SystemSet::on_enter(GameState::Playing).with_system(setup))
        .add_system_set(
            SystemSet::on_update(GameState::Playing)
                .with_system(player_movement_system)
                .with_system(player_shoot_system)
                .with_system(bullet_movement_system)
                .with_system(cursor_position_system)
                .with_system(bullet_cleanup_system)
                .with_system(bullet_collision_system)
                .with_system(brown_tank_shoot_system)
                .with_system(playing_system),
        )
        .add_system_set(
            SystemSet::on_enter(GameState::Lose).with_system(lose_setup_system),
        )
        .add_system_set(SystemSet::on_update(GameState::Lose).with_system(lose_system))
        .add_system_set(SystemSet::on_exit(GameState::Lose).with_system(teardown_system))
        .add_system_set(SystemSet::on_enter(GameState::Win).with_system(win_setup_system))
        .add_system_set(SystemSet::on_update(GameState::Win).with_system(win_system))
        .add_system_set(
            SystemSet::on_exit(GameState::Win)
                .with_system(blank_text_system)
                .with_system(next_level_system)
                .with_system(teardown_system),
        )
        .add_system(text_update_system)
        .run()
}

#[derive(Component)]
struct UiElement;
#[derive(Component)]
struct FpsText;
#[derive(Component)]
struct WinText;

enum Level {
    L1,
    L2,
}

struct CurrentLevel(Option<Level>);

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
enum GameState {
    Win,
    Lose,
    Playing,
}

struct CursorPosition {
    pos: Vec2,
}

#[derive(Component)]
struct GameTimer(Timer);

#[derive(Component)]
enum Collider {
    Wall,
    Player,
    Bullet,
    Enemy,
}

#[derive(Component)]
struct Player {
    speed: f32,
}

#[derive(Component)]
struct Bullet {
    velocity: Vec3,
}

#[derive(Component)]
struct Hitbox(Vec2);

#[derive(Component)]
struct RicochetLimit(u32);

#[derive(Component)]
struct RicochetCount(u32);

#[derive(Component)]
struct BulletOwner(Entity);

#[derive(Component)]
struct BulletLimit(u8);

#[derive(Component)]
struct Enemy;

#[derive(Component)]
struct BrownTank;

// Camera system
fn setup_cameras(mut commands: Commands) {
    // game camera
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());

    // UI camera needed to render text
    commands.spawn_bundle(UiCameraBundle::default());
}

// Text systems
fn setup_text(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Row,
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                ..Default::default()
            },
            visibility: Visibility { is_visible: false },
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
		    visibility: Visibility { is_visible: false },
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
		    visibility: Visibility { is_visible: false },
                    ..Default::default()
                })
                .insert(UiElement);
        });
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

// Initial setup system
fn setup(
    commands: Commands,
    asset_server: Res<AssetServer>,
    current_level: Res<CurrentLevel>,
) {
    if let Some(level) = &current_level.0 {
        match level {
            Level::L1 => setup_level1(commands, asset_server),
            Level::L2 => setup_level2(commands, asset_server),
        }
    }
}

// Creator for "prefabs"
struct Creator<'a> {
    commands: Commands<'a, 'a>,
    asset_server: Res<'a, AssetServer>,
}

impl<'a> Creator<'a> {
    fn create_player(&mut self, x: f32, y: f32) {
        let texture_handle = self.asset_server.load("player.png");
        self.commands
            .spawn_bundle(SpriteBundle {
                texture: texture_handle,
                transform: Transform::from_xyz(x, y, 0.0),
                ..Default::default()
            })
            .insert(Player { speed: 100.0 })
            .insert(BulletLimit(5))
            .insert(Hitbox(Vec2::new(32.0, 32.0)))
            .insert(Collider::Player);
    }

    fn create_wall(&mut self, x: f32, y: f32) {
        let texture_handle = self.asset_server.load("wall.png");
        self.commands
            .spawn_bundle(SpriteBundle {
                texture: texture_handle,
                transform: Transform::from_xyz(x, y, 0.0),
                ..Default::default()
            })
            .insert(Hitbox(Vec2::new(32.0, 32.0)))
            .insert(Collider::Wall);
    }

    fn create_brown_tank(&mut self, x: f32, y: f32) {
        let texture_handle = self.asset_server.load("enemy_brown.png");
        self.commands
            .spawn_bundle(SpriteBundle {
                texture: texture_handle,
                transform: Transform::from_xyz(x, y, 0.0),
                ..Default::default()
            })
            .insert(BulletLimit(1))
            .insert(BrownTank)
            .insert(Enemy)
            .insert(Hitbox(Vec2::new(32.0, 32.0)))
            .insert(Collider::Enemy);
    }
}

// Level 1 setup system
fn setup_level1(
    commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let mut creator = Creator {
        commands,
        asset_server,
    };

    // player
    creator.create_player(0.0, 0.0);

    // create walls
    creator.create_wall(32.0, 64.0);
    creator.create_wall(-32.0, 64.0);
    creator.create_wall(32.0, -64.0);

    // create enemies
    creator.create_brown_tank(-130.0, 150.0);
}

// Level 2 setup system
fn setup_level2(
    commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let mut creator = Creator {
        commands,
        asset_server,
    };

    // player
    creator.create_player(0.0, 0.0);

    // create walls
    creator.create_wall(32.0, 64.0);
    creator.create_wall(-32.0, 64.0);
    creator.create_wall(32.0, -64.0);
    creator.create_wall(-32.0, -64.0);
    creator.create_wall(-80.0, -80.0);

    // create enemies
    creator.create_brown_tank(-100.0, 100.0);
    creator.create_brown_tank(100.0, 150.0);
}

// Main game systems
#[allow(clippy::type_complexity)]
fn player_movement_system(
    time: Res<Time>,
    keyboard_input: Res<Input<KeyCode>>,
    mut queries: QuerySet<(
        QueryState<(&Transform, &Hitbox), With<Player>>,
        QueryState<(&Collider, &Transform, &Hitbox)>,
        QueryState<(&Player, &mut Transform)>,
    )>,
) {
    let mut collisions: Vec<Collision> = vec![];
    if let Ok((player_transform, player_hitbox)) = queries.q0().get_single() {
        let player_transform = player_transform.clone();
        let player_hitbox = player_hitbox.0.clone();
        for (collider, transform, hitbox) in queries.q1().iter() {
            let collision = collide(
                player_transform.translation,
                player_hitbox,
                transform.translation,
                hitbox.0,
            );

            // Stop player movement on collision with walls or enemies
            if let Some(collision) = collision {
                match *collider {
                    Collider::Wall | Collider::Enemy => collisions.push(collision),
                    _ => (),
                }
            }
        }
    }

    if let Ok((player, mut player_transform)) = queries.q2().get_single_mut() {
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

        // Normalize the direction so player doesn't move faster on diagonals
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

// helper function to get cursor position
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
    // Delete bullets if they are outside of the window
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
    query: Query<(Entity, &BulletLimit, &Transform), With<Player>>,
    bullet_query: Query<&BulletOwner, With<Bullet>>,
    cursor_position: Res<CursorPosition>,
) {
    if let Ok((player_entity, bullet_limit, player_transform)) = query.get_single() {
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
                    texture: texture_handle,
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
                    .insert(Hitbox(Vec2::new(8.0, 8.0)))
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
        &Hitbox,
    )>,
    collider_query: Query<(Entity, &Collider, &Transform, &Hitbox)>,
) {
    for (
        bullet_entity,
        mut bullet,
        bullet_owner,
        ricochet_limit,
        mut ricochet_count,
        bullet_transform,
        bullet_hitbox,
    ) in bullet_query.iter_mut()
    {
        let velocity = &mut bullet.velocity;
        for (collider_entity, collider, transform, hitbox) in collider_query.iter() {
            let collision = collide(
                bullet_transform.translation,
                bullet_hitbox.0,
                transform.translation,
                hitbox.0,
            );

            if let Some(collision) = collision {
                match *collider {
                    // Bullets destroy each other on contact
                    Collider::Bullet => commands.entity(collider_entity).despawn(),
                    Collider::Enemy | Collider::Player => {
                        // Make sure freshly fired bullets do not kill the tank that fired it
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

                        // Destroy bullets if they have reached their ricochet limit
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
    brown_tank_query: Query<(Entity, &BulletLimit, &Transform), With<BrownTank>>,
    player_query: Query<&Transform, With<Player>>,
    bullet_query: Query<&BulletOwner, With<Bullet>>,
) {
    for (tank_entity, bullet_limit, tank_transform) in brown_tank_query.iter() {
        if let Ok(player_transform) = player_query.get_single() {
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
                        texture: texture_handle,
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
                        .insert(Hitbox(Vec2::new(8.0, 8.0)))
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
    // If there are no more enemies set state to Win, otherwise, if the player was destroyed, set
    // state to lose.
    // In the unlikely event of a tie, the player win takes precedence for a less frustrating
    // experience :)
    if enemy_query.iter().count() == 0 {
        game_state
            .set(GameState::Win)
            .expect("Error: Failed to push Win state");
    } else if player_query.get_single().is_err() {
        game_state
            .set(GameState::Lose)
            .expect("Error: Failed to push Lose state")
    }
}

// Teardown system
// Clean-up all entities, excluding camera and UI elements
fn teardown_system(
    mut commands: Commands,
    entities: Query<Entity, (Without<Camera>, Without<UiElement>)>,
) {
    for entity in entities.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

// Lose state systems
fn lose_setup_system(mut commands: Commands) {
    // Start a timer
    commands
        .spawn()
        .insert(GameTimer(Timer::from_seconds(4.0, false)));
}

fn lose_system(
    time: Res<Time>,
    mut query: Query<&mut GameTimer>,
    mut game_state: ResMut<State<GameState>>,
) {
    // Reset current level if timer reaches 0
    if let Ok(mut timer) = query.get_single_mut() {
        if timer.0.tick(time.delta()).just_finished() {
            game_state
                .set(GameState::Playing)
                .expect("Error: Failed to set Playing state");
        }
    }
}

// Win state systems
fn win_setup_system(mut commands: Commands, mut query: Query<&mut Text, With<WinText>>) {
    // Start a timer
    commands
        .spawn()
        .insert(GameTimer(Timer::from_seconds(4.0, false)));

    if let Ok(mut text) = query.get_single_mut() {
        text.sections[0].style.color = Color::WHITE;
    }
}

fn win_system(
    time: Res<Time>,
    mut query: Query<&mut GameTimer>,
    mut game_state: ResMut<State<GameState>>,
) {
    // Set state to Playing again after timer reaches 0
    if let Ok(mut timer) = query.get_single_mut() {
        if timer.0.tick(time.delta()).just_finished() {
            game_state
                .set(GameState::Playing)
                .expect("Error: Failed to set Playing state");
        }
    }
}

fn blank_text_system(mut query: Query<&mut Text, With<WinText>>) {
    // Clear the "Mission complete!" text by setting color to NONE
    if let Ok(mut text) = query.get_single_mut() {
        text.sections[0].style.color = Color::NONE;
    }
}

fn next_level_system(mut current_level: ResMut<CurrentLevel>) {
    // Set next level to go to
    if let Some(level) = &current_level.0 {
        current_level.0 = match level {
            Level::L1 => Some(Level::L2),
            Level::L2 => None,
        }
    }
}
