use bevy::prelude::*;
use rand::Rng;

// Game States
#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
enum GameState {
    #[default]
    Menu,
    Playing,
    GameOver,
}

#[derive(Component)]
struct MenuUI;

// Components
#[derive(Component)]
struct Player;

#[derive(Component)]
struct Enemy;

#[derive(Component)]
struct ShootingEnemy {
    shoot_timer: Timer,
}

#[derive(Component)]
struct EnemySpeed(f32);

#[derive(Component)]
struct EnemyBullet;

#[derive(Component)]
struct Speed(f32);

#[derive(Component)]
struct Damage(f32);

#[derive(Component)]
struct LastDirection(Vec3);

#[derive(Component)]
struct Bullet {
    velocity: Vec3,
    damage: f32,
}

// Health Plugin
struct HealthPlugin;

impl Plugin for HealthPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(CollisionCooldown(Timer::from_seconds(0.5, TimerMode::Repeating)))
            .add_systems(Update, (update_health_bars, check_collisions, check_death));
    }
}

#[derive(Component)]
struct Health {
    current: f32,
    max: f32,
}

#[derive(Component)]
struct HealthBar;

#[derive(Resource)]
struct CollisionCooldown(Timer);

#[derive(Resource)]
struct EnemySpawnTimer(Timer);

#[derive(Component)]
struct GameOverText;

#[derive(Resource, Default)]
struct Score(u32);

#[derive(Component)]
struct ScoreText;

#[derive(Component)]
struct Dead;

#[derive(Component)]
struct Particle {
    velocity: Vec3,
    lifetime: Timer,
}

#[derive(Component)]
struct MeltParticle {
    lifetime: Timer,
    max_radius: f32,
}

#[derive(Resource)]
struct DeathTransition {
    timer: Timer,
    active: bool,
}

impl Default for DeathTransition {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(2.0, TimerMode::Once),
            active: false,
        }
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins(HealthPlugin)
        .init_state::<GameState>()
        .insert_resource(ClearColor(Color::srgba(31.0/255.0, 32.0/255.0, 32.0/255.0, 1.0)))
        .insert_resource(EnemySpawnTimer(Timer::from_seconds(1.0, TimerMode::Repeating)))
        .init_resource::<Score>()
        .init_resource::<DeathTransition>()
        .add_systems(Startup, setup_camera)
        .add_systems(OnEnter(GameState::Menu), setup_menu)
        .add_systems(Update, menu_input.run_if(in_state(GameState::Menu)))
        .add_systems(OnExit(GameState::Menu), cleanup_menu)
        .add_systems(OnEnter(GameState::Playing), setup_game)
        .add_systems(Update, (move_player, move_enemies, camera_follow, shoot_bullet, enemy_shoot_bullets, move_bullets, check_bullet_collisions, update_score_text, update_particles, spawn_enemies).run_if(in_state(GameState::Playing)))
        .add_systems(Update, update_death_transition.run_if(in_state(GameState::Playing)))
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn setup_menu(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        Text::new("GRAGUSI SURVIVORS\n\nPress SPACE to Start\n\nWASD - Move\nLeft Click - Shoot"),
        TextFont {
            font: asset_server.load("font/BigBlueTerm437NerdFontMono-Regular.ttf"),
            font_size: 32.0,
            ..default()
        },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Percent(30.0),
            left: Val::Percent(50.0),
            ..default()
        },
        MenuUI,
    ));
}

fn menu_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if keys.just_pressed(KeyCode::Space) {
        next_state.set(GameState::Playing);
    }
}

fn cleanup_menu(
    mut commands: Commands,
    menu_query: Query<Entity, With<MenuUI>>,
) {
    for entity in menu_query.iter() {
        commands.entity(entity).despawn();
    }
}

