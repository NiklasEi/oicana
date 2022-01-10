use crate::enemies::{Enemy, EnemyLabels, Health, Tameable};
use crate::AppState;
use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;
use bevy_prototype_lyon::shapes::Circle;

pub struct BulletPlugin;

impl Plugin for BulletPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            SystemSet::on_update(AppState::InGame)
                .with_system(update_bullets.system().label(EnemyLabels::Damage)),
        )
        .add_system_set(
            SystemSet::on_exit(AppState::InGame).with_system(break_down_bullets.system()),
        );
    }
}

#[derive(Component)]
pub struct Bullet {
    pub damage: i32,
    pub speed: f32,
}

fn update_bullets(
    mut commands: Commands,
    mut bullet_query: Query<(Entity, &Bullet, &mut Transform)>,
    mut enemy_query: Query<
        (&Transform, &mut Health, &mut Enemy),
        (Without<Tameable>, Without<Bullet>),
    >,
    time: Res<Time>,
) {
    let delta = time.delta().as_secs_f32();
    for (target_transform, mut health, mut enemy) in enemy_query.iter_mut() {
        let mut to_remove = vec![];
        for bullet_id in enemy.bullets.iter() {
            let bullet = bullet_query.get_mut(*bullet_id);
            if let Ok((entity, bullet, mut transform)) = bullet {
                let distance = target_transform.translation - transform.translation;
                if distance.length() < bullet.speed * delta {
                    health.value -= bullet.damage;
                    commands.entity(entity).despawn();
                    to_remove.push(bullet_id.clone());
                } else {
                    let movement = distance.normalize() * bullet.speed * delta;
                    transform.translation += movement;
                }
            } else {
                to_remove.push(bullet_id.clone());
            }
        }
        for bullet_id in to_remove.drain(..) {
            let position = enemy
                .bullets
                .iter()
                .position(|&id| id == bullet_id)
                .unwrap();
            enemy.bullets.remove(position);
        }
    }
}

pub fn spawn_bullet(commands: &mut Commands, bullet: Bullet, translation: Vec3) -> Entity {
    commands
        .spawn_bundle(GeometryBuilder::build_as(
            &Circle {
                radius: 3.,
                center: Vec2::splat(0.)
            },
            DrawMode::Fill(FillMode::color(Color::BLACK)),
            Transform::from_translation(translation),
        ))
        .insert(bullet)
        .id()
}

fn break_down_bullets(mut commands: Commands, bullets_query: Query<Entity, With<Bullet>>) {
    for entity in bullets_query.iter() {
        commands.entity(entity).despawn();
    }
}
