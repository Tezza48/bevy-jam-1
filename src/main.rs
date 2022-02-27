use std::collections::{HashMap, HashSet};

use bevy::{prelude::*, math::XY};
use bevy_inspector_egui::{Inspectable, RegisterInspectable, WorldInspectorPlugin};

fn main() {
    App::new()
        .insert_resource(Msaa { samples: 4 })
        .insert_resource(ClearColor(Color::BISQUE))
        .add_plugins(DefaultPlugins)
        .add_plugin(WorldInspectorPlugin::new())
        .register_inspectable::<Grid>()
        .register_inspectable::<GridPosition>()
        // .register_inspectable::<GridObject>()
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
        .insert(GridObject::Player)
        .insert(GridPosition {
            x: 0, y: 0,
        })
        .id();

    let wall = commands
        .spawn_bundle(ColorMesh2dBundle {
            mesh: meshes.add(shape::Quad::default().into()).into(),
            material: materials.add(Color::DARK_GRAY.into()),
            transform: Transform::from_scale(Vec3::splat(60.0))
                .with_translation(Vec3::new(0.0, 0.0, 99.0)),
            ..Default::default()
        })
        .insert(GridObject::Wall)
        .insert(GridPosition {
            x: 3, y: 3,
        })
        .id();

    let block = commands
        .spawn_bundle(ColorMesh2dBundle {
            mesh: meshes.add(shape::Quad::default().into()).into(),
            material: materials.add(Color::ORANGE_RED.into()),
            transform: Transform::from_scale(Vec3::splat(56.0))
                .with_translation(Vec3::new(0.0, 0.0, 99.0)),
            ..Default::default()
        })
        .insert(GridObject::PushBlock(BlockType::Red))
        .insert(GridPosition {
            x: 4, y: 4,
        })
        .id();

    let mut grid_component = Grid::default();
    // grid_component.objects.insert((0, 0), (GridObject::Movable, player));
    // grid_component.objects.insert((3, 3), (GridObject::Immovable, wall));
    // grid_component.objects.insert((4, 4), (GridObject::Movable, block));

    commands.insert_resource(grid_component);

    commands
        .spawn()
        .insert(GlobalTransform::default())
        .insert(Transform {
            ..Default::default()
        });
        // .add_child(player)
        // .add_child(block)
        // .add_child(wall);

    // Bagckground grid
    commands.spawn()
    .insert(GlobalTransform::default())
    .insert(Transform::default())
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
    mut grid: ResMut<Grid>,
    mut grid_objects: Query<(&GridObject, &mut GridPosition, Entity)>,
) {
    let mut move_dir = (0, 0);

    if keyboard_input.just_pressed(KeyCode::W) || keyboard_input.just_pressed(KeyCode::Up) {
        move_dir.1 += 1;
    } else if keyboard_input.just_pressed(KeyCode::S) || keyboard_input.just_pressed(KeyCode::Down) {
        move_dir.1 -= 1;
    } else if keyboard_input.just_pressed(KeyCode::D) || keyboard_input.just_pressed(KeyCode::Right) {
        move_dir.0 += 1;
    } else if keyboard_input.just_pressed(KeyCode::A) || keyboard_input.just_pressed(KeyCode::Left) {
        move_dir.0 -= 1;
    }
    
    let (player_entity, player_pos) = grid_objects.iter().find(|e| {
        matches!(e.0, GridObject::Player)
    }).map(|e| {
        (e.2, *e.1)
    }).expect("No player found with GridPosition");

    let new_player_pos = GridPosition {
        x: player_pos.x + move_dir.0,
        y: player_pos.y + move_dir.1,
    };

    // // If there's nothing in the way, move the player.
    // if let None = grid_objects.iter().find(|(obj, position, entity)| {
    //     if position.x == new_player_pos.x && position.y == new_player_pos.y {
    //         return true;
    //     }

    //     false
    // }) {
    //     let (_, mut position, _) = grid_objects.get_mut(player_entity).unwrap();
    //     position.x = new_player_pos.x;
    //     position.y = new_player_pos.y;

    //     return;
    // }

    // If there's a wall in the way, don't move
    if let Some((obj, pos, other_entity)) = grid_objects.iter().find(|(obj, position, _)| {
        position.x == new_player_pos.x && position.y == new_player_pos.y
    }) {
        match obj {
            GridObject::Player => return,
            GridObject::PushBlock(_) => {
                let new_block_position = GridPosition {
                    x: new_player_pos.x + move_dir.0,
                    y: new_player_pos.y + move_dir.1,
                };

                // if there's nothing where the block would be pushed to
                if let None = grid_objects.iter().find(|(obj, position, _)| {
                    position.x == new_block_position.x && position.y == new_block_position.y
                }) {
                    let (_, mut position, _) = grid_objects.get_mut(player_entity).unwrap();
                    position.x = new_player_pos.x;
                    position.y = new_player_pos.y;
                    
                    let (_, mut position, _) = grid_objects.get_mut(other_entity).unwrap();
                    position.x = new_block_position.x;
                    position.y = new_block_position.y;
                }
                return;
            },
            GridObject::Button(_) => {
                let (_, mut position, _) = grid_objects.get_mut(player_entity).unwrap();
                position.x = new_player_pos.x;
                position.y = new_player_pos.y;
                
            }
            GridObject::Wall => return,
        }
    } else {
        let (_, mut position, _) = grid_objects.get_mut(player_entity).unwrap();
        position.x = new_player_pos.x;
        position.y = new_player_pos.y;

        return;
    }
}

fn apply_grid_entity_position(
    mut query: Query<(&GridPosition, &mut Transform)>,
    grid: Res<Grid>,
) {
    for (position, mut transform) in query.iter_mut() {
        transform.translation.x = position.x as f32 * grid.cell_size;
        transform.translation.y = position.y as f32 * grid.cell_size;
    }
}

enum BlockType {
    Red,
    Green,
    Blue,
}


#[derive(Component)]
enum GridObject {
    Player,
    PushBlock(BlockType),
    Button(BlockType),
    Wall,
}

#[derive(Inspectable)]
struct Grid {
    cell_size: f32,
}

#[derive(Component, Inspectable, Clone, Copy)]
struct GridPosition {
    x: i32,
    y: i32,
}

impl Default for Grid {
    fn default() -> Self {
        Self {
            cell_size: 64.0,
        }
    }
}
