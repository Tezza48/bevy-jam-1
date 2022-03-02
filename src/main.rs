mod app_state;
use app_state::*;

mod in_game;

use bevy::{prelude::*};
use bevy_inspector_egui::{RegisterInspectable, WorldInspectorPlugin};

fn main() {
    App::new()
        .insert_resource(Msaa { samples: 4 })
        .insert_resource(ClearColor(Color::DARK_GRAY))
        .add_plugins(DefaultPlugins)

        .add_plugin(WorldInspectorPlugin::new())
        .register_inspectable::<in_game::Grid>()
        .register_inspectable::<in_game::GridPosition>()

        // All relating to Menu
        .add_state(AppState::Menu)
        .add_system_set(
            SystemSet::on_exit(AppState::Menu)
            .with_system(in_menu::on_exit)
        )
        .add_system_set(
            SystemSet::on_update(AppState::Menu)
            .with_system(in_menu::play_button)
        )

        .add_system_set(SystemSet::on_enter(AppState::Menu).with_system(in_menu::on_enter))
        // All relating to InGame
        .init_resource::<in_game::Grid>()
        .add_event::<in_game::PlayerMoveEvent>()
        .add_event::<in_game::BlockMoveEvent>()
        .add_event::<in_game::ButtonStateChangeEvent>()
        .add_system_set(SystemSet::on_enter(AppState::InGame).with_system(in_game::on_enter))
        .add_system_set(
            SystemSet::on_update(AppState::InGame)
                .with_system(in_game::update_player_keyboard)
                .with_system(in_game::player_move_event_listener)
                .with_system(in_game::apply_grid_entity_position)
                .with_system(in_game::block_move_event_listener),
        )
        .add_system_set(SystemSet::on_exit(AppState::InGame).with_system(in_game::on_exit))
        .add_startup_system(startup)
        .run();
}

fn startup(mut commands: Commands, mut app_state: ResMut<State<AppState>>) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    commands.spawn_bundle(UiCameraBundle::default());

    // app_state.set(AppState::InGame).expect("Failed to set ");
}

mod in_menu {
    use bevy::{prelude::*, math::Rect};

    use crate::app_state;

    #[derive(Component)]
    pub struct Cleanup;

    pub fn on_enter(mut commands: Commands, asset_server: Res<AssetServer>) {
        let font = asset_server.load("font/roboto_thin.ttf");

        let style = TextStyle {
            font,
            font_size: 50.0,
            color: Color::WHITE,
        };

        commands
            .spawn_bundle(NodeBundle {
                style: Style {
                    size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                    justify_content: JustifyContent::SpaceBetween,
                    ..Default::default()
                },
                color: Color::NONE.into(),
                ..Default::default()
            })
            .insert(Cleanup)
            .with_children(|parent| {
                parent.spawn_bundle(TextBundle {
                    style: Style {
                        align_self: AlignSelf::Center,
                        margin: Rect { left: Val::Percent(20.0), ..Default::default() },
                        ..Default::default()
                    },
                    text: Text::with_section(
                        "Learning Difficulties",
                    style.clone(),
                    TextAlignment::default(),
                    ),
                    ..Default::default()
                });

                parent.spawn_bundle(ButtonBundle {
                    style: Style {
                        size: Size::new(Val::Auto, Val::Auto),
                        margin: Rect { right: Val::Percent(20.0), ..Default::default() },
                        padding: Rect { left: Val::Percent(1.0), right: Val::Percent(1.0), ..Default::default() },
                        // horizontally center child text
                        justify_content: JustifyContent::Center,
                        // vertically center child text
                        align_items: AlignItems::Center,
                        align_self: AlignSelf::Center,
                        ..Default::default()
                    },
                    color: UiColor(Color::rgb(102.0 / 255.0, 102.0 / 255.0, 102.0 / 255.0)),
                    ..Default::default()
                }).with_children(|parent| {
                    parent.spawn_bundle(TextBundle {
                        text: Text::with_section(
                            "Play",
                            style.clone(),
                            TextAlignment::default(),
                        ),
                        ..Default::default()
                    });
                });
            });

        // commands.spawn_bundle(Text2dBundle {
        //     text: Text::with_section(
        //         "Learning Difficulties",
        //         style,
        //         TextAlignment::default(),
        //     ),
        //     ..Default::default()
        // })
        // .insert(Cleanup);
    }

    pub fn play_button(
        mut interaction_query: Query<(&Interaction, &mut UiColor), With<Button>>,
        mut state: ResMut<State<app_state::AppState>>,
    ) {
        for (interaction, mut color) in interaction_query.iter_mut() {
            match interaction {
                Interaction::Clicked => {
                    *color = UiColor(Color::rgb(0.3, 0.5, 0.6));

                    state.set(app_state::AppState::InGame).unwrap();
                },
                Interaction::Hovered => {
                    println!("Hovered a button");
                    *color = UiColor(Color::rgb(0.4, 0.5, 0.6));
                },
                Interaction::None => {
                    *color = UiColor(Color::rgb(0.4, 0.4, 0.4));
                },
            }
        }
    }

    pub fn on_exit(
        mut commands: Commands,
        query: Query<Entity, With<Cleanup>>,
    ) {
        for e in query.iter() {
            commands.entity(e).despawn_recursive();
        }
    }
}

