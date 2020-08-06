pub mod entities;
pub mod game;
pub mod resources;
pub mod systems;

use amethyst::{
    core::{frame_limiter::FrameRateLimitStrategy, transform::TransformBundle},
    prelude::*,
    renderer::{
        plugins::{RenderFlat2D, RenderToWindow},
        types::DefaultBackend,
        RenderingBundle,
    },
    utils::application_root_dir,
};

use crate::game::Game;
use std::time::Duration;

fn main() -> amethyst::Result<()> {
    amethyst::start_logger(Default::default());

    let app_root = application_root_dir()?;
    let display_config_path = app_root.join("config").join("display.ron");

    let game_data = GameDataBuilder::default()
        .with_bundle(
            RenderingBundle::<DefaultBackend>::new()
                // The RenderToWindow plugin provides all the scaffolding for opening a window and drawing on it
                .with_plugin(
                    RenderToWindow::from_config_path(display_config_path)?
                        .with_clear([0.0, 0.0, 0.0, 1.0]),
                )
                // RenderFlat2D plugin is used to render entities with a `SpriteRender` component.
                .with_plugin(RenderFlat2D::default()),
        )?
        .with(systems::behaviour::Idle, "behaviour_idle", &[])
        .with(systems::behaviour::FlyTo, "behaviour_fly_to", &[])
        .with(systems::Movement, "movement", &["behaviour_fly_to"])
        .with(systems::Rotation, "rotation", &["behaviour_fly_to"])
        .with(
            systems::DerivePositionalTransform,
            "derive_positional_transform",
            &["movement"],
        )
        .with(
            systems::DeriveRotationalTransform,
            "derive_rotational_transform",
            &["rotation"],
        )
        .with(systems::CameraControl, "camera_control", &["movement"])
        .with_bundle(TransformBundle::new())?;

    let assets_dir = app_root.join("assets");
    let mut game = Application::build(assets_dir, Game)?
        .with_frame_limit(
            FrameRateLimitStrategy::SleepAndYield(Duration::from_millis(10)),
            50,
        )
        .build(game_data)?;
    game.run();

    Ok(())
}
