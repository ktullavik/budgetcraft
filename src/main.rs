use bevy::{prelude::*, app::AppExit, window::{PresentMode, WindowResolution}};
use bevy_rapier3d::prelude::{RapierPhysicsPlugin, NoUserData};
use plugins::{camera::CameraPlugin, world::WorldPlugin, player::PlayerPlugin};

mod plugins;

pub const CHUNK_WIDTH: usize = 8;
pub const CHUNK_HEIGHT: usize = 256;
pub const CHUNK_VOL: usize = CHUNK_WIDTH * CHUNK_WIDTH * CHUNK_HEIGHT;
pub const RENDER_DISTANCE: i32 = 24;


#[derive(Default, Resource, Debug, Eq, PartialEq, States, Hash, Clone)]
enum GameState {
    Running,
    #[default]
    Stopped
}


fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()).set(WindowPlugin {
            primary_window: Some(Window {
                title: "BudgetCraft".to_string(),
                present_mode: PresentMode::AutoVsync,
                resolution: WindowResolution::new(1280.0, 720.0),
                ..default()
            }),
            ..default()
        }))

        // Init state before our own plugins.
        .init_state::<GameState>()

        .add_plugins(CameraPlugin)
        .add_plugins(GamePlugin)
        .add_plugins(menu_plugin)
        .add_systems(OnEnter(GameState::Stopped), main_menu_setup)
        .run();
}


pub struct GamePlugin;

impl Plugin for GamePlugin {

    fn build(&self, app: &mut App) {
        app
        .add_plugins(PlayerPlugin)
        .add_plugins(WorldPlugin)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default());
    }
}


const TEXT_COLOR: Color = Color::rgb(0.9, 0.9, 0.9);
const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::rgb(0.25, 0.25, 0.25);
const HOVERED_PRESSED_BUTTON: Color = Color::rgb(0.25, 0.65, 0.25);
const PRESSED_BUTTON: Color = Color::rgb(0.35, 0.75, 0.35);


    // All actions that can be triggered from a button click
#[derive(Component)]
    enum MenuButtonAction {
        Play,
        Quit,
    }


// Tag component used to tag entities added on the main menu screen
#[derive(Component)]
struct OnMainMenuScreen;



pub fn menu_plugin(app: &mut App) {
    app
        .add_systems(OnExit(GameState::Stopped), despawn_screen::<OnMainMenuScreen>)
        .add_systems(Update, (menu_action, button_system).run_if(in_state(GameState::Stopped)));
}



fn main_menu_setup(mut commands: Commands) {
    // Common style for all buttons on the screen
    let button_style = Style {
        width: Val::Px(250.0),
        height: Val::Px(65.0),
        margin: UiRect::all(Val::Px(20.0)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    };
    let button_text_style = TextStyle {
        font_size: 40.0,
        color: TEXT_COLOR,
        ..default()
    };

    commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                ..default()
            },
           OnMainMenuScreen,
        ))
        .with_children(|parent| {
            parent
                .spawn(NodeBundle {
                    style: Style {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    background_color: Color::CRIMSON.into(),
                    ..default()
                })
                .with_children(|parent| {
                    // Display the game name
                    parent.spawn(
                        TextBundle::from_section(
                            "BudgetCraft",
                            TextStyle {
                                font_size: 80.0,
                                color: TEXT_COLOR,
                                ..default()
                            },
                        )
                        .with_style(Style {
                            margin: UiRect::all(Val::Px(50.0)),
                            ..default()
                        }),
                    );

                    // Display buttons for each action available from the main menu:
                    // - new game
                    // - quit
                    parent
                        .spawn((
                            ButtonBundle {
                                style: button_style.clone(),
                                background_color: NORMAL_BUTTON.into(),
                                ..default()
                            },
                            MenuButtonAction::Play,
                        ))
                        .with_children(|parent| {
                            parent.spawn(TextBundle::from_section(
                                "Play",
                                button_text_style.clone(),
                            ));
                        });
                    parent
                        .spawn((
                            ButtonBundle {
                                style: button_style,
                                background_color: NORMAL_BUTTON.into(),
                                ..default()
                            },
                            MenuButtonAction::Quit,
                        ))
                        .with_children(|parent| {
                            parent.spawn(TextBundle::from_section("Quit", button_text_style));
                        });
                });
        });
}


fn menu_action(
    interaction_query: Query<(&Interaction, &MenuButtonAction), (Changed<Interaction>, With<Button>)>,
    mut app_exit_events: EventWriter<AppExit>,
    mut game_state: ResMut<NextState<GameState>>) {

    for (interaction, menu_button_action) in &interaction_query {
        if *interaction == Interaction::Pressed {
            match menu_button_action {
                MenuButtonAction::Quit => {
                    app_exit_events.send(AppExit);
                }
                MenuButtonAction::Play => {
                    game_state.set(GameState::Running);
                }
            }
        }
    }
}


// Tag component used to mark which setting is currently selected
#[derive(Component)]
struct SelectedOption;


fn button_system(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, Option<&SelectedOption>),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, mut color, selected) in &mut interaction_query {
        *color = match (*interaction, selected) {
            (Interaction::Pressed, _) | (Interaction::None, Some(_)) => PRESSED_BUTTON.into(),
            (Interaction::Hovered, Some(_)) => HOVERED_PRESSED_BUTTON.into(),
            (Interaction::Hovered, None) => HOVERED_BUTTON.into(),
            (Interaction::None, None) => NORMAL_BUTTON.into(),
        }
    }
}


// Generic system that takes a component as a parameter, and will despawn all entities with that component
fn despawn_screen<T: Component>(to_despawn: Query<Entity, With<T>>, mut commands: Commands) {
    for entity in &to_despawn {
        commands.entity(entity).despawn_recursive();
    }
}
