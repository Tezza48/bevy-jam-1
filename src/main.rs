mod app_state;
mod in_menu;
mod in_game;

use app_state::*;


use bevy::{prelude::*};
use bevy_inspector_egui::{RegisterInspectable, WorldInspectorPlugin};

fn main() {
    App::new()
        .insert_resource(Msaa { samples: 4 })
        .insert_resource(ClearColor(Color::DARK_GRAY))
        .add_plugins(DefaultPlugins)
        .add_plugin(WorldInspectorPlugin::new())
        .add_plugin(in_menu::InMenuStatePlugin)
        .add_plugin(in_game::InGameStatePlugin)
        .add_startup_system(startup)
        .run();
}

fn startup(mut commands: Commands) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    commands.spawn_bundle(UiCameraBundle::default());
}