fn setup_game(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Spawn player
    commands.spawn((
        Sprite::from_image(asset_server.load("Colored/tile_0006.png")),
        Transform::from_xyz(0.0, 0.0, 0.0).with_scale(Vec3::splat(4.0)),
        Player,
        Speed(200.0),
        Health { current: 100.0, max: 100.0 },
        LastDirection(Vec3::Y),
    )).with_children(|parent| {
        // Health bar background
        parent.spawn((
            Sprite {
                color: Color::srgb(0.3, 0.3, 0.3),
                custom_size: Some(Vec2::new(12.0, 1.5)),
                ..default()
            },
            Transform::from_xyz(0.0, 8.0, 0.0),
        ));
        // Health bar foreground
        parent.spawn((
            Sprite {
                color: Color::srgb(0.0, 0.8, 0.0),
                custom_size: Some(Vec2::new(12.0, 1.5)),
                ..default()
            },
            Transform::from_xyz(0.0, 8.0, 0.1),
            HealthBar,
        ));
    });

    // Spawn enemies
    commands.spawn((
        Sprite::from_image(asset_server.load("Colored/tile_0020.png")),
        Transform::from_xyz(800.0, 400.0, 0.0).with_scale(Vec3::splat(4.0)),
        Enemy,
        Health { current: 50.0, max: 50.0 },
        Damage(10.0),
        EnemySpeed(50.0),
    )).with_children(|parent| {
        // Health bar background
        parent.spawn((
            Sprite {
                color: Color::srgb(0.3, 0.3, 0.3),
                custom_size: Some(Vec2::new(12.0, 1.5)),
                ..default()
            },
            Transform::from_xyz(0.0, 8.0, 0.0),
        ));
        // Health bar foreground
        parent.spawn((
            Sprite {
                color: Color::srgb(0.8, 0.0, 0.0),
                custom_size: Some(Vec2::new(12.0, 1.5)),
                ..default()
            },
            Transform::from_xyz(0.0, 8.0, 0.1),
            HealthBar,
        ));
    });

    commands.spawn((
        Sprite::from_image(asset_server.load("Colored/tile_0027.png")),
        Transform::from_xyz(-700.0, -500.0, 0.0).with_scale(Vec3::splat(4.0)),
        Enemy,
        Health { current: 50.0, max: 50.0 },
        Damage(15.0),
        EnemySpeed(70.0),
    )).with_children(|parent| {
        // Health bar background
        parent.spawn((
            Sprite {
                color: Color::srgb(0.3, 0.3, 0.3),
                custom_size: Some(Vec2::new(12.0, 1.5)),
                ..default()
            },
            Transform::from_xyz(0.0, 8.0, 0.0),
        ));
        // Health bar foreground
        parent.spawn((
            Sprite {
                color: Color::srgb(0.8, 0.0, 0.0),
                custom_size: Some(Vec2::new(12.0, 1.5)),
                ..default()
            },
            Transform::from_xyz(0.0, 8.0, 0.1),
            HealthBar,
        ));
    });

    // Spawn score UI
    commands.spawn((
        Text::new("Score: 0"),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            right: Val::Px(10.0),
            ..default()
        },
        TextFont {
            font: asset_server.load("font/BigBlueTerm437NerdFontMono-Regular.ttf"),
            font_size: 30.0,
            ..default()
        },
        TextColor(Color::WHITE),
        ScoreText,
    ));
}

fn move_player(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut query: Query<(&mut Transform, &Speed, &mut LastDirection), (With<Player>, Without<Dead>)>,
) {
    if let Ok((mut transform, speed, mut last_dir)) = query.single_mut() {
        let mut direction = Vec3::ZERO;

        if keys.pressed(KeyCode::KeyW) { direction.y += 1.0; }
        if keys.pressed(KeyCode::KeyS) { direction.y -= 1.0; }
        if keys.pressed(KeyCode::KeyA) { direction.x -= 1.0; }
        if keys.pressed(KeyCode::KeyD) { direction.x += 1.0; }

        if direction.length() > 0.0 {
            let normalized = direction.normalize();
            last_dir.0 = normalized;
            transform.translation += normalized * speed.0 * time.delta_secs();
        }
    }
}

fn move_enemies(
    time: Res<Time>,
    player_query: Query<&Transform, (With<Player>, Without<Enemy>, Without<Dead>)>,
    mut enemy_query: Query<(&mut Transform, &EnemySpeed), (With<Enemy>, Without<Player>)>,
) {
    if let Ok(player_transform) = player_query.single() {
        for (mut enemy_transform, speed) in enemy_query.iter_mut() {
            let direction = (player_transform.translation - enemy_transform.translation).normalize();
            enemy_transform.translation += direction * speed.0 * time.delta_secs();
        }
    } else {
        // Player is dead, enemies move away from origin
        for (mut enemy_transform, speed) in enemy_query.iter_mut() {
            let direction = enemy_transform.translation.normalize();
            enemy_transform.translation += direction * speed.0 * time.delta_secs();
        }
    }
}

