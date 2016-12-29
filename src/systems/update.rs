use app::UpdateContext;
use fnv::FnvHashMap;
use app;
use specs;
use physics::{EntityInformation, Resolution, ShapeCast, Collision, CollisionBehavior};
use components::*;
use resources::*;
use specs::Join;
use utils::math::*;

pub fn add_systems(planner: &mut ::specs::Planner<UpdateContext>) {
    planner.add_system(PhysicSystem, "physic", 10);
    planner.add_system(WeaponSystem, "weapon", 9);
}

pub struct PhysicSystem;
#[allow(illegal_floating_point_constant_pattern)]
impl specs::System<app::UpdateContext> for PhysicSystem {
    fn run(&mut self, arg: specs::RunArg, context: app::UpdateContext) {
        use std::f32::consts::PI;
        use std::f32;

        let (dynamics, mut states, dampings, forces, types, mut physic_world, entities) = arg.fetch(|world| {
            (
                world.read::<PhysicDynamic>(),
                world.write::<PhysicState>(),
                world.read::<PhysicDamping>(),
                world.read::<PhysicForce>(),
                world.read::<PhysicType>(),
                world.write_resource::<PhysicWorld>(),
                world.entities(),
            )
        });

        let dt = context.dt;

        let mut resolutions = FnvHashMap::<specs::Entity,Resolution>::default();

        for (_, state, typ, entity) in (&dynamics, &mut states, &types, &entities).iter() {
            let mut f = [0., 0.];

            if let Some(&PhysicDamping(damping)) = dampings.get(entity) {
                f[0] -= damping*state.vel[0];
                f[1] -= damping*state.vel[1];
            }
            if let Some(force) = forces.get(entity) {
                f[0] += force.coef*force.strength*force.angle.cos();
                f[1] += force.coef*force.strength*force.angle.sin();
            }

            state.acc[0] = f[0]/typ.weight;
            state.acc[1] = f[1]/typ.weight;

            state.vel[0] += dt*state.acc[0];
            state.vel[1] += dt*state.acc[1];

            state.pos[0] += dt*state.vel[0];
            state.pos[1] += dt*state.vel[1];

            if typ.mask == 0 { continue }

            let shape_cast = ShapeCast {
                pos: state.pos,
                shape: typ.shape.clone(),
                mask: typ.mask,
                group: typ.group,
                not: vec!(entity),
            };

            physic_world.apply_on_shape(&shape_cast, &mut |other_info,collision| {
                let other_type = types.get(other_info.entity).expect("physic entity expect type component");
                let rate = match (typ.weight, other_type.weight) {
                     (f32::MAX, f32::MAX) => 0.5,
                    (f32::MAX, _) => 1.,
                    (_, f32::MAX) => 0.,
                    _ => typ.weight/(typ.weight+other_type.weight),
                };

                if rate != 1. {
                    let resolution = Resolution {
                        dx: collision.delta_x*(1.-rate),
                        dy: collision.delta_y*(1.-rate),
                    };
                    resolutions.entry(entity).or_insert(Resolution::none()).push(resolution);
                }
                if rate != 0. {
                    let resolution = Resolution {
                        dx: -collision.delta_x*rate,
                        dy: -collision.delta_y*rate,
                    };
                    resolutions.entry(other_info.entity).or_insert(Resolution::none()).push(resolution);
                }
            });

            physic_world.insert_dynamic(EntityInformation {
                entity: entity,
                pos: state.pos,
                group: typ.group,
                shape: typ.shape.clone(),
                mask: typ.mask,
            });
        }

        for (entity,res) in resolutions {
            let state = states.get_mut(entity).unwrap();
            let typ = types.get(entity).unwrap();

            state.pos[0] += res.dx;
            state.pos[1] += res.dy;

            match typ.collision {
                CollisionBehavior::Bounce => {
                    let angle = state.vel[1].atan2(state.vel[0]) + PI;
                    state.vel[0] = angle.cos();
                    state.vel[1] = angle.sin();
                },
                CollisionBehavior::Stop => state.vel = [0.,0.],
                CollisionBehavior::Back => {
                    state.vel[0] = -state.vel[0];
                    state.vel[1] = -state.vel[1];
                },
                CollisionBehavior::Persist => (),
            }
        }

        physic_world.movable = FnvHashMap::default(); // TODO only rewrite those that have been resolved
        for (_,state,typ,entity) in (&dynamics, &mut states, &types, &entities).iter() {
            physic_world.insert_dynamic(EntityInformation {
                entity: entity,
                pos: state.pos,
                group: typ.group,
                shape: typ.shape.clone(),
                mask: typ.mask,
            });
        }
    }
}

pub struct WeaponSystem;
impl specs::System<app::UpdateContext> for WeaponSystem {
    fn run(&mut self, arg: specs::RunArg, context: app::UpdateContext) {
        use weapon::{State, Kind};
        let (mut weapons, mut next_weapons, shoots, entities) = arg.fetch(|world| {
            (
                world.write::<Weapon>(),
                world.write::<NextWeapon>(),
                world.read::<Shoot>(),
                world.entities(),
            )
        });

        // create a fake weapon if next weapon and none weapon
        for (next_weapon, entity) in (&mut next_weapons, &entities).iter() {
            if weapons.get(entity).is_none() {
                let fake_weapon =  Weapon {
                    reload_factor: 0.,
                    setup_factor: 0.,
                    setdown_factor: 0.,
                    state: State::Setdown(1.),
                    kind: Kind::Sniper,
                };
                match weapons.insert(entity, fake_weapon) {
                    specs::InsertResult::Inserted => (),
                    _ => unreachable!(),
                }
            }
        }

        for (weapon, entity) in (&mut weapons, &entities).iter() {
            let shoot = shoots.get(entity).is_some();

            // set down
            match weapon.state {
                State::Setdown(_) => (),
                ref mut state @ _ => if next_weapons.get(entity).is_some() {
                    *state = State::Setdown(0.);
                },
            }

            // update loading
            match weapon.state {
                State::Reload(ref mut t) => *t += context.dt*weapon.reload_factor,
                State::Setup(ref mut t) => *t += context.dt*weapon.setup_factor,
                State::Setdown(ref mut t) => *t += context.dt*weapon.setdown_factor,
                State::Ready => (),
            }

            // update state and weapon if set down
            match weapon.state {
                State::Reload(t) | State::Setup(t) => if t >= 1. {
                    if shoot {
                        weapon.state = State::Reload(t-1.);
                        if let Kind::Hammer(ref mut b) = weapon.kind {
                            *b = !*b;
                        }
                        // TODO shoot
                    } else {
                        weapon.state = State::Ready;
                    };
                },
                State::Ready => if shoot {
                    weapon.state = State::Reload(context.dt*weapon.reload_factor);
                },
                State::Setdown(t) => if t >= 1. {
                    *weapon = next_weapons.remove(entity).unwrap().0;
                    weapon.state = State::Setup(t-1.);
                },
            };
        }
    }
}