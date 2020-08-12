pub mod behaviour;

use amethyst::{
    core::{math, transform::Transform},
    ecs::{
        join::Join,
        prelude::{ReadExpect, ReadStorage, System, WriteExpect, WriteStorage},
    },
    renderer::Camera,
    ui::UiTransform,
    window::ScreenDimensions,
};
use std::ops::{Deref, DerefMut};

use crate::components::*;
use crate::resources::*;

pub struct IncrementTime;

impl<'a> System<'a> for IncrementTime {
    type SystemData = WriteExpect<'a, CurrentTime>;

    fn run(&mut self, mut time: Self::SystemData) {
        time.0 += Time(1);
    }
}

pub struct Movement;

impl<'a> System<'a> for Movement {
    type SystemData = (ReadStorage<'a, Velocity>, WriteStorage<'a, Position>);

    fn run(&mut self, (vel, mut pos): Self::SystemData) {
        for (vel, pos) in (&vel, &mut pos).join() {
            *pos.deref_mut() = vel.deref().transform_point(pos.deref());
        }
    }
}

pub struct Rotation;

impl<'a> System<'a> for Rotation {
    type SystemData = (ReadStorage<'a, AngularMomentum>, WriteStorage<'a, Angle>);

    fn run(&mut self, (momentum, mut angle): Self::SystemData) {
        for (momentum, angle) in (&momentum, &mut angle).join() {
            *angle.deref_mut() += *momentum.deref();
        }
    }
}

pub struct DerivePositionalTransform;

impl<'a> System<'a> for DerivePositionalTransform {
    type SystemData = (
        ReadStorage<'a, Position>,
        ReadStorage<'a, Trader>,
        ReadStorage<'a, Station>,
        WriteStorage<'a, Transform>,
    );

    fn run(&mut self, (pos, trader, station, mut transform): Self::SystemData) {
        for (pos, trader, station, transform) in
            (&pos, (&trader).maybe(), (&station).maybe(), &mut transform).join()
        {
            transform.set_scale([1.0, 1.0, 1.0].into());

            // Vary z-level based on type
            let z = if trader.is_some() {
                0.2
            } else if station.is_some() {
                0.1
            } else {
                0.0
            };

            transform.set_translation_xyz(pos.x.into(), pos.y.into(), z);
        }
    }
}

pub struct DeriveRotationalTransform;

impl<'a> System<'a> for DeriveRotationalTransform {
    type SystemData = (ReadStorage<'a, Angle>, WriteStorage<'a, Transform>);

    fn run(&mut self, (angle, mut transform): Self::SystemData) {
        for (angle, transform) in (&angle, &mut transform).join() {
            transform.set_rotation_2d(-*angle.deref());
        }
    }
}

pub struct CameraControl;

impl<'a> System<'a> for CameraControl {
    type SystemData = (
        WriteExpect<'a, CameraState>,
        ReadExpect<'a, ScreenDimensions>,
        ReadStorage<'a, Position>,
        WriteStorage<'a, Camera>,
        WriteStorage<'a, Transform>,
    );

    fn run(&mut self, (camera_state, size, pos, mut camera, mut transform): Self::SystemData) {
        let zoom = camera_state.zoom;

        for (camera, transform) in (&mut camera, &mut transform).join() {
            // Update the camera size per zoom level.
            *camera = Camera::standard_2d(size.width() / zoom, size.height() / zoom);

            match &camera_state.behaviour {
                CameraBehaviour::Static => (),
                CameraBehaviour::Follow(target) => {
                    if let Some(pos) = pos.get(*target) {
                        transform.set_translation_xyz(pos.x, pos.y, 1.0);
                    }
                }
                CameraBehaviour::Pan(translation) => {
                    let factor = f32::powf(2., 1. / zoom);
                    transform.append_translation_xyz(
                        translation.x * factor,
                        translation.y * factor,
                        0.,
                    );
                }
            }
        }
    }
}

pub struct UiRelativePositioning;

impl<'a> System<'a> for UiRelativePositioning {
    type SystemData = (
        ReadExpect<'a, ScreenDimensions>,
        ReadStorage<'a, Camera>,
        ReadStorage<'a, Transform>,
        ReadStorage<'a, Hitbox>,
        ReadStorage<'a, UiRelative>,
        WriteStorage<'a, UiTransform>,
    );

    fn run(
        &mut self,
        (size, camera, transform, hitbox, ui_relative, mut ui_transform): Self::SystemData,
    ) {
        let size = size.diagonal();

        for (camera, camera_transform) in (&camera, &transform).join() {
            let projection = camera.projection();
            for (ui_relative, ui_transform) in (&ui_relative, &mut ui_transform).join() {
                let UiRelative(parent) = ui_relative;
                if let Some(parent_transform) = transform.get(*parent) {
                    let middle_world = (*parent_transform).clone();
                    let middle_screen = projection.world_to_screen(
                        math::Point::from(*parent_transform.translation()),
                        size,
                        camera_transform,
                    );
                    ui_transform.local_x = middle_screen.x;
                    // TODO fix this inversion.
                    ui_transform.local_y = size.y - middle_screen.y;

                    if let Some(hitbox) = hitbox.get(*parent) {
                        let topright_world = middle_world
                            .clone()
                            .prepend_translation(math::Vector3::new(
                                hitbox.x / 2.,
                                hitbox.y / 2.,
                                0.,
                            ))
                            .clone();
                        let topright_screen = projection.world_to_screen(
                            math::Point::from(*topright_world.translation()),
                            size,
                            camera_transform,
                        );

                        let hwidth = topright_screen.x - middle_screen.x;
                        let hheight = middle_screen.y - topright_screen.y;

                        ui_transform.width = hwidth * 2.;
                        ui_transform.height = hheight * 2.;
                    }
                }
            }
        }
    }
}
