use std::fmt::Alignment;

use bevy::{math, prelude::*};

use crate::app_state::AppState;

use super::ButtonStateChangeEvent;
use super::Level;
use super::LevelInitialized;

struct LevelCompleteEvent;

pub struct GameUiPlugin;
impl Plugin for GameUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<LevelCompleteEvent>()
            .add_system_set(SystemSet::on_enter(AppState::InGame).with_system(create_ui))
            .add_system_set(
                SystemSet::on_update(AppState::InGame)
                    .with_system(update_ui)
                    .with_system(level_complete_ui),
            );
    }
}

fn create_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load("font/gomarice_gogono_cocoa_mochi.ttf");

    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                // position: todo!(),
                align_self: AlignSelf::FlexEnd,
                position_type: PositionType::Absolute,
                position: math::Rect {
                    right: Val::Px(0.0),
                    ..Default::default()
                },
                ..Default::default()
            },
            color: UiColor(Color::BLACK),
            ..Default::default()
        })
        .with_children(|parent| {
            parent.spawn_bundle(NodeBundle {
                image: UiImage(asset_server.load("sprites/box.png")),
                style: Style {
                    size: Size {
                        width: Val::Px(50.0),
                        height: Val::Px(50.0),
                    },
                    ..Default::default()
                },
                ..Default::default()
            });

            parent
                .spawn_bundle(TextBundle {
                    text: Text::with_section(
                        "x / x",
                        TextStyle {
                            font,
                            font_size: 25.0,
                            color: Color::WHITE,
                        },
                        TextAlignment::default(),
                    ),
                    ..Default::default()
                })
                .insert(PressedButtonsDisplay);
        });
}

#[derive(Component)]
struct PressedButtonsDisplay;

fn update_ui(
    mut button_state_change_event: EventReader<ButtonStateChangeEvent>,
    mut level_initialized_event: EventReader<LevelInitialized>,
    mut query: Query<&mut Text, With<PressedButtonsDisplay>>,
    mut level: ResMut<Level>,
    mut level_complete_event: EventWriter<LevelCompleteEvent>,
) {
    fn update_text_count(
        level: &mut Level,
        query: &mut Query<&mut Text, With<PressedButtonsDisplay>>,
    ) {
        for mut text in query.iter_mut() {
            text.sections[0].value = format!(
                "{} / {}",
                &level.pressed_button_count, &level.total_button_count
            );
        }
    }

    for _ in level_initialized_event.iter() {
        update_text_count(&mut level, &mut query);
    }

    for button_state in button_state_change_event.iter() {
        match button_state {
            ButtonStateChangeEvent::Pressed(_) => {
                println!("Pressed");
                level.pressed_button_count += 1;

                if level.pressed_button_count == level.total_button_count {
                    level_complete_event.send(LevelCompleteEvent);
                }
            }
            ButtonStateChangeEvent::Unpressed(_) => {
                println!("Unpressed (before check)");
                if level.pressed_button_count == 0 {
                    println!("\t count already 0");
                    continue;
                }

                println!("Decreasing count");
                level.pressed_button_count =
                    level.total_button_count.min(level.pressed_button_count - 1);
            }
        }

        update_text_count(&mut level, &mut query);
    }
}

fn level_complete_ui(
    mut level_complete_event: EventReader<LevelCompleteEvent>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    for _ in level_complete_event.iter() {
        println!("Level is complete");
        commands.spawn_bundle(TextBundle {
            style: Style {
                align_self: AlignSelf::FlexEnd,
                position_type: PositionType::Absolute,
                position: Rect {
                    bottom: Val::Percent(50.0),
                    right: Val::Percent(50.0),
                    ..Default::default()
                },
                ..Default::default()
            },
            text: Text::with_section(
                "Level Complete",
                TextStyle {
                    font: asset_server.load("font/roboto_thin.ttf"),
                    font_size: 90.0,
                    color: Color::WHITE,
                },
                TextAlignment::default(),
            ),
            ..Default::default()
        });
    }
}
