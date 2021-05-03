use bevy::{
    prelude::*,
    sprite::collide_aabb::{collide, Collision},
};

fn main() {
    App::build()
        .insert_resource(CursorPosition { pos: Vec2::ZERO })
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup.system())
        .add_system(player_movement_system.system())
        .add_system(player_shoot_system.system())
        .add_system(bullet_movement_system.system())
        .add_system(cursor_position_system.system())
        .add_system(bullet_cleanup_system.system())
        .add_system(bullet_collision_system.system())
        .add_system(brown_tank_shoot_system.system())
        .add_system(win_or_lose_system.system())
        .run()
}

struct CursorPosition {
    pos: Vec2
}

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

struct RicochetLimit {
    limit: u32,
}

struct RicochetCount {
    count: u32,
}

struct BulletOwner {
    owner: Entity
}

struct BulletLimit {
    limit: u8,
}

struct Enemy;

struct BrownTank;

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    materials: ResMut<Assets<ColorMaterial>>,
) {
    // camera
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());

    let mut creator = Creator {
        commands,
        asset_server,
        materials
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
            .insert(BulletLimit { limit: 5 })
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
            .insert(BulletLimit { limit: 1 })
            .insert(BrownTank)
            .insert(Enemy)
            .insert(Collider::Enemy);
    }
}

