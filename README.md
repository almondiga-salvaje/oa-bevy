I have a dream of sharing my ideas and knowledge to my coworkers

# Installation

```bash
# https://rust-lang.org/learn/get-started/ 
# install crate (rust package manager)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# install rust analyzer on vscode 
# https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer

cargo run
```

# Bevy Cheatsheet

Quick reference snippets from [Bevy Cheatbook](https://bevy-cheatbook.github.io/)

## Spawning Entities

### Basic entity spawn
```rust
commands.spawn((
    ComponentA,
    ComponentB::default(),
));
```

### Spawn with transform and sprite
```rust
commands.spawn((
    Sprite::from_image(asset_server.load("sprite.png")),
    Transform::from_xyz(0.0, 0.0, 0.0),
    Player,
));
```

### Spawn entity bundle
```rust
commands.spawn(SpriteBundle {
    texture: asset_server.load("icon.png"),
    transform: Transform::from_xyz(100.0, 200.0, 0.0),
    ..default()
});
```

## Hierarchy (Parent-Child Relationships)

### Spawn with children
```rust
commands.spawn((
    Parent,
    Transform::default(),
)).with_children(|parent| {
    parent.spawn((
        Child,
        Transform::from_xyz(0.0, 10.0, 0.0),
    ));
    parent.spawn((
        AnotherChild,
        Transform::from_xyz(5.0, 0.0, 0.0),
    ));
});
```

### Add child to existing entity
```rust
commands.entity(parent_entity).with_children(|parent| {
    parent.spawn(ChildComponent);
});
```

### Push child
```rust
commands.entity(parent_entity).add_child(child_entity);
```

## Queries

### Basic query
```rust
fn system(query: Query<&ComponentA>) {
    for component in query.iter() {
        // Use component
    }
}
```

### Query multiple components
```rust
fn system(query: Query<(&ComponentA, &ComponentB, &mut ComponentC)>) {
    for (a, b, mut c) in query.iter() {
        // Read a, b; mutate c
    }
}
```

### Query with filters - With
```rust
// Only entities that have both Player AND Health
fn system(query: Query<&Transform, With<Player>>) {
    for transform in query.iter() {
        // Only processes entities with Player component
    }
}
```

### Query with filters - Without
```rust
// Only entities that have Transform but NOT Dead
fn system(query: Query<&Transform, Without<Dead>>) {
    for transform in query.iter() {
        // Skips dead entities
    }
}
```

### Combined With and Without
```rust
fn system(
    query: Query<(&Transform, &Health), (With<Player>, Without<Dead>)>
) {
    for (transform, health) in query.iter() {
        // Only alive players
    }
}
```

### Multiple query filters
```rust
fn system(
    player_query: Query<&Transform, (With<Player>, Without<Enemy>)>,
    enemy_query: Query<&Transform, (With<Enemy>, Without<Player>)>,
) {
    // Separate queries for players and enemies
}
```

### Single entity query
```rust
fn system(query: Query<&Player>) {
    if let Ok(player) = query.single() {
        // Exactly one player
    }
}

fn system(mut query: Query<&mut Health, With<Player>>) {
    if let Ok(mut health) = query.single_mut() {
        health.current -= 10.0;
    }
}
```

### Get specific entity
```rust
fn system(query: Query<&Health>) {
    if let Ok(health) = query.get(entity_id) {
        // Access specific entity
    }
}
```

### Query with Entity ID
```rust
fn system(query: Query<(Entity, &Transform, &Health)>) {
    for (entity, transform, health) in query.iter() {
        // entity is the Entity ID
    }
}
```

## Resources

### Define a resource
```rust
#[derive(Resource)]
struct GameSettings {
    volume: f32,
    difficulty: u32,
}

#[derive(Resource, Default)]
struct Score(u32);
```

### Insert resource in main
```rust
App::new()
    .insert_resource(GameSettings { volume: 0.8, difficulty: 1 })
    .init_resource::<Score>() // Uses Default trait
    .run();
```

### Access resource in system
```rust
fn system(settings: Res<GameSettings>) {
    println!("Volume: {}", settings.volume);
}

fn system_mut(mut score: ResMut<Score>) {
    score.0 += 10;
}
```

### Optional resource
```rust
fn system(settings: Option<Res<GameSettings>>) {
    if let Some(settings) = settings {
        // Resource exists
    }
}
```

## Entity Iteration

### Iterate and modify
```rust
fn system(mut query: Query<(&mut Transform, &Velocity)>) {
    for (mut transform, velocity) in query.iter_mut() {
        transform.translation += velocity.0;
    }
}
```

### Iterate with entity ID
```rust
fn system(
    mut commands: Commands,
    query: Query<(Entity, &Health)>
) {
    for (entity, health) in query.iter() {
        if health.current <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}
```

### Iterate over children
```rust
fn system(
    parent_query: Query<&Children, With<Parent>>,
    child_query: Query<&Transform>,
) {
    for children in parent_query.iter() {
        for child_entity in children.iter() {
            if let Ok(child_transform) = child_query.get(child_entity) {
                // Access child data
            }
        }
    }
}
```

### Despawn with children
```rust
fn system(
    mut commands: Commands,
    query: Query<(Entity, &Children), With<Enemy>>
) {
    for (entity, children) in query.iter() {
        // Despawn all children first
        for child in children.iter() {
            commands.entity(child).despawn();
        }
        // Then despawn parent
        commands.entity(entity).despawn();
    }
}
```

## Commands

### Spawn entity and get ID
```rust
let entity_id = commands.spawn(MyComponent).id();
```

### Insert/remove components
```rust
commands.entity(entity_id).insert(NewComponent);
commands.entity(entity_id).remove::<OldComponent>();
```

### Despawn entity
```rust
commands.entity(entity_id).despawn();
commands.entity(entity_id).despawn_recursive(); // With children
```

## Timers

### Create timer resource
```rust
#[derive(Resource)]
struct SpawnTimer(Timer);

App::new()
    .insert_resource(SpawnTimer(Timer::from_seconds(1.0, TimerMode::Repeating)))
```

### Use timer in system
```rust
fn system(time: Res<Time>, mut timer: ResMut<SpawnTimer>) {
    timer.0.tick(time.delta());
    
    if timer.0.just_finished() {
        // Spawn enemy every second
    }
}
```

### Component-based timer
```rust
#[derive(Component)]
struct Lifetime {
    timer: Timer,
}

fn system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Lifetime)>,
    time: Res<Time>,
) {
    for (entity, mut lifetime) in query.iter_mut() {
        lifetime.timer.tick(time.delta());
        
        if lifetime.timer.finished() {
            commands.entity(entity).despawn();
        }
    }
}
```

## Events

### Define custom event
```rust
#[derive(Event)]
struct PlayerDiedEvent;

#[derive(Event)]
struct ScoreEvent(u32);
```

### Register event
```rust
App::new()
    .add_event::<PlayerDiedEvent>()
```

### Send event
```rust
fn system(mut events: EventWriter<PlayerDiedEvent>) {
    events.send(PlayerDiedEvent);
}
```

### Receive event
```rust
fn system(mut events: EventReader<PlayerDiedEvent>) {
    for event in events.read() {
        // Handle event
    }
}
```

## Common Patterns

### Distance check
```rust
fn check_collisions(
    query_a: Query<&Transform, With<Player>>,
    query_b: Query<&Transform, With<Enemy>>,
) {
    for player_transform in query_a.iter() {
        for enemy_transform in query_b.iter() {
            let distance = player_transform.translation.distance(enemy_transform.translation);
            
            if distance < 30.0 {
                // Collision!
            }
        }
    }
}
```

### Changed detection
```rust
fn system(query: Query<&Health, Changed<Health>>) {
    for health in query.iter() {
        // Only runs when Health component changes
    }
}
```

### Asset loading
```rust
fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        Sprite::from_image(asset_server.load("sprite.png")),
        Transform::default(),
    ));
}
```
