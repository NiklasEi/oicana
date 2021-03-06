use crate::AppState;
use bevy::prelude::*;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.insert_resource(GameState::default())
            .init_resource::<ButtonMaterials>()
            .add_system_set(SystemSet::on_enter(AppState::InGame).with_system(init_life.system()))
            .add_system_set(
                SystemSet::on_update(AppState::InGame)
                    .with_system(update_game_state.system())
                    .with_system(retry_system.system())
                    .with_system(click_retry_button.system()),
            );
    }
}

struct ButtonMaterials {
    normal: Handle<ColorMaterial>,
    hovered: Handle<ColorMaterial>,
}

impl FromWorld for ButtonMaterials {
    fn from_world(world: &mut World) -> Self {
        let mut materials = world.get_resource_mut::<Assets<ColorMaterial>>().unwrap();
        ButtonMaterials {
            normal: materials.add(Color::rgb(0.15, 0.15, 0.15).into()),
            hovered: materials.add(Color::rgb(0.25, 0.25, 0.25).into()),
        }
    }
}

struct RetryButton;

struct HealthText;

struct ScoreText;

pub struct GameState {
    pub health: usize,
    pub score: usize,
    pub enemy_health: i32,
}

impl Default for GameState {
    fn default() -> Self {
        GameState {
            health: 20,
            score: 0,
            enemy_health: 1,
        }
    }
}

fn init_life(
    mut commands: Commands,
    asset_server: ResMut<AssetServer>,
    game_state: Res<GameState>,
    mut color_materials: ResMut<Assets<ColorMaterial>>,
) {
    let font = asset_server.load("fonts/FiraSans-Bold.ttf");
    let material = color_materials.add(Color::NONE.into());
    commands.spawn_bundle(UiCameraBundle::default());
    // root node
    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                position: Rect {
                    left: Val::Px(10.),
                    top: Val::Px(10.),
                    ..Default::default()
                },
                ..Default::default()
            },
            material,
            ..Default::default()
        })
        .with_children(|parent| {
            parent
                .spawn_bundle(TextBundle {
                    text: Text {
                        sections: vec![TextSection {
                            value: format!("Health: {}", game_state.health),
                            style: TextStyle {
                                font_size: 40.0,
                                color: Color::rgb(0.6, 0.6, 0.6),
                                font,
                            },
                        }],
                        alignment: Default::default(),
                    },
                    ..Default::default()
                })
                .insert(HealthText);
        });
    let font = asset_server.load("fonts/FiraSans-Bold.ttf");
    let material = color_materials.add(Color::NONE.into());
    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                position: Rect {
                    right: Val::Px(10.),
                    top: Val::Px(10.),
                    ..Default::default()
                },
                ..Default::default()
            },
            material,
            ..Default::default()
        })
        .with_children(|parent| {
            parent
                .spawn_bundle(TextBundle {
                    text: Text {
                        sections: vec![TextSection {
                            value: format!("Score: {}", game_state.score),
                            style: TextStyle {
                                font_size: 40.0,
                                color: Color::rgb(0.6, 0.6, 0.6),
                                font,
                            },
                        }],
                        alignment: Default::default(),
                    },
                    ..Default::default()
                })
                .insert(ScoreText);
        });
}

fn update_game_state(
    game_state: Res<GameState>,
    mut health_query: Query<&mut Text, (With<HealthText>, Without<ScoreText>)>,
    mut score_query: Query<&mut Text, (With<ScoreText>, Without<HealthText>)>,
) {
    if game_state.is_changed() {
        for mut text in health_query.iter_mut() {
            text.sections.first_mut().unwrap().value = format!("Health: {}", game_state.health);
        }
        for mut text in score_query.iter_mut() {
            text.sections.first_mut().unwrap().value = format!("Score: {}", game_state.score);
        }
    }
}

fn retry_system(
    mut commands: Commands,
    asset_server: ResMut<AssetServer>,
    game_state: Res<GameState>,
    button_materials: Res<ButtonMaterials>,
) {
    if game_state.is_changed() && game_state.health < 1 {
        commands
            .spawn_bundle(ButtonBundle {
                style: Style {
                    size: Size::new(Val::Px(150.0), Val::Px(65.0)),
                    margin: Rect::all(Val::Auto),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..Default::default()
                },
                material: button_materials.normal.clone(),
                ..Default::default()
            })
            .insert(RetryButton)
            .with_children(|parent| {
                parent.spawn_bundle(TextBundle {
                    text: Text {
                        sections: vec![TextSection {
                            value: "Restart".to_string(),
                            style: TextStyle {
                                font_size: 40.0,
                                color: Color::rgb(0.9, 0.9, 0.9),
                                font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                            },
                        }],
                        alignment: Default::default(),
                    },
                    ..Default::default()
                });
            });
    }
}

fn click_retry_button(
    mut commands: Commands,
    button_materials: Res<ButtonMaterials>,
    mut state: ResMut<State<AppState>>,
    mut game_state: ResMut<GameState>,
    mut interaction_query: Query<
        (Entity, &Interaction, &mut Handle<ColorMaterial>, &Children),
        With<Button>,
    >,
    text_query: Query<Entity, With<Text>>,
) {
    for (button, interaction, mut material, children) in interaction_query.iter_mut() {
        let text = text_query.get(children[0]).unwrap();
        match *interaction {
            Interaction::Clicked => {
                *game_state = GameState::default();
                commands.entity(button).despawn();
                commands.entity(text).despawn();
                state.set(AppState::Restart).unwrap();
            }
            Interaction::Hovered => {
                *material = button_materials.hovered.clone();
            }
            Interaction::None => {
                *material = button_materials.normal.clone();
            }
        }
    }
}
