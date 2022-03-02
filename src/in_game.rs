use std::ops::DerefMut;

use bevy::prelude::*;
use bevy_inspector_egui::{Inspectable, RegisterInspectable};

use crate::app_state::*;

pub struct InGameStatePlugin;
impl Plugin for InGameStatePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Grid>()
            .register_inspectable::<Grid>()
            .register_inspectable::<GridPosition>()
            .add_event::<PlayerMoveEvent>()
            .add_event::<BlockMoveEvent>()
            .add_event::<ButtonStateChangeEvent>()
            .add_system_set(SystemSet::on_enter(AppState::InGame).with_system(on_enter))
            .add_system_set(
                SystemSet::on_update(AppState::InGame)
                    .with_system(update_player_keyboard)
                    .with_system(player_move_event_listener)
                    .with_system(apply_grid_entity_position)
                    .with_system(block_move_event_listener),
            )
            .add_system_set(SystemSet::on_exit(AppState::InGame).with_system(on_exit));
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum BlockType {
    Red,
    Green,
    Blue,
}

#[derive(Component, Debug)]
enum GridObject {
    Player,
    PushBlock(BlockType),
    Button(BlockType, Option<Entity>),
    Wall,
}

#[derive(Inspectable)]
struct Grid {
    cell_size: f32,
}

#[derive(Component)]
struct Cleanup;

struct Level {
    pressed_button_count: u32,
    total_button_count: u32,
}

struct LevelData {
    objects: Vec<(GridObject, GridPosition)>,
}

struct PlayerMoveEvent(i32, i32);

struct BlockMoveEvent {
    pub block: Entity,
    pub position: (i32, i32),
}

#[derive(Component, Inspectable, Clone, Copy)]
struct GridPosition {
    pub x: i32,
    pub y: i32,
}

impl Default for Grid {
    fn default() -> Self {
        Self { cell_size: 64.0 }
    }
}

enum ButtonStateChangeEvent {
    Pressed(Entity),
    Unpressed(Entity),
}

fn on_enter(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
) {
    let level_data = LevelData {
        objects: vec![
            (GridObject::Player, GridPosition { x: 0, y: 0 }),
            (GridObject::Wall, GridPosition { x: 3, y: 3 }),
            (
                GridObject::Button(BlockType::Red, None),
                GridPosition { x: -2, y: -2 },
            ),
            (
                GridObject::Button(BlockType::Green, None),
                GridPosition { x: 0, y: -2 },
            ),
            (
                GridObject::Button(BlockType::Blue, None),
                GridPosition { x: 2, y: -2 },
            ),
            (
                GridObject::PushBlock(BlockType::Red),
                GridPosition { x: -2, y: 2 },
            ),
            (
                GridObject::PushBlock(BlockType::Green),
                GridPosition { x: 0, y: 2 },
            ),
            (
                GridObject::PushBlock(BlockType::Blue),
                GridPosition { x: 2, y: 2 },
            ),
        ],
    };

    level_data.spawn(&mut commands, &mut meshes, &mut materials, &asset_server);

    commands
        .spawn()
        .insert(Cleanup)
        .insert(GlobalTransform::default())
        .insert(Transform {
            ..Default::default()
        });

    // Bagckground grid
    commands
        .spawn()
        .insert(Cleanup)
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
}

fn on_exit(mut commands: Commands, query: Query<Entity, With<Cleanup>>) {
    for e in query.iter() {
        commands.entity(e).despawn_recursive();
    }
}

fn player_move_event_listener(
    mut listener: EventReader<PlayerMoveEvent>,
    mut grid_objects: Query<(&GridObject, &mut GridPosition, Entity)>,
    mut block_move_events: EventWriter<BlockMoveEvent>,
) {
    for move_dir in listener.iter() {
        let (player_entity, player_pos) = grid_objects
            .iter()
            .find(|e| matches!(e.0, GridObject::Player))
            .map(|e| (e.2, *e.1))
            .expect("No player found with GridPosition");

        let new_player_pos = GridPosition {
            x: player_pos.x + move_dir.0,
            y: player_pos.y + move_dir.1,
        };

        // If there's a wall in the way, don't move
        if let Some((obj, _, other_entity)) = grid_objects.iter().find(|(object, position, _)| {
            position.x == new_player_pos.x
                && position.y == new_player_pos.y
                && !matches!(object, GridObject::Button(_, _))
        }) {
            match obj {
                GridObject::Player => return,
                GridObject::PushBlock(_) => {
                    let new_block_position = GridPosition {
                        x: new_player_pos.x + move_dir.0,
                        y: new_player_pos.y + move_dir.1,
                    };

                    if let None = grid_objects.iter().find(|(object, position, _)| {
                        let is_overlapped = position.x == new_block_position.x
                            && position.y == new_block_position.y;

                        let is_button = matches!(object, GridObject::Button(_, _));

                        if is_button {
                            return false;
                        }

                        return is_overlapped;
                    }) {
                        let (_, mut position, _) = grid_objects
                            .get_mut(player_entity)
                            .expect("Player entity not found whilst pushing");
                        position.x = new_player_pos.x;
                        position.y = new_player_pos.y;

                        let (_, mut position, _) = grid_objects
                            .get_mut(other_entity)
                            .expect("Block entity not found whilst pushing");
                        position.x = new_block_position.x;
                        position.y = new_block_position.y;

                        block_move_events.send(BlockMoveEvent {
                            block: other_entity,
                            position: (new_block_position.x, new_block_position.y),
                        });
                    }

                    return;
                }
                GridObject::Button(_, _) => {
                    let (_, mut position, _) = grid_objects
                        .get_mut(player_entity)
                        .expect("Player entity not found");
                    position.x = new_player_pos.x;
                    position.y = new_player_pos.y;
                }
                GridObject::Wall => return,
            }
        } else {
            let (_, mut position, _) = grid_objects
                .get_mut(player_entity)
                .expect("Player entity not found whilst pushing");
            position.x = new_player_pos.x;
            position.y = new_player_pos.y;
        }
    }
}

