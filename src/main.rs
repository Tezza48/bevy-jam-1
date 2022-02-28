use std::ops::DerefMut;

use bevy::{prelude::*};
use bevy_inspector_egui::{Inspectable, RegisterInspectable, WorldInspectorPlugin};

struct BlockMoveEvent(Entity);

fn main() {
    App::new()
        .insert_resource(Msaa { samples: 4 })
        .insert_resource(ClearColor(Color::BISQUE))
        .add_plugins(DefaultPlugins)
        .add_plugin(WorldInspectorPlugin::new())
        .register_inspectable::<Grid>()
        .register_inspectable::<GridPosition>()
        .add_startup_system(startup)
        .add_system(update_player_keyboard)
        .add_system(player_move_event_listener)
        .add_system(apply_grid_entity_position)
        .add_system(block_move_event_listener)
        .add_event::<PlayerMoveEvent>()
        .add_event::<BlockMoveEvent>()
        .run();
}

fn startup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands
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
        });

    commands
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
        });

    commands
        .spawn_bundle(ColorMesh2dBundle {
            mesh: meshes.add(shape::Quad::default().into()).into(),
            material: materials.add(Color::RED.into()),
            transform: Transform::from_scale(Vec3::splat(64.0))
                .with_translation(Vec3::new(0.0, 0.0, 5.0)),
            ..Default::default()
        })
        .insert(GridObject::Button(None))
        .insert(GridPosition {
            x: -1,
            y: 2,
        });

    commands
        .spawn_bundle(ColorMesh2dBundle {
            mesh: meshes.add(shape::Quad::default().into()).into(),
            material: materials.add(Color::ORANGE_RED.into()),
            transform: Transform::from_scale(Vec3::splat(56.0))
                .with_translation(Vec3::new(0.0, 0.0, 99.0)),
            ..Default::default()
        })
        .insert(GridObject::PushBlock(BlockType::Regular))
        .insert(GridPosition {
            x: 4, y: 4,
        });

    commands.insert_resource(Grid::default());

    commands
        .spawn()
        .insert(GlobalTransform::default())
        .insert(Transform {
            ..Default::default()
        });

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

struct PlayerMoveEvent(i32, i32);

