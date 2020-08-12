use crate::components::*;
use amethyst::ecs::{
    join::Join,
    prelude::{ReadStorage, System, WriteStorage},
    Entities, Entity,
};
use std::ops::{Deref, DerefMut};
use amethyst::core::math;
use math::geometry::Translation;
use core::f32::consts::PI;

pub type Vector2 = amethyst::core::math::base::Vector2<f32>;

const TAU: f32 = 2.0 * PI;

pub struct Idle;

impl<'a> System<'a> for Idle {
    type SystemData = (
        Entities<'a>,
        ReadStorage<'a, Station>,
        WriteStorage<'a, ShipBehaviour>,
    );

    fn run(&mut self, (entities, station, mut behaviour): Self::SystemData) {
        use rand::seq::IteratorRandom;
        let mut rng = rand::thread_rng();
        let stations: Vec<Entity> = (&entities, &station).join().map(|(e, _)| e).collect();

        for behaviour in (&mut behaviour).join() {
            if let ShipBehaviour::Idle = behaviour {
                let station = stations.iter().choose(&mut rng).unwrap();
                *behaviour = ShipBehaviour::FlyTo(*station);
            }
        }
    }
}

pub struct FlyTo;

// Compute the score of a burn to a specific amount of rotation in seconds, smaller is better.
// Also compute the strength to burn with at the start of that burn.
fn rotation_burn_(target: f32, avel: f32, max_aaccel: f32) -> (f32, f32) {
    if target * avel > 0.0 {
        // target will be in the direction we're going, extra burn at the start while we can still
        // use our momentum.
        let extra_time = (2.0 * (-avel.abs() + (max_aaccel * target.abs() + avel.powi(2)).sqrt())) / max_aaccel;
        (extra_time, target.signum())
    } else {
        // target will lie in the opposite direction, extra burn at the end after we killed the
        // opposite momentum.
        let extra_time = (4.0 * target.abs() / max_aaccel).sqrt();
        (extra_time, target.signum())
    }
}

// Target is an angle relative to our heading in radians, avel is our angular velocity in radians
// per second, max_aaccel is our maximum angular acceleration in radians per second squared.
fn rotation_burn(target: f32, avel: f32, max_aaccel: f32) -> f32 {
    // Find the fastest way to reach the target rotation, which may be left or right.
    let braking_offset = avel.abs() * avel / max_aaccel / 2.0;
    // How far to the target rotation if we burned to zero velocity immediately.
    let adjusted_target = (target - braking_offset).rem_euclid(TAU);
    dbg!(braking_offset, adjusted_target);
    let (left_score, left_power) = rotation_burn_(adjusted_target - TAU, avel, max_aaccel);
    let (right_score, right_power) = rotation_burn_(adjusted_target, avel, max_aaccel);
    dbg!(left_score, right_score);
    if left_score > right_score {
        right_power
    } else {
        left_power
    }
}

fn angle_from_vec(v: Vector2) -> f32 {
    v.x.atan2(v.y)
}

impl<'a> System<'a> for FlyTo {
    type SystemData = (
        ReadStorage<'a, Angle>,
        ReadStorage<'a, Position>,
        WriteStorage<'a, ShipBehaviour>,
        WriteStorage<'a, Velocity>,
        WriteStorage<'a, AngularMomentum>,
    );

    fn run(&mut self, (angle, pos, mut behaviour, mut vel, mut avel): Self::SystemData) {
        for (&angle, &our_pos, behaviour, vel, avel) in (&angle, &pos, &mut behaviour, &mut vel, &mut avel).join() {
            if let ShipBehaviour::FlyTo(target) = behaviour {
                if let Some(&target_pos) = pos.get(*target) {
                    let vel = vel.deref_mut();
                    let avel = avel.deref_mut();
                    let max_accel = 0.005;
                    let max_aaccel = 0.0002;
                    let to_target = *target_pos.deref() - *our_pos.deref();
                    let heading_vec = Vector2::new(angle.sin(), angle.cos());
                    let course_angle = angle_from_vec(vel.vector);
                    let heading_angle = angle.deref();
                    let bearing_angle = angle_from_vec(to_target);
                    let adjust = (bearing_angle - course_angle + TAU / 2.0).rem_euclid(TAU) - TAU / 2.0;
                    let adjust = if adjust.abs() < TAU / 4.0 {
                        adjust
                    } else {
                        TAU / 2.0 - adjust
                    };

                    let target_angle = bearing_angle + adjust;
                    //let target_angle = bearing_angle;
                    let rotation_power = rotation_burn(target_angle - heading_angle, *avel, max_aaccel);
                    dbg!(course_angle, heading_angle, bearing_angle, target_angle, target_angle - heading_angle, *avel, rotation_power);

                    if to_target.norm_squared() < 100.0 /* && vel.vector.norm_squared() < 0.0000001 && *avel * *avel < 0.0000001 */ {
                        *behaviour = ShipBehaviour::Idle;
                        return;
                    };

                    let left = rotation_power.max(0.0);
                    let right = (-rotation_power).max(0.0);

                    // Compute acceleration.
                    let accel = heading_vec * (left + right) * max_accel;
                    let angle_accel = (left - right) * max_aaccel;

                    *vel = Translation::from(vel.vector + accel);
                    *avel += angle_accel;
                }
            }
        }
    }
}
