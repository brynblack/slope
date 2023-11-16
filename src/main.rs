use std::f32::consts::PI;

use bevy::{
    asset::LoadState,
    core_pipeline::Skybox,
    prelude::*,
    render::render_resource::{TextureViewDescriptor, TextureViewDimension},
    window::WindowMode,
};
use bevy_rapier3d::prelude::*;

#[derive(Component)]
struct Player;

#[derive(Resource)]
struct Cubemap {
    is_loaded: bool,
    image_handle: Handle<Image>,
}

#[derive(Debug, Default, Clone, Eq, PartialEq, Hash, States)]
enum AppState {
    Generated,
    #[default]
    Idle,
}

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Slope".to_string(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            RapierPhysicsPlugin::<NoUserData>::default(),
        ))
        .insert_resource(Msaa::default())
        .add_systems(Startup, setup_world)
        .add_systems(
            Update,
            (
                correct_skybox,
                follow_player,
                handle_input,
                generate_floor.run_if(in_state(AppState::Idle)),
                check_distance,
            ),
        )
        .add_event::<GenerateMapEvent>()
        .add_state::<AppState>()
        .run();
}

/// Sets up the environment.
fn setup_world(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut ev_generatemap: EventWriter<GenerateMapEvent>,
) {
    // Load the skybox.
    let skybox_handle = asset_server.load("skybox.png");

    commands.insert_resource(Cubemap {
        is_loaded: false,
        image_handle: skybox_handle.clone(),
    });

    // Spawns the camera and skybox.
    commands.spawn((Camera3dBundle::default(), Skybox(skybox_handle.clone())));

    // Spawn the player as a ball.
    commands
        .spawn(RigidBody::Dynamic)
        .insert(Collider::ball(0.5))
        .insert(Restitution::coefficient(0.7))
        .insert(Damping {
            linear_damping: 0.5,
            angular_damping: 1.0,
        })
        .insert(GravityScale(10.0))
        .insert(Velocity::zero())
        .insert(PbrBundle {
            mesh: meshes
                .add(Mesh::from(shape::UVSphere {
                    radius: 0.5,
                    sectors: 32,
                    stacks: 32,
                }))
                .into(),
            material: materials
                .add(StandardMaterial {
                    base_color: Color::hex("FF0000").unwrap(),
                    perceptual_roughness: 0.,
                    ..default()
                })
                .into(),
            ..default()
        })
        .insert(TransformBundle::from(Transform::from_xyz(0.0, 4.0, 0.0)))
        .insert(Player);

    // Spawn a light that acts as sunlight.
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 10000.,
            shadows_enabled: true,
            ..Default::default()
        },
        transform: Transform {
            rotation: Quat::from_scaled_axis(Vec3::new(-PI / 2., 0., 0.)),
            ..default()
        },
        ..Default::default()
    });

    ev_generatemap.send(GenerateMapEvent);
}

/// Converts the input skybox to a cubemap.
fn correct_skybox(
    asset_server: Res<AssetServer>,
    mut images: ResMut<Assets<Image>>,
    mut cubemap: ResMut<Cubemap>,
    mut skyboxes: Query<&mut Skybox>,
) {
    if !cubemap.is_loaded
        && asset_server.get_load_state(cubemap.image_handle.clone_weak()) == Some(LoadState::Loaded)
    {
        let image = images.get_mut(&cubemap.image_handle).unwrap();
        if image.texture_descriptor.array_layer_count() == 1 {
            image.reinterpret_stacked_2d_as_array(
                image.texture_descriptor.size.height / image.texture_descriptor.size.width,
            );
            image.texture_view_descriptor = Some(TextureViewDescriptor {
                dimension: Some(TextureViewDimension::Cube),
                ..default()
            });
        }

        for mut skybox in &mut skyboxes {
            skybox.0 = cubemap.image_handle.clone();
        }

        cubemap.is_loaded = true;
    }
}

/// Locks the camera to the position of the player.
fn follow_player(
    mut camera: Query<&mut Transform, With<Camera>>,
    player: Query<&Transform, (With<Player>, Without<Camera>)>,
) {
    let mut camera = camera.get_single_mut().unwrap();
    let player = player.get_single().unwrap();

    // Lock the position of the camera to the player
    camera.translation.x = player.translation.x + 0.;
    camera.translation.y = player.translation.y + 5.;
    camera.translation.z = player.translation.z + 10.;

    // Rotate the camera to look at the ball
    *camera = camera.looking_at(player.translation, Vec3::Y);
}

/// A handler for user input.
fn handle_input(
    mut player: Query<&mut Velocity, With<Player>>,
    mut windows: Query<&mut Window>,
    keyboard_input: Res<Input<KeyCode>>,
) {
    let mut player = player.get_single_mut().unwrap();
    let mut window = windows.single_mut();

    if keyboard_input.pressed(KeyCode::A) {
        player.linvel.x -= 0.1;
    };

    if keyboard_input.pressed(KeyCode::S) {
        player.linvel.x += 0.1;
    };

    if keyboard_input.just_pressed(KeyCode::F11) {
        window.mode = match window.mode {
            WindowMode::BorderlessFullscreen => WindowMode::Windowed,
            _ => WindowMode::BorderlessFullscreen,
        }
    };
}

/// Generates the floor that the player will roll down.
fn generate_floor(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut ev_generatemap: EventReader<GenerateMapEvent>,
    player: Query<&Transform, (With<Player>, Without<Camera>)>,
) {
    for _ev in ev_generatemap.read() {
        let translation = player.single().translation;
        commands
            .spawn(Collider::cuboid(5.0, 0.5, 25.0))
            .insert(PbrBundle {
                mesh: meshes
                    .add(Mesh::from(shape::Box {
                        min_x: -5.,
                        max_x: 5.,
                        min_y: -0.5,
                        max_y: 0.5,
                        min_z: -25.,
                        max_z: 25.,
                    }))
                    .into(),
                material: materials
                    .add(StandardMaterial {
                        base_color: Color::hex("FFFFFF").unwrap(),
                        perceptual_roughness: 1.,
                        ..default()
                    })
                    .into(),
                ..default()
            })
            .insert(TransformBundle::from(
                Transform::from_xyz(translation.x, translation.y - 2.0, translation.z)
                    .with_rotation(Quat::from_rotation_x(-PI / 8.)),
            ));
    }
}

#[derive(Event)]
struct GenerateMapEvent;

fn check_distance(
    mut ev_generatemap: EventWriter<GenerateMapEvent>,
    query: Query<&Transform, (With<Player>, Without<Camera>)>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    for transform in query.iter() {
        match transform.translation.z.ceil().abs() % 10. == 0. {
            true => {
                ev_generatemap.send(GenerateMapEvent);
                next_state.set(AppState::Generated);
            }
            false => {
                next_state.set(AppState::Idle);
            }
        }
    }
}