fn spawn_enemies(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    time: Res<Time>,
    mut timer: ResMut<EnemySpawnTimer>,
    player_query: Query<&Transform, With<Player>>,
) {
    timer.0.tick(time.delta());
    
    if timer.0.just_finished() {
        if let Ok(player_transform) = player_query.single() {
            let mut rng = rand::rng();
            
            // Random angle around the player
            let angle = rng.random_range(0.0..std::f32::consts::TAU);
            let distance = 800.0; // Spawn distance from player
            
            let spawn_x = player_transform.translation.x + angle.cos() * distance;
            let spawn_y = player_transform.translation.y + angle.sin() * distance;
            
            // Random enemy type (3 types now)
            let enemy_type = rng.random_range(0..3);
            let (sprite_path, damage, speed, is_shooter) = match enemy_type {
                0 => ("Colored/tile_0020.png", 10.0, rng.random_range(40.0..60.0), false),
                1 => ("Colored/tile_0027.png", 15.0, rng.random_range(60.0..80.0), false),
                _ => ("Colored/tile_0009.png", 5.0, rng.random_range(30.0..40.0), true),
            };
            
            let mut entity = commands.spawn((
                Sprite::from_image(asset_server.load(sprite_path)),
                Transform::from_xyz(spawn_x, spawn_y, 0.0).with_scale(Vec3::splat(4.0)),
                Enemy,
                Health { current: 50.0, max: 50.0 },
                Damage(damage),
                EnemySpeed(speed),
            ));
            
            if is_shooter {
                entity.insert(ShootingEnemy {
                    shoot_timer: Timer::from_seconds(2.0, TimerMode::Repeating),
                });
            }
            
            entity.with_children(|parent| {
                // Health bar background
                parent.spawn((
                    Sprite {
                        color: Color::srgb(0.3, 0.3, 0.3),
                        custom_size: Some(Vec2::new(12.0, 1.5)),
                        ..default()
                    },
                    Transform::from_xyz(0.0, 8.0, 0.0),
                ));
                // Health bar foreground
                parent.spawn((
                    Sprite {
                        color: Color::srgb(0.8, 0.0, 0.0),
                        custom_size: Some(Vec2::new(12.0, 1.5)),
                        ..default()
                    },
                    Transform::from_xyz(0.0, 8.0, 0.1),
                    HealthBar,
                ));
            });
        }
    }
}

fn camera_follow(
    player_query: Query<&Transform, With<Player>>,
    mut camera_query: Query<&mut Transform, (With<Camera2d>, Without<Player>)>,
    time: Res<Time>,
) {
    if let (Ok(player_transform), Ok(mut camera_transform)) = (player_query.single(), camera_query.single_mut()) {
        let target = player_transform.translation;
        camera_transform.translation = camera_transform.translation.lerp(target, 5.0 * time.delta_secs());
    }
}

fn shoot_bullet(
    mut commands: Commands,
    mouse: Res<ButtonInput<MouseButton>>,
    player_query: Query<&Transform, (With<Player>, Without<Dead>)>,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
    windows: Query<&Window>,
) {
    if mouse.just_pressed(MouseButton::Left) {
        if let (Ok(player_transform), Ok((camera, camera_transform)), Ok(window)) = 
            (player_query.single(), camera_query.single(), windows.single()) {
            
            if let Some(cursor_pos) = window.cursor_position() {
                // Convert cursor position to world coordinates
                if let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_pos) {
                    let direction = (world_pos - player_transform.translation.truncate()).normalize().extend(0.0);
                    
                    commands.spawn((
                        Sprite {
                            color: Color::srgb(1.0, 0.2, 0.0),
                            custom_size: Some(Vec2::new(3.0, 3.0)),
                            ..default()
                        },
                        Transform::from_translation(player_transform.translation + direction * 20.0)
                            .with_scale(Vec3::splat(2.0)),
                        Bullet {
                            velocity: direction * 300.0,
                            damage: 25.0,
                        },
                    ));
                }
            }
        }
    }
}

