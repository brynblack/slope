use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

#[derive(Component)]
struct Player;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        // .add_plugin(RapierDebugRenderPlugin::default())
        .add_startup_system(setup)
        .add_system(follow_player)
        .add_system(move_player)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 0.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..Default::default()
    });

    commands
        .spawn(Collider::cuboid(100.0, 0.1, 100.0))
        .insert(PbrBundle {
            mesh: meshes
                .add(Mesh::from(shape::Plane {
                    size: 200.0,
                    subdivisions: 1,
                }))
                .into(),
            material: materials
                .add(StandardMaterial {
                    base_color: Color::hex("FFFFFF").unwrap(),
                    perceptual_roughness: 0.5,
                    ..default()
                })
                .into(),
            ..default()
        })
        .insert(TransformBundle::from(Transform::from_xyz(0.0, -2.0, 0.0)));

    commands
        .spawn(RigidBody::Dynamic)
        .insert(Collider::ball(0.5))
        .insert(Restitution::coefficient(0.7))
        .insert(KinematicCharacterController::default())
        .insert(Velocity {
            linvel: Vec3::new(0.0, 0.0, 0.0),
            angvel: Vec3::new(0.0, 0.0, 0.0),
        })
        .insert(PbrBundle {
            mesh: meshes
                .add(Mesh::from(shape::UVSphere {
                    radius: 0.5,
                    sectors: 16,
                    stacks: 16,
                }))
                .into(),
            material: materials
                .add(StandardMaterial {
                    base_color: Color::hex("00FF00").unwrap(),
                    ..default()
                })
                .into(),
            ..default()
        })
        .insert(TransformBundle::from(Transform::from_xyz(0.0, 4.0, 0.0)))
        .insert(Player);

    commands.spawn(PointLightBundle {
        transform: Transform::from_xyz(50.0, 50.0, 50.0),
        point_light: PointLight {
            intensity: 600000.,
            range: 100.,
            ..default()
        },
        ..default()
    });
}

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
    *camera = camera.looking_at(
        player.translation,
        Vec3 {
            x: 0.0,
            y: 1.0,
            z: 0.0,
        },
    );
}

fn move_player(
    mut player: Query<&mut Velocity, With<Player>>,
    keyboard_input: Res<Input<KeyCode>>,
) {
    let mut player = player.get_single_mut().unwrap();

    if keyboard_input.pressed(KeyCode::S) {
        player.linvel.x += 0.01;
    };

    if keyboard_input.pressed(KeyCode::W) {
        player.linvel.z -= 0.01;
    };

    if keyboard_input.pressed(KeyCode::A) {
        player.linvel.x -= 0.01;
    };

    if keyboard_input.pressed(KeyCode::R) {
        player.linvel.z += 0.01;
    };
}
