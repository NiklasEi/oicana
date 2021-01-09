use crate::map::Map;
use bevy::prelude::*;
use bevy::utils::Instant;
use bevy_prototype_lyon::prelude::*;
use rand::distributions::Standard;
use rand::prelude::*;
use std::f32::consts::{FRAC_PI_6, PI};

pub struct EnemiesPlugin;

impl Plugin for EnemiesPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_resource(WaveState {
            last_spawn: Instant::now(),
        })
        .add_system(spawn_enemies.system())
        .add_system(update_enemies.system());
    }
}

struct WaveState {
    pub last_spawn: Instant,
}

struct Enemy {
    current_waypoint_index: usize,
    form: EnemyForm,
    color: EnemyColor,
}

enum EnemyForm {
    Circle,
    Triangle,
    Quadratic,
}

impl Distribution<EnemyForm> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> EnemyForm {
        match rng.gen_range(0..3) {
            0 => EnemyForm::Circle,
            1 => EnemyForm::Triangle,
            _ => EnemyForm::Quadratic,
        }
    }
}

#[derive(Clone)]
enum EnemyColor {
    Red,
    Blue,
}

impl Distribution<EnemyColor> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> EnemyColor {
        match rng.gen_range(0..2) {
            0 => EnemyColor::Red,
            _ => EnemyColor::Blue,
        }
    }
}

impl EnemyColor {
    pub fn get_color_material_handle(
        self,
        mut materials: &mut ResMut<Assets<ColorMaterial>>,
    ) -> Handle<ColorMaterial> {
        match self {
            EnemyColor::Blue => materials.add(Color::rgb(0.1, 0.4, 0.5).into()),
            EnemyColor::Red => materials.add(Color::rgb(0.8, 0.0, 0.0).into()),
        }
    }
}

fn spawn_enemies(
    commands: &mut Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    map: Res<Map>,
    time: Res<Time>,
    mut wave_state: ResMut<WaveState>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    if time.last_update().is_some()
        && time
            .last_update()
            .unwrap()
            .duration_since(wave_state.last_spawn)
            .as_secs_f32()
            < 1.
    {
        return;
    } else if time.last_update().is_some() {
        wave_state.last_spawn = time.last_update().unwrap();
    }

    let form: EnemyForm = random();
    let color: EnemyColor = random();
    match form {
        EnemyForm::Circle => {
            create_circle_enemy(commands, &mut materials, color, &map, &mut meshes)
        }
        EnemyForm::Quadratic => {
            create_quadratic_enemy(commands, &mut materials, color, &map, &mut meshes)
        }
        EnemyForm::Triangle => {
            create_triangle_enemy(commands, &mut materials, color, &map, &mut meshes)
        }
    }
}

fn create_circle_enemy(
    commands: &mut Commands,
    mut materials: &mut ResMut<Assets<ColorMaterial>>,
    color: EnemyColor,
    map: &Res<Map>,
    meshes: &mut ResMut<Assets<Mesh>>,
) {
    let mut builder = PathBuilder::new();
    builder.arc(point(0.000001, 0.000001), 10., 10., 2. * PI, 0.1);
    let path = builder.build();
    commands
        .spawn(path.fill(
            color.clone().get_color_material_handle(&mut materials),
            meshes,
            Vec3::new(map.spawn.x, map.spawn.y, 0.),
            &FillOptions::default(),
        ))
        .with(Enemy {
            current_waypoint_index: 0,
            form: EnemyForm::Circle,
            color,
        });
}

fn create_triangle_enemy(
    commands: &mut Commands,
    mut materials: &mut ResMut<Assets<ColorMaterial>>,
    color: EnemyColor,
    map: &Res<Map>,
    meshes: &mut ResMut<Assets<Mesh>>,
) {
    let mut builder = PathBuilder::new();
    builder.move_to(point(-5., 9.));
    builder.line_to(point(-5., -9.));
    builder.line_to(point(10., 0.));
    builder.line_to(point(-5., 9.));
    let path = builder.build();
    commands
        .spawn(path.fill(
            color.clone().get_color_material_handle(&mut materials),
            meshes,
            Vec3::new(map.spawn.x, map.spawn.y, 0.),
            &FillOptions::default(),
        ))
        .with(Enemy {
            current_waypoint_index: 0,
            form: EnemyForm::Triangle,
            color,
        });
}

fn create_quadratic_enemy(
    commands: &mut Commands,
    mut materials: &mut ResMut<Assets<ColorMaterial>>,
    color: EnemyColor,
    map: &Res<Map>,
    meshes: &mut ResMut<Assets<Mesh>>,
) {
    let mut builder = PathBuilder::new();
    builder.move_to(point(-9., 9.));
    builder.line_to(point(-9., -9.));
    builder.line_to(point(9., -9.));
    builder.line_to(point(9., 9.));
    builder.line_to(point(-9., 9.));
    let path = builder.build();
    commands
        .spawn(path.fill(
            color.clone().get_color_material_handle(&mut materials),
            meshes,
            Vec3::new(map.spawn.x, map.spawn.y, 0.),
            &FillOptions::default(),
        ))
        .with(Enemy {
            current_waypoint_index: 0,
            form: EnemyForm::Quadratic,
            color,
        });
}

fn update_enemies(
    time: Res<Time>,
    map: Res<Map>,
    mut enemy_query: Query<(&mut Enemy, &mut Transform)>,
) {
    let delta = time.delta().as_millis() as f32;
    let speed = 0.1;
    for (mut enemy, mut transform) in enemy_query.iter_mut() {
        if enemy.current_waypoint_index >= map.waypoints.len() {
            continue;
        }
        let destination = map.waypoints.get(enemy.current_waypoint_index).unwrap();
        let distance = Vec3::new(destination.x, destination.y, 0.) - transform.translation;
        if distance == Vec3::zero() {
            enemy.current_waypoint_index += 1;
            continue;
        }
        let movement = distance.normalize() * delta * speed;
        if movement.length() > distance.length() {
            transform.translation = Vec3::new(destination.x, destination.y, 0.);
            enemy.current_waypoint_index += 1;
        } else {
            transform.translation += movement;
        }
    }
}