fn enemy_shoot_bullets(
    mut commands: Commands,
    time: Res<Time>,
    player_query: Query<&Transform, (With<Player>, Without<Dead>)>,
    mut enemy_query: Query<(&Transform, &mut ShootingEnemy), Without<Player>>,
) {
    if let Ok(player_transform) = player_query.single() {
        for (enemy_transform, mut shooter) in enemy_query.iter_mut() {
            shooter.shoot_timer.tick(time.delta());
            
            if shooter.shoot_timer.just_finished() {
                let direction = (player_transform.translation - enemy_transform.translation).normalize();
                
                commands.spawn((
                    Sprite {
                        color: Color::srgb(0.8, 0.0, 0.8),
                        custom_size: Some(Vec2::new(3.0, 3.0)),
                        ..default()
                    },
                    Transform::from_translation(enemy_transform.translation + direction * 20.0)
                        .with_scale(Vec3::splat(2.0)),
                    Bullet {
                        velocity: direction * 200.0,
                        damage: 15.0,
                    },
                    EnemyBullet,
                ));
            }
        }
    }
}

fn move_bullets(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transform, &Bullet)>,
    time: Res<Time>,
) {
    for (entity, mut transform, bullet) in query.iter_mut() {
        transform.translation += bullet.velocity * time.delta_secs();
        
        // Despawn bullets that go too far
        if transform.translation.length() > 1000.0 {
            commands.entity(entity).despawn();
        }
    }
}

fn check_bullet_collisions(
    mut commands: Commands,
    bullet_query: Query<(Entity, &Transform, &Bullet, Option<&EnemyBullet>)>,
    mut enemy_query: Query<(Entity, &Transform, &mut Health, &Children), With<Enemy>>,
    mut player_query: Query<(&Transform, &mut Health, &Children), (With<Player>, Without<Enemy>)>,
    mut score: ResMut<Score>,
) {
    for (bullet_entity, bullet_transform, bullet, enemy_bullet) in bullet_query.iter() {
        // Enemy bullets hit the player
        if enemy_bullet.is_some() {
            if let Ok((player_transform, mut player_health, _)) = player_query.single_mut() {
                let distance = bullet_transform.translation.distance(player_transform.translation);
                let collision_distance = 30.0;
                
                if distance < collision_distance {
                    player_health.current = (player_health.current - bullet.damage).max(0.0);
                    commands.entity(bullet_entity).despawn();
                    break;
                }
            }
        } else {
            // Player bullets hit enemies
            for (enemy_entity, enemy_transform, mut health, children) in enemy_query.iter_mut() {
                let distance = bullet_transform.translation.distance(enemy_transform.translation);
                let collision_distance = 30.0;
                
                if distance < collision_distance {
                    // Damage enemy
                    health.current = (health.current - bullet.damage).max(0.0);
                    
                    // Despawn bullet
                    commands.entity(bullet_entity).despawn();
                    
                    // Check if enemy is dead
                    if health.current <= 0.0 {
                        // Increment score
                        score.0 += 1;
                        
                        // Despawn enemy and its children (health bars)
                        for child in children.iter() {
                            commands.entity(child).despawn();
                        }
                        commands.entity(enemy_entity).despawn();
                    }
                    
                    break;
                }
            }
        }
    }
}

fn update_score_text(
    score: Res<Score>,
    mut query: Query<&mut Text, With<ScoreText>>,
) {
    if score.is_changed() {
        for mut text in query.iter_mut() {
            **text = format!("Score: {}", score.0);
        }
    }
}

fn update_particles(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transform, &mut Particle)>,
    time: Res<Time>,
) {
    for (entity, mut transform, mut particle) in query.iter_mut() {
        particle.lifetime.tick(time.delta());
        
        if particle.lifetime.is_finished() {
            commands.entity(entity).despawn();
        } else {
            transform.translation += particle.velocity * time.delta_secs();
            particle.velocity *= 0.98; // Slow down over time
        }
    }
}

fn update_health_bars(
    health_query: Query<(&Health, &Children)>,
    mut bar_query: Query<(&mut Transform, &mut Sprite), With<HealthBar>>,
) {
    for (health, children) in health_query.iter() {
        for child in children.iter() {
            if let Ok((mut transform, mut sprite)) = bar_query.get_mut(child) {
                let health_percent = health.current / health.max;
                if let Some(size) = sprite.custom_size.as_mut() {
                    size.x = 12.0 * health_percent;
                }
                transform.translation.x = -6.0 * (1.0 - health_percent);
            }
        }
    }
}

