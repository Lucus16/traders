pub mod base;
pub mod cargo;
pub mod ui;

pub use base::*;
pub use cargo::*;
pub use ui::*;

use amethyst::{
    assets::Handle,
    core::transform::Transform,
    ecs::{Builder, Component, Entity, NullStorage, VecStorage, World, WorldExt},
    renderer::{SpriteRender, SpriteSheet},
};

#[derive(Debug)]
pub enum ShipBehaviour {
    Idle,
    FlyTo(Entity),
}

impl Component for ShipBehaviour {
    type Storage = VecStorage<Self>;
}

#[derive(Default)]
pub struct Station;

impl Component for Station {
    type Storage = NullStorage<Self>;
}

#[derive(Default)]
pub struct Trader;

impl Component for Trader {
    type Storage = NullStorage<Self>;
}

pub type Parent = amethyst::core::transform::Parent;
pub type ParentHierarchy = amethyst::core::transform::ParentHierarchy;

fn compute_name(target: &Entity, name: &str) -> String {
    format!("{}-{}-{}", name, target.gen().id(), target.id())
}

static METAL_ORE_RECIPE: FabricationRecipe = FabricationRecipe {
    duration: Time(50),
    ingredients: &[],
    products: &[(CargoType::MetalOre, CargoUnits(1))],
};
static _METAL_RECIPE: FabricationRecipe = FabricationRecipe {
    duration: Time(50),
    ingredients: &[(CargoType::MetalOre, CargoUnits(5))],
    products: &[(CargoType::Metal, CargoUnits(1))],
};

pub fn create_station(world: &mut World, pos: Position) -> Entity {
    let sprite_number = 1;
    let sprite_sheet = (*world.fetch::<Handle<SpriteSheet>>()).clone();

    let (width, height) = (51., 58.);
    let hitbox = Hitbox::new(Translation2::new(width, height));

    let res = world
        .create_entity()
        .with(Station)
        .with(pos)
        .with(SpriteRender {
            sprite_sheet,
            sprite_number,
        })
        .with(hitbox)
        .with(Transform::default())
        .with(Angle::new(f32::default()))
        .with(AngularMomentum::new(0.001))
        .with(Cargo::new(CargoUnits(1000000)))
        .with(FabricationModule {
            progress: None,
            recipe: &METAL_ORE_RECIPE,
        })
        .build();

    let anchor = create_ui_anchor(world, &res, "station", true);
    create_ui_label(world, &anchor, compute_name(&res, "station"));

    res
}

pub fn create_trader(world: &mut World, pos: Position, behaviour: ShipBehaviour) -> Entity {
    let sprite_number = 0;
    let sprite_sheet = (*world.fetch::<Handle<SpriteSheet>>()).clone();

    let (width, height) = (39., 57.);

    let hitbox = Hitbox::new(Translation2::new(width, height));

    let res = world
        .create_entity()
        .with(Trader)
        .with(pos)
        .with(Velocity::default())
        .with(SpriteRender {
            sprite_sheet,
            sprite_number,
        })
        .with(hitbox)
        .with(Transform::default())
        .with(Angle::new(f32::default()))
        .with(AngularMomentum::new(0.0))
        .with(behaviour)
        .with(Cargo::new(CargoUnits(100)))
        .build();

    let anchor = create_ui_anchor(world, &res, "trader", true);
    create_ui_label(world, &anchor, compute_name(&res, "trader"));

    res
}