fn player_move_event_listener(
    mut listener: EventReader<PlayerMoveEvent>,
    mut grid_objects: Query<(&GridObject, &mut GridPosition, Entity)>,
    mut block_move_events: EventWriter<BlockMoveEvent>,
) {
    for move_dir in listener.iter() {
        let (player_entity, player_pos) = grid_objects.iter().find(|e| {
            matches!(e.0, GridObject::Player)
        }).map(|e| {
            (e.2, *e.1)
        }).expect("No player found with GridPosition");
    
        let new_player_pos = GridPosition {
            x: player_pos.x + move_dir.0,
            y: player_pos.y + move_dir.1,
        };
    
        // If there's a wall in the way, don't move
        if let Some((obj, _, other_entity)) = grid_objects.iter().find(|(object, position, _)| {
            position.x == new_player_pos.x && position.y == new_player_pos.y && !matches!(object, GridObject::Button(_))
        }) {
            match obj {
                GridObject::Player => return,
                GridObject::PushBlock(_) => {
                    let new_block_position = GridPosition {
                        x: new_player_pos.x + move_dir.0,
                        y: new_player_pos.y + move_dir.1,
                    };
                    
                    if let None = grid_objects.iter().find(|(object, position, _)| {
                        let is_overlapped = position.x == new_block_position.x && position.y == new_block_position.y;
                        
                        let is_button = matches!(object, GridObject::Button(_));

                        if is_button {
                            return false;
                        }
                        
                        return is_overlapped;
                    }) {
                        let (_, mut position, _) = grid_objects.get_mut(player_entity).expect("Player entity not found whilst pushing");
                        position.x = new_player_pos.x;
                        position.y = new_player_pos.y;
                        
                        let (_, mut position, _) = grid_objects.get_mut(other_entity).expect("Block entity not found whilst pushing");
                        position.x = new_block_position.x;
                        position.y = new_block_position.y;

                        block_move_events.send(BlockMoveEvent(other_entity));
                    }

                    // 
                    // let overlapping_objects = grid_objects.iter().filter(|(_, position, _)| {
                    //     position.x == new_block_position.x && position.y == new_block_position.y
                    // }).map(|(object, _, overlapped_entity)| {
                    //     if let GridObject::Button(_) = object {
                    //         return true
                    //     }
                    //     false
                    // }).reduce(|acc, is_button| { acc && is_button});

                    // if let Some(has_button) = overlapping_objects {
                    //     if has_button {
                    //         let (_, mut position, _) = grid_objects.get_mut(player_entity).expect("Player entity not found whilst pushing");
                    //         position.x = new_player_pos.x;
                    //         position.y = new_player_pos.y;
                            
                    //         let (_, mut position, _) = grid_objects.get_mut(other_entity).expect("Block entity not found whilst pushing");
                    //         position.x = new_block_position.x;
                    //         position.y = new_block_position.y;
    
                    //         block_move_events.send(BlockMoveEvent(other_entity));
                    //     }
                    // }
                    // for (object, _, overlapped_entity) in overlapping_objects {
                        
                    // }

                    // Blocks are allowed to be moved over buttons
                    // if let Some((object, _, _)) = grid_objects.iter().find(|(_, position, _)| {
                    //     position.x == new_block_position.x && position.y == new_block_position.y
                    // }) {
                    //     if let GridObject::Button(_) = object {
                    //         let (_, mut position, _) = grid_objects.get_mut(player_entity).expect("Player entity not found whilst pushing");
                    //         position.x = new_player_pos.x;
                    //         position.y = new_player_pos.y;
                            
                    //         let (_, mut position, _) = grid_objects.get_mut(other_entity).expect("Block entity not found whilst pushing");
                    //         position.x = new_block_position.x;
                    //         position.y = new_block_position.y;
    
                    //         block_move_events.send(BlockMoveEvent(other_entity));
                    //     }
                    // }
                    return;
                },
                GridObject::Button(_) => {
                    let (_, mut position, _) = grid_objects.get_mut(player_entity).expect("Player entity not found");
                    position.x = new_player_pos.x;
                    position.y = new_player_pos.y;
                }
                GridObject::Wall => return,
            }
        } else {
            let (_, mut position, _) = grid_objects.get_mut(player_entity).expect("Player entity not found whilst pushing");
            position.x = new_player_pos.x;
            position.y = new_player_pos.y;
        }
    }
}

fn block_move_event_listener(
    mut move_events: EventReader<BlockMoveEvent>,
    mut query: Query<(&GridPosition, &mut GridObject, Entity)>,
) {
    for entity in move_events.iter() {
        let moved_block = entity.0;
        let block_pos = *query.get_component::<GridPosition>(moved_block).unwrap();

        query.iter_mut().for_each(|(pos, mut object, moved_block)| {
            if let GridObject::Button(pressing_entity) = object.deref_mut() {
                // This is a button, we care about this one
                // the moved block is already on the button so we know it's being removed
                if let Some(_) = pressing_entity.take() {
                    // TODO WT: Events for buttons being pressed and unpressed (to change the state of the button sprite).
                    println!("Block moved off of button");
                }

                if pos.x != block_pos.x || pos.y != block_pos.y {
                    return;
                }

                *pressing_entity = Some(moved_block);

                println!("Block moved onto button");

            } else {
                return;
            }
        });

        // check to see if block was moved off

    }
}

fn update_player_keyboard(
    keyboard_input: Res<Input<KeyCode>>,
    mut writer: EventWriter<PlayerMoveEvent>,
) {
    let mut move_dir = PlayerMoveEvent(0, 0);
        
    if keyboard_input.just_pressed(KeyCode::W) || keyboard_input.just_pressed(KeyCode::Up) {
        move_dir.1 += 1;
    } else if keyboard_input.just_pressed(KeyCode::S) || keyboard_input.just_pressed(KeyCode::Down) {
        move_dir.1 -= 1;
    } else if keyboard_input.just_pressed(KeyCode::D) || keyboard_input.just_pressed(KeyCode::Right) {
        move_dir.0 += 1;
    } else if keyboard_input.just_pressed(KeyCode::A) || keyboard_input.just_pressed(KeyCode::Left) {
        move_dir.0 -= 1;
    } else {
        return;
    }

    writer.send(move_dir);
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
    Regular,
    Green,
    Blue,
}


#[derive(Component)]
enum GridObject {
    Player,
    PushBlock(BlockType),
    Button(Option<Entity>),
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
