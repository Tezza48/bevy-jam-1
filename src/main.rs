use std::collections::{HashMap, HashSet};

use bevy::prelude::*;
use bevy_inspector_egui::{Inspectable, RegisterInspectable, WorldInspectorPlugin};

fn main() {
    App::new()
        .insert_resource(Msaa { samples: 4 })
        .insert_resource(ClearColor(Color::BISQUE))
        .add_plugins(DefaultPlugins)
        .add_plugin(WorldInspectorPlugin::new())
        .register_inspectable::<GridPosition>()
        .register_inspectable::<Grid>()
        .add_startup_system(startup)
        .add_system(update_player_keyboard)
        .add_system(apply_grid_entity_position)
        .run();
}

fn startup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let player = commands
        .spawn_bundle(ColorMesh2dBundle {
            mesh: meshes.add(shape::Quad::default().into()).into(),
            material: materials.add(Color::ORANGE.into()),
            transform: Transform::from_scale(Vec3::splat(60.0))
                .with_translation(Vec3::new(0.0, 0.0, 100.0)),
            ..Default::default()
        })
        .insert(Player)
        .insert(GridPosition::default())
        .id();

    let wall = commands
    .spawn_bundle(ColorMesh2dBundle {
        mesh: meshes.add(shape::Quad::default().into()).into(),
        material: materials.add(Color::DARK_GRAY.into()),
        transform: Transform::from_scale(Vec3::splat(60.0))
            .with_translation(Vec3::new(0.0, 0.0, 99.0)),
        ..Default::default()
    })
    .insert(Immovable)
    .insert(Impassible)
    .insert(GridPosition {
        pos: (3, 3),
    })
    .id();

    commands
        .spawn()
        .insert(GlobalTransform::default())
        .insert(Transform {
            ..Default::default()
        })
        .insert(Grid::default())
        .add_child(player)
        .add_child(wall)
        .with_children(|parent| {
            for y in -16..16 {
                for x in -16..16 {
                    // TODO WT: Spawn a background for the actual level
                    parent.spawn_bundle(ColorMesh2dBundle {
                        mesh: meshes.add(shape::Quad::default().into()).into(),
                        material: materials.add(Color::GRAY.into()),
                        transform: Transform::from_scale(Vec3::splat(63.0))
                            .with_translation(Vec3::new(x as f32 * 64.0, y as f32 * 64.0, 0.0)),
                        ..Default::default()
                    });
                }
            }
        });

    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
}

fn update_player_keyboard(
    keyboard_input: Res<Input<KeyCode>>,
    mut queries: QuerySet<(QueryState<&mut GridPosition, With<Player>>, QueryState<&GridPosition, With<Impassible>>)>,
) {
    
    let mut x = 0;
    let mut y = 0;

    if keyboard_input.just_pressed(KeyCode::W) || keyboard_input.just_pressed(KeyCode::Up) {
        y += 1;
    }
    if keyboard_input.just_pressed(KeyCode::S) || keyboard_input.just_pressed(KeyCode::Down) {
        y -= 1;
    }
    if keyboard_input.just_pressed(KeyCode::D) || keyboard_input.just_pressed(KeyCode::Right) {
        x += 1;
    }
    if keyboard_input.just_pressed(KeyCode::A) || keyboard_input.just_pressed(KeyCode::Left) {
        x -= 1;
    }
    
    // TODO WT: handle gamepad input

    let disallowed_locations = queries.q1().iter().map(|g| {g.pos}).collect::<HashSet<_>>();

    for mut grid_entity in queries.q0().iter_mut() {
        let new_pos = (grid_entity.pos.0 + x, grid_entity.pos.1 + y);

        if let None = disallowed_locations.get(&new_pos) {
            grid_entity.pos = new_pos;
        }
    }
}

fn apply_grid_entity_position(
    mut query: Query<(&GridPosition, &mut Transform)>,
    grid_query: Query<&Grid>,
    // time: Res<Time>,
) {
    let grid = grid_query.iter().next().unwrap();
    for (grid_entity, mut transform) in query.iter_mut() {
        transform.translation = Vec3::new(
            grid_entity.pos.0 as f32 * grid.cell_size,
            grid_entity.pos.1 as f32 * grid.cell_size,
            transform.translation.z,
        );
    }
}

#[derive(Component, Inspectable)]
struct Grid {
    cell_size: f32,
    // #[inspectable(ignore)]
    // objects: HashMap<(u32, u32), Entity>,
}

impl Default for Grid {
    fn default() -> Self {
        Self {
            cell_size: 64.0,
            // objects: HashMap::new(),
        }
    }
}

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Immovable;

#[derive(Component)]
struct Impassible;

#[derive(Component, Default, Inspectable)]
struct GridPosition {
    pos: (i32, i32),
    // last_pos: Vec2,
    // movable: bool,
}
