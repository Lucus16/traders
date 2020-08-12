use amethyst::{
    assets::{AssetStorage, Handle, Loader},
    core::transform::Transform,
    ecs::ReadStorage,
    input::{is_close_requested, is_key_down, InputEvent, ScrollDirection},
    prelude::*,
    renderer::{Camera, ImageFormat, SpriteSheet, SpriteSheetFormat, Texture},
    ui::{FontAsset, TtfFormat, UiEvent, UiEventType},
    winit::{MouseButton, VirtualKeyCode},
};
use log::info;

use crate::components::*;
use crate::resources::*;

pub const ARENA_HEIGHT: f32 = 1000.0;
pub const ARENA_WIDTH: f32 = 1000.0;

const DELTA: f32 = 0.00001;

fn initialise_camera(world: &mut World) {
    // Setup camera in a way that our screen covers whole arena and (0, 0) is in the bottom left.
    let mut transform = Transform::default();
    transform.set_translation_xyz(ARENA_WIDTH * 0.5, ARENA_HEIGHT * 0.5, 1.0);

    world
        .create_entity()
        .with(Camera::standard_2d(1024., 768.))
        .with(transform)
        .build();
}

fn load_sprite_sheet(world: &mut World) -> Handle<SpriteSheet> {
    let texture_handle = {
        let loader = world.read_resource::<Loader>();
        let texture_storage = world.read_resource::<AssetStorage<Texture>>();
        loader.load(
            "texture/ships.gif",
            ImageFormat::default(),
            (),
            &texture_storage,
        )
    };

    let loader = world.read_resource::<Loader>();
    let store = world.read_resource::<AssetStorage<SpriteSheet>>();
    loader.load(
        "texture/ships.ron",
        SpriteSheetFormat(texture_handle),
        (),
        &store,
    )
}

fn load_font(world: &mut World) -> Handle<FontAsset> {
    let loader = world.read_resource::<Loader>();
    let store = world.read_resource::<AssetStorage<FontAsset>>();

    loader.load("fonts/square.ttf", TtfFormat, (), &store)
}

pub struct Game;

impl SimpleState for Game {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let world = data.world;

        let spritesheet = load_sprite_sheet(world);
        world.insert(spritesheet);
        let font = load_font(world);
        world.insert(font);
        world.insert(CurrentTime::default());
        world.insert(Selection::default());
        world.insert(HoverSelectable::default());

        initialise_camera(world);

        world.register::<Position>();
        world.register::<Velocity>();
        world.register::<Trader>();
        world.register::<Station>();
        world.register::<ShipBehaviour>();
        world.register::<Parent>();
        world.register::<Hitbox>();
        world.register::<Cargo>();
        world.register::<FabricationModule>();
        world.register::<UiRelative>();
        world.register::<UiSelectable>();

        create_station(world, Position::new(Point2::new(800., 700.)));
        create_station(world, Position::new(Point2::new(300., 100.)));
        create_station(world, Position::new(Point2::new(200., 600.)));

        //create_trader(
        //    world,
        //    Position::new(Point2::new(900., 900.)),
        //    ShipBehaviour::Idle,
        //);

        let t1 = create_trader(
            world,
            Position::new(Point2::new(50., 50.)),
            ShipBehaviour::Idle,
        );

        let camera_state = CameraState {
            zoom: 1.0,
            behaviour: CameraBehaviour::Follow(t1),
        };

        world.insert(camera_state);
    }

    fn handle_event(
        &mut self,
        data: StateData<'_, GameData<'_, '_>>,
        event: StateEvent,
    ) -> SimpleTrans {
        let StateData { world, .. } = data;

        match &event {
            StateEvent::Window(event) => {
                if is_close_requested(&event) || is_key_down(&event, VirtualKeyCode::Escape) {
                    Trans::Quit
                } else {
                    Trans::None
                }
            }
            StateEvent::Ui(UiEvent { event_type, target }) => {
                let (selectable, relative): (ReadStorage<UiSelectable>, ReadStorage<UiRelative>) =
                    world.system_data();
                let mut hover_selectable = world.fetch_mut::<HoverSelectable>();

                match event_type {
                    UiEventType::HoverStart => {
                        if selectable.get(*target).is_some() {
                            if let Some(UiRelative(parent)) = relative.get(*target) {
                                hover_selectable.0.insert(*parent);
                            }
                        }
                    }
                    UiEventType::HoverStop => {
                        if selectable.get(*target).is_some() {
                            if let Some(UiRelative(parent)) = relative.get(*target) {
                                hover_selectable.0.remove(parent);
                            }
                        }
                    }
                    _ => info!(
                        "[HANDLE_EVENT] You just interacted with a ui element: {:?}",
                        event_type
                    ),
                }
                Trans::None
            }
            StateEvent::Input(input) => {
                use InputEvent::*;
                match input {
                    MouseWheelMoved(dir) => {
                        let dir = match *dir {
                            ScrollDirection::ScrollUp => -0.1,
                            ScrollDirection::ScrollDown => 0.1,
                            _ => 0.0,
                        };
                        let mut camera_state = world.fetch_mut::<CameraState>();
                        camera_state.zoom = f32::min(f32::max(0.1, camera_state.zoom + dir), 2.0);
                    }
                    MouseButtonPressed(MouseButton::Left) => {
                        let hover_selectable = world.fetch::<HoverSelectable>();
                        let mut camera_state = world.fetch_mut::<CameraState>();

                        if hover_selectable.0.len() == 1 {
                            if let Some(target) = hover_selectable.0.iter().next() {
                                camera_state.behaviour = CameraBehaviour::Follow(*target);
                            }
                        }

                        info!("Press detected: {:?} {:?}.", input, hover_selectable.0);
                    }
                    MouseMoved { .. }
                    | CursorMoved { .. }
                    | KeyTyped(_)
                    | ButtonPressed(_)
                    | ButtonReleased(_)
                    | MouseButtonReleased(_) => (),
                    KeyPressed { key_code, .. } => {
                        let mut camera_state = world.fetch_mut::<CameraState>();
                        let mut t = match camera_state.behaviour {
                            CameraBehaviour::Pan(t) => t,
                            _ => Translation2::new(0., 0.),
                        };

                        match key_code {
                            VirtualKeyCode::W => t.y = 1.,
                            VirtualKeyCode::A => t.x = -1.,
                            VirtualKeyCode::S => t.y = -1.,
                            VirtualKeyCode::D => t.x = 1.,
                            _ => (),
                        }

                        if nalgebra_glm::length(&t.vector) > DELTA {
                            camera_state.behaviour = CameraBehaviour::Pan(t);
                        }
                    }
                    KeyReleased { key_code, .. } => {
                        let mut camera_state = world.fetch_mut::<CameraState>();

                        match camera_state.behaviour {
                            CameraBehaviour::Pan(mut t) => {
                                match key_code {
                                    VirtualKeyCode::W => t.y = 0.,
                                    VirtualKeyCode::A => t.x = 0.,
                                    VirtualKeyCode::S => t.y = 0.,
                                    VirtualKeyCode::D => t.x = 0.,
                                    _ => (),
                                }

                                if nalgebra_glm::length(&t.vector) > DELTA {
                                    camera_state.behaviour = CameraBehaviour::Pan(t);
                                } else {
                                    camera_state.behaviour = CameraBehaviour::Static;
                                }
                            }
                            _ => (),
                        }
                    }
                    _ => {
                        info!("Input Event detected: {:?}.", input);
                    }
                }
                Trans::None
            }
        }
    }
}