fn player_movement_system(
    time: Res<Time>,
    keyboard_input: Res<Input<KeyCode>>,
    mut queries: QuerySet<(
        Query<(&Transform, &Sprite), With<Player>>,
        Query<(&Collider, &Transform, &Sprite)>,
        Query<(&Player, &mut Transform)>)>,
) {
    let mut collisions: Vec<Collision> = vec![];
    if let Ok((player_transform, player_sprite)) = queries.q0().single() {
        for (collider, transform, sprite) in queries.q1().iter() {
            let collision = collide(
                player_transform.translation,
                player_sprite.size,
                transform.translation,
                sprite.size
            );

            if let Some(collision) = collision {
                match *collider {
                    Collider::Wall | Collider::Enemy => collisions.push(collision),
                    _ => ()
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

        let normalized_direction = direction.try_normalize().unwrap_or_else(|| Vec2::ZERO);
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

fn cursor_position_system(
    windows: Res<Windows>,
    mut cursor_position: ResMut<CursorPosition>,
) {
    if let Some(cursor_pos) = calculate_cursor_position(windows) {
        cursor_position.pos = cursor_pos;
    }
}

fn calculate_cursor_position(windows: Res<Windows>) -> Option<Vec2> {
    let window = windows.get_primary()?;
    let cursor_position = window.cursor_position()?;
    Some(Vec2::new(cursor_position.x - window.width() / 2.0, cursor_position.y - window.height() / 2.0))
}

fn bullet_cleanup_system(
    mut commands: Commands,
    windows: Res<Windows>,
    query: Query<(Entity, &Transform), With<Bullet>>,
) {
    if let Some(window) = windows.get_primary() {
        for (entity, transform) in query.iter() {
            if transform.translation.x > window.width() / 2.0 ||
                transform.translation.x < 0.0 - window.width() / 2.0 ||
                transform.translation.y > window.height() / 2.0 ||
                transform.translation.y < 0.0 - window.height() / 2.0
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
        if mouse_input.just_pressed(MouseButton::Left) &&
            bullet_query
                .iter()
                .filter(|owner| owner.owner == player_entity)
                .count() < bullet_limit.limit.into() 
        {
            if let Some(bullet_direction) = Vec3::new(
                cursor_position.pos.x - player_transform.translation.x,
                cursor_position.pos.y - player_transform.translation.y,
                0.0
            ).try_normalize() {
                let texture_handle = asset_server.load("bullet.png");
                let bullet_transform = Transform::from_xyz(
                    player_transform.translation.x,
                    player_transform.translation.y,
                    0.0);
                let sprite_bundle = SpriteBundle {
                    material: materials.add(texture_handle.into()),
                    transform: bullet_transform,
                    ..Default::default()
                };

                // bullet
                commands
                    .spawn_bundle(sprite_bundle)
                    .insert(Bullet { velocity: 150.0 * bullet_direction })
                    .insert(BulletOwner { owner: player_entity })
                    .insert(RicochetLimit { limit: 1 })
                    .insert(RicochetCount { count: 0 })
                    .insert(Collider::Bullet);
            }
        }
    }
}

fn bullet_movement_system(
    time: Res<Time>,
    mut query: Query<(&Bullet, &mut Transform)>,
) {
    for (bullet, mut transform) in query.iter_mut() {
        transform.translation += time.delta_seconds() * bullet.velocity;
    }
}

fn bullet_collision_system(
    mut commands: Commands,
    mut bullet_query: Query<(Entity, &mut Bullet, &BulletOwner, &RicochetLimit, &mut RicochetCount, &Transform, &Sprite)>,
    collider_query: Query<(Entity, &Collider, &Transform, &Sprite)>,
) {
    for (bullet_entity, mut bullet, bullet_owner, ricochet_limit, mut ricochet_count, bullet_transform, bullet_sprite) in bullet_query.iter_mut() {
        let velocity = &mut bullet.velocity;
        for (collider_entity, collider, transform, sprite) in collider_query.iter() {
            let collision = collide(
                bullet_transform.translation,
                bullet_sprite.size,
                transform.translation,
                sprite.size
            );

            if let Some(collision) = collision {
                match *collider {
                    Collider::Bullet => commands.entity(collider_entity).despawn(),
                    Collider::Enemy | Collider::Player => {
                        if !(bullet_owner.owner == collider_entity && ricochet_count.count < 1) {
                            commands.entity(bullet_entity).despawn();
                            commands.entity(collider_entity).despawn()
                        }
                    },
                    Collider::Wall => {
                        let mut reflect_x = false;
                        let mut reflect_y = false;

                        match collision {
                            Collision::Left => reflect_x = velocity.x > 0.0,
                            Collision::Right => reflect_x = velocity.x < 0.0,
                            Collision::Top => reflect_y = velocity.y < 0.0,
                            Collision::Bottom => reflect_y = velocity.y > 0.0,
                        }

                        if ricochet_count.count >= ricochet_limit.limit && (reflect_x || reflect_y) {
                            commands.entity(bullet_entity).despawn();
                        } else {
                            if reflect_x {
                                velocity.x = -velocity.x;
                            }

                            if reflect_y {
                                velocity.y = -velocity.y;
                            }

                            if reflect_x || reflect_y {
                                ricochet_count.count += 1;
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
            if bullet_query.iter()
                .filter(|owner| owner.owner == tank_entity)
                .count() < bullet_limit.limit.into()
            {
                if let Some(bullet_direction) = Vec3::new(
                    player_transform.translation.x - tank_transform.translation.x,
                    player_transform.translation.y - tank_transform.translation.y,
                    0.0
                ).try_normalize() {
                    let texture_handle = asset_server.load("bullet.png");
                    let bullet_transform = Transform::from_xyz(
                        tank_transform.translation.x,
                        tank_transform.translation.y,
                        0.0);
                    let sprite_bundle = SpriteBundle {
                        material: materials.add(texture_handle.into()),
                        transform: bullet_transform,
                        ..Default::default()
                    };

                    commands
                        .spawn_bundle(sprite_bundle)
                        .insert(Bullet { velocity: 150.0 * bullet_direction })
                        .insert(BulletOwner { owner: tank_entity })
                        .insert(RicochetLimit { limit: 1 })
                        .insert(RicochetCount { count: 0 })
                        .insert(Collider::Bullet);
                }
            }
        }
    }
}

fn win_or_lose_system(
    player_query: Query<&Player>,
    enemy_query: Query<&Enemy>,
) {
    if enemy_query.iter().count() == 0 {
        println!("You win!")
    }

    match player_query.single() {
        Err(_) => println!("You lose!"),
        _ => (),
    }
}