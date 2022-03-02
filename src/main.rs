mod app_state;
use app_state::*;

mod in_game;

use std::ops::DerefMut;

use bevy::{ecs::system::EntityCommands, prelude::*};
use bevy_inspector_egui::{Inspectable, RegisterInspectable, WorldInspectorPlugin};

fn main() {
    App::new()
        .insert_resource(Msaa { samples: 4 })
        .insert_resource(ClearColor(Color::DARK_GRAY))
        .add_plugins(DefaultPlugins)
        .add_plugin(WorldInspectorPlugin::new())
        // All relating to Menu
        .add_state(AppState::Menu)
        .add_system_set(
            SystemSet::on_exit(AppState::Menu)
            .with_system(in_menu::on_exit)
        )

        .add_system_set(SystemSet::on_enter(AppState::Menu).with_system(in_menu::on_enter))
        // All relating to InGame
        .init_resource::<in_game::Grid>()
        .register_inspectable::<in_game::Grid>()
        .register_inspectable::<in_game::GridPosition>()
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

    // app_state.set(AppState::InGame).expect("Failed to set ");
}

mod in_menu {
    use bevy::prelude::*;

    #[derive(Component)]
    pub struct Cleanup;

    pub fn on_enter(mut commands: Commands, asset_server: Res<AssetServer>) {
        let font = asset_server.load("font/roboto_thin.ttf");
    
        let style = TextStyle {
            font,
            font_size: 50.0,
            color: Color::WHITE,
        };
    
        commands.spawn_bundle(Text2dBundle {
            text: Text::with_section(
                "Learning Difficulties",
                style,
                TextAlignment::default(),
            ),
            ..Default::default()
        })
        .insert(Cleanup);
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