fn block_move_event_listener(
    mut move_events: EventReader<BlockMoveEvent>,
    mut button_state_change_event: EventWriter<ButtonStateChangeEvent>,
    mut query: Query<(&GridPosition, &mut GridObject, Entity)>,
) {
    for BlockMoveEvent { block, position } in move_events.iter() {
        let block_object = query.get_component::<GridObject>(*block).unwrap();
        let block_type = if let GridObject::PushBlock(kind) = block_object {
            *kind
        } else {
            continue;
        };

        query.iter_mut().for_each(|(pos, mut object, button)| {
            if let GridObject::Button(button_type, pressing_entity) = object.deref_mut() {
                // This is a button, we care about this one
                // the moved block is already on the button so we know it's being removed
                if let Some(_) = pressing_entity.take() {
                    // TODO WT: Events for buttons being pressed and unpressed (to change the state of the button sprite).
                    println!("Block moved off of button");
                    button_state_change_event.send(ButtonStateChangeEvent::Unpressed(button));
                }

                if pos.x != position.0 || pos.y != position.1 {
                    return;
                }

                if block_type != *button_type {
                    println!("types didn't match");
                    return;
                }

                println!(
                    "Pushed block type {:?}, required type: {:?}",
                    &block_type, &button_type
                );

                *pressing_entity = Some(*block);

                println!("Block moved onto button");
                button_state_change_event.send(ButtonStateChangeEvent::Pressed(button));
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
    } else if keyboard_input.just_pressed(KeyCode::S) || keyboard_input.just_pressed(KeyCode::Down)
    {
        move_dir.1 -= 1;
    } else if keyboard_input.just_pressed(KeyCode::D) || keyboard_input.just_pressed(KeyCode::Right)
    {
        move_dir.0 += 1;
    } else if keyboard_input.just_pressed(KeyCode::A) || keyboard_input.just_pressed(KeyCode::Left)
    {
        move_dir.0 -= 1;
    } else {
        return;
    }

    writer.send(move_dir);
}

fn apply_grid_entity_position(mut query: Query<(&GridPosition, &mut Transform)>, grid: Res<Grid>) {
    for (position, mut transform) in query.iter_mut() {
        transform.translation.x = position.x as f32 * grid.cell_size;
        transform.translation.y = position.y as f32 * grid.cell_size;
    }
}

impl LevelData {
    fn spawn(
        &self,
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<ColorMaterial>>,
        asset_server: &Res<AssetServer>,
    ) {
        for (object_type, position) in &self.objects {
            match object_type {
                GridObject::Player => {
                    commands
                        .spawn_bundle(ColorMesh2dBundle {
                            mesh: meshes.add(shape::Quad::default().into()).into(),
                            material: materials.add(Color::ORANGE.into()),
                            transform: Transform::from_scale(Vec3::splat(60.0))
                                .with_translation(Vec3::new(0.0, 0.0, 100.0)),
                            ..Default::default()
                        })
                        .insert(Cleanup)
                        .insert(GridObject::Player)
                        .insert(*position);
                }
                GridObject::PushBlock(kind) => {
                    commands
                        .spawn_bundle(SpriteBundle {
                            texture: asset_server.load("sprites/box.png"),
                            sprite: Sprite {
                                custom_size: Some(Vec2::splat(64.0)),
                                ..Default::default()
                            },
                            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 99.0)),
                            ..Default::default()
                        })
                        .insert(Cleanup)
                        .insert(GridObject::PushBlock(*kind))
                        .insert(*position)
                        .with_children(|parent| {
                            parent.spawn_bundle(SpriteBundle {
                                texture: asset_server.load("sprites/color_label.png"),
                                sprite: Sprite {
                                    custom_size: Some(Vec2::splat(32.0)),
                                    color: match kind {
                                        BlockType::Red => Color::ORANGE_RED,
                                        BlockType::Green => Color::SEA_GREEN,
                                        BlockType::Blue => Color::ALICE_BLUE,
                                    },
                                    ..Default::default()
                                },
                                ..Default::default()
                            });
                        });
                }
                GridObject::Button(kind, _) => {
                    commands
                        .spawn_bundle(ColorMesh2dBundle {
                            mesh: meshes.add(shape::Quad::default().into()).into(),
                            material: materials.add(
                                match kind {
                                    BlockType::Red => Color::ORANGE_RED,
                                    BlockType::Green => Color::SEA_GREEN,
                                    BlockType::Blue => Color::ALICE_BLUE,
                                }
                                .into(),
                            ),
                            transform: Transform::from_scale(Vec3::splat(64.0))
                                .with_translation(Vec3::new(0.0, 0.0, 5.0)),
                            ..Default::default()
                        })
                        .insert(Cleanup)
                        .insert(GridObject::Button(*kind, None))
                        .insert(*position);
                }
                GridObject::Wall => {
                    commands
                        .spawn_bundle(ColorMesh2dBundle {
                            mesh: meshes.add(shape::Quad::default().into()).into(),
                            material: materials.add(Color::DARK_GRAY.into()),
                            transform: Transform::from_scale(Vec3::splat(60.0))
                                .with_translation(Vec3::new(0.0, 0.0, 99.0)),
                            ..Default::default()
                        })
                        .insert(Cleanup)
                        .insert(GridObject::Wall)
                        .insert(*position);
                }
            }
        }
    }
}
