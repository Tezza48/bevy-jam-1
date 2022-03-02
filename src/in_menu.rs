use bevy::{prelude::*, math::Rect};

use crate::app_state::*;

pub struct InMenuStatePlugin;
impl Plugin for InMenuStatePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_state(AppState::Menu)
            .add_system_set(
                SystemSet::on_exit(AppState::Menu)
                .with_system(on_exit)
            )
            .add_system_set(
                SystemSet::on_update(AppState::Menu)
                    .with_system(play_button)
            )

            .add_system_set(SystemSet::on_enter(AppState::Menu).with_system(on_enter));
    }
}

#[derive(Component)]
pub struct Cleanup;

fn on_enter(mut commands: Commands, asset_server: Res<AssetServer>) {
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
}

fn play_button(
    mut interaction_query: Query<(&Interaction, &mut UiColor), With<Button>>,
    mut state: ResMut<State<AppState>>,
) {
    for (interaction, mut color) in interaction_query.iter_mut() {
        match interaction {
            Interaction::Clicked => {
                *color = UiColor(Color::rgb(0.3, 0.5, 0.6));

                state.set(AppState::InGame).unwrap();
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

fn on_exit(
    mut commands: Commands,
    query: Query<Entity, With<Cleanup>>,
) {
    for e in query.iter() {
        commands.entity(e).despawn_recursive();
    }
}