fn check_collisions(
    mut player_query: Query<(&Transform, &mut Health), With<Player>>,
    enemy_query: Query<(&Transform, &Damage), With<Enemy>>,
    mut cooldown: ResMut<CollisionCooldown>,
    time: Res<Time>,
) {
    cooldown.0.tick(time.delta());
    
    if !cooldown.0.just_finished() {
        return;
    }

    if let Ok((player_transform, mut player_health)) = player_query.single_mut() {
        for (enemy_transform, damage) in enemy_query.iter() {
            let distance = player_transform.translation.distance(enemy_transform.translation);
            let collision_distance = 40.0; // Approximate sprite size * scale
            
            if distance < collision_distance {
                player_health.current = (player_health.current - damage.0).max(0.0);
                println!("Player hit! Health: {}/{}", player_health.current, player_health.max);
            }
        }
    }
}

fn update_death_transition(
    time: Res<Time>,
    mut transition: ResMut<DeathTransition>,
    mut melt_query: Query<(&mut Sprite, &mut MeltParticle)>,
) {
    if !transition.active {
        return;
    }
    
    transition.timer.tick(time.delta());
    
    // Update melting particles - expand them over time
    for (mut sprite, mut melt) in melt_query.iter_mut() {
        melt.lifetime.tick(time.delta());
        let melt_progress = melt.lifetime.fraction();
        
        // Expand the circle as it "melts down"
        if let Some(size) = sprite.custom_size.as_mut() {
            let radius = melt.max_radius * melt_progress;
            *size = Vec2::new(radius * 2.0, radius * 2.0);
        }
    }
    
    if transition.timer.just_finished() {
        transition.active = false;
    }
}

/// If player health is zero, mark as dead, change sprite, spawn melting effect and explosion particles.
fn check_death(
    mut commands: Commands,
    mut player_query: Query<(Entity, &Transform, &Health, &mut Sprite), (With<Player>, Without<Dead>)>,
    game_over_query: Query<Entity, With<GameOverText>>,
    asset_server: Res<AssetServer>,
    mut transition: ResMut<DeathTransition>,
    windows: Query<&Window>,
) {
    if let Ok((player_entity, transform, health, mut sprite)) = player_query.single_mut() {
        if health.current <= 0.0 && game_over_query.is_empty() {
            // Mark player as dead
            commands.entity(player_entity).insert(Dead);
            
            // Change sprite to death sprite
            sprite.image = asset_server.load("Colored/tile_0120.png");
            
            // Start death transition
            transition.active = true;
            transition.timer.reset();
            
            // Get window size for positioning
            let (width, height) = if let Ok(window) = windows.single() {
                (window.width(), window.height())
            } else {
                (800.0, 600.0) // Default fallback
            };
            
            // Spawn melting black circles across the screen
            let mut rng = rand::rng();
            for _ in 0..30 {
                let x = rng.random::<f32>() * width - width / 2.0;
                let y = rng.random::<f32>() * height - height / 2.0;
                let max_radius = 80.0 + rng.random::<f32>() * 100.0;
                
                commands.spawn((
                    Sprite {
                        color: Color::BLACK,
                        custom_size: Some(Vec2::new(0.0, 0.0)),
                        ..default()
                    },
                    Transform::from_xyz(x, y, 10.0),
                    MeltParticle {
                        lifetime: Timer::from_seconds(2.0, TimerMode::Once),
                        max_radius,
                    },
                ));
            }
            
            // Spawn explosion particles
            for _ in 0..50 {
                let angle = rng.random::<f32>() * std::f32::consts::TAU;
                let speed = 100.0 + rng.random::<f32>() * 100.0;
                let velocity = Vec3::new(angle.cos(), angle.sin(), 0.0) * speed;
                
                commands.spawn((
                    Sprite {
                        color: Color::srgb(1.0, rng.random::<f32>() * 0.3, 0.0),
                        custom_size: Some(Vec2::new(2.0, 2.0)),
                        ..default()
                    },
                    Transform::from_translation(transform.translation),
                    Particle {
                        velocity,
                        lifetime: Timer::from_seconds(1.0, TimerMode::Once),
                    },
                ));
            }
            
            // Spawn Game Over UI immediately on top
            commands.spawn((
                Text::new("GAME OVER\n\nPress SPACE for Menu"),
                Node {
                    position_type: PositionType::Absolute,
                    top: Val::Percent(35.0),
                    width: Val::Percent(100.0),
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                TextFont {
                    font: asset_server.load("font/BigBlueTerm437NerdFontMono-Regular.ttf"),
                    font_size: 60.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.0, 0.0)),
                GameOverText,
                ZIndex(1000),
            ));
        }
    }
}
