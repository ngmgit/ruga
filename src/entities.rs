use components::*;
use specs;
use graphics::{Color, Layer};
use config;

pub fn add_character(world: &mut specs::World, pos: [isize;2]) {
    world.create_now()
        .with::<PhysicState>(PhysicState::new(pos))
        .with::<PhysicDynamic>(PhysicDynamic)
        .with::<PhysicType>(PhysicType::new_movable(
                config.entities.char_group.val,
                config.entities.char_mask.val,
                Shape::Circle(config.entities.char_radius),
                CollisionBehavior::Persist,
                config.entities.char_velocity,
                config.entities.char_time,
                config.entities.char_weight))
        .with::<PhysicForce>(PhysicForce::new())
        .with::<Life>(Life::new(config.entities.char_die_snd))
        .with::<Graphic>(Graphic::new(
                Color::from_str(&*config.entities.char_color),
                Layer::from_str(&*config.entities.char_layer)))
        .with::<PlayerControl>(PlayerControl)
        .build();
    world.create_now()
        .with::<GridSquare>(GridSquare::new(pos))
        .with::<Graphic>(Graphic::new(
                Color::from_str(&*config.entities.portal_end_color),
                Layer::from_str(&*config.entities.portal_end_layer)))
        .build();
}

pub fn add_wall(world: &mut specs::World, pos: [isize;2]) {
    world.create_now()
        .with::<PhysicState>(PhysicState::new(pos))
        .with::<PhysicStatic>(PhysicStatic)
        .with::<PhysicType>(PhysicType::new_static(
                config.entities.wall_group.val,
                config.entities.wall_mask.val,
                Shape::Square(config.entities.wall_radius)))
        .with::<Graphic>(Graphic::new(
                Color::from_str(&*config.entities.wall_color),
                Layer::from_str(&*config.entities.wall_layer)))
        .build();
}

pub fn add_column(world: &mut specs::World, pos: [isize;2]) {
    world.create_now()
        .with::<PhysicState>(PhysicState::new(pos))
        .with::<PhysicStatic>(PhysicStatic)
        .with::<PhysicType>(PhysicType::new_static(
                config.entities.column_group.val,
                config.entities.column_mask.val,
                Shape::Square(config.entities.column_radius)))
        .with::<Graphic>(Graphic::new(
                Color::from_str(&*config.entities.column_color),
                Layer::from_str(&*config.entities.column_layer)))
        .build();
    world.create_now()
        .with::<PhysicState>(PhysicState::new(pos))
        .with::<Ball>(Ball::new(pos,config.entities.ball_create_snd))
        .with::<PhysicDynamic>(PhysicDynamic)
        .with::<PhysicType>(PhysicType::new_movable(
                config.entities.ball_group.val,
                config.entities.ball_mask.val,
                Shape::Circle(config.entities.ball_radius),
                CollisionBehavior::Persist,
                config.entities.ball_velocity,
                config.entities.ball_time,
                config.entities.ball_weight))
        .with::<PhysicForce>(PhysicForce::new_full())
        .with::<PhysicTrigger>(PhysicTrigger::new())
        .with::<Life>(Life::new(config.entities.ball_die_snd))
        .with::<Graphic>(Graphic::new(
                Color::from_str(&*config.entities.ball_color),
                Layer::from_str(&*config.entities.ball_layer)))
        .with::<TowardPlayerControl>(TowardPlayerControl)
        .with::<Killer>(Killer {
            kamikaze: false,
            mask: config.entities.ball_killer_mask.val,
            kill_snd: config.entities.ball_kill_snd,
        })
        .build();
}

pub fn add_monster(world: &mut specs::World, pos: [isize;2]) {
    world.create_now()
        .with::<PhysicState>(PhysicState::new(pos))
        .with::<PhysicDynamic>(PhysicDynamic)
        .with::<PhysicType>(PhysicType::new_movable(
                config.entities.monster_group.val,
                config.entities.monster_mask.val,
                Shape::Circle(config.entities.monster_radius),
                CollisionBehavior::Persist,
                config.entities.monster_velocity,
                config.entities.monster_time,
                config.entities.monster_weight))
        .with::<PhysicForce>(PhysicForce::new())
        .with::<Life>(Life::new(config.entities.monster_die_snd))
        .with::<Graphic>(Graphic::new(
                Color::from_str(&*config.entities.monster_color),
                Layer::from_str(&*config.entities.monster_layer)))
        .with::<MonsterControl>(MonsterControl::new())
        .with::<Killer>(Killer {
            kamikaze: true,
            mask: config.entities.monster_killer_mask.val,
            kill_snd: config.entities.monster_kill_snd,
        })
        .build();
}

pub fn add_laser(world: &mut specs::World, pos: [isize;2]) {
    world.create_now()
        .with::<PhysicState>(PhysicState::new(pos))
        .with::<PhysicStatic>(PhysicStatic)
        .with::<PhysicType>(PhysicType::new_static(
                config.entities.laser_group.val,
                config.entities.laser_mask.val,
                Shape::Square(config.entities.laser_radius)))
        .with::<Graphic>(Graphic::new(
                Color::from_str(&*config.entities.laser_color),
                Layer::from_str(&*config.entities.laser_layer)))
        .with::<Killer>(Killer {
            kamikaze: false,
            mask: config.entities.laser_killer_mask.val,
            kill_snd: config.entities.laser_kill_snd,
        })
        .build();
}

pub fn add_portal(world: &mut specs::World, pos: [isize;2], destination: String) {
    world.create_now()
        .with::<Portal>(Portal::new(destination))
        .with::<GridSquare>(GridSquare::new(pos))
        .with::<Graphic>(Graphic::new(
                Color::from_str(&*config.entities.portal_start_color),
                Layer::from_str(&*config.entities.portal_start_layer)))
        .build();
}

