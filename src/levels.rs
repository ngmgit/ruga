use specs;
use entities;
use hlua;
use hlua::Lua;
// use hlua::any::AnyLuaValue;
use std::fs::File;
use config;
use std::path::Path;
use std::io;
use specs::Join;
use physic;
use std::collections::HashSet;

#[derive(Debug)]
pub enum LoadError {
    OpenFile(io::Error),
    Lua(hlua::LuaError),
}

pub fn load<'l>(level: &str, world: &specs::World) -> Result<specs::Entity,LoadError>{
    // flush world
    for entity in world.entities().iter() {
        world.delete_later(entity);
    }
    world.maintain();

    // init lua level creation context
    let mut lua: Lua<'l> = Lua::new();
    lua.set("add_character", hlua::function2(|x: i32,y: i32| {
        entities::add_character(world,[x as isize,y as isize]);
    }));
    lua.set("add_wall", hlua::function2(|x: i32,y: i32| {
        entities::add_wall(world,[x as isize,y as isize]);
    }));
    lua.set("add_column", hlua::function2(|x: i32,y: i32| {
        entities::add_column(world,[x as isize,y as isize]);
    }));
    lua.set("add_monster", hlua::function2(|x: i32,y: i32| {
        entities::add_monster(world,[x as isize,y as isize]);
    }));
    lua.set("add_laser", hlua::function2(|x: i32,y: i32| {
        entities::add_laser(world,[x as isize,y as isize]);
    }));
    lua.set("add_portal", hlua::function3(|x: i32,y: i32,dest: String| {
        entities::add_portal(world,[x as isize,y as isize],dest);
    }));

    // execute common script
    let path = Path::new(&*config.levels.dir).join(Path::new(&*format!("{}{}",config.levels.common,".lua")));
    let file = try!(File::open(&path).map_err(|e| LoadError::OpenFile(e)));
    try!(lua.execute_from_reader::<(),_>(file).map_err(|e| LoadError::Lua(e)));

    // execute level script
    let path = Path::new(&*config.levels.dir).join(Path::new(&*format!("{}{}",level,".lua")));
    let file = try!(File::open(&path).map_err(|e| LoadError::OpenFile(e)));
    try!(lua.execute_from_reader::<(),_>(file).map_err(|e| LoadError::Lua(e)));

    // add_physic_world
    let master_entity = world.create_now()
        .with::<physic::PhysicWorld>(physic::PhysicWorld::new())
        .build();

    // init_physic_world
    let mut physic_worlds = world.write::<physic::PhysicWorld>();
    physic_worlds.get_mut(master_entity).unwrap().fill(&world);

    Ok(master_entity)
}

#[test]
fn test_levels() {
    let mut visited = HashSet::new();
    let mut to_visit = vec!(config.levels.first_level.clone());

    loop {
        let level = match to_visit.pop() {
            Some(level) => level,
            None => break,
        };
        // init lua level creation context
        let mut lua = Lua::new();
        lua.set("add_character", hlua::function2(|_: i32,_: i32| {
        }));
        lua.set("add_wall", hlua::function2(|_: i32,_: i32| {
        }));
        lua.set("add_column", hlua::function2(|_: i32,_: i32| {
        }));
        lua.set("add_monster", hlua::function2(|_: i32,_: i32| {
        }));
        lua.set("add_laser", hlua::function2(|_: i32,_: i32| {
        }));
        lua.set("add_portal", hlua::function3(|_: i32,_: i32,dest: String| {
            if !visited.contains(&dest) {
                to_visit.push(dest);
            }
        }));

        // execute common script
        let path = Path::new(&*config.levels.dir).join(Path::new(&*format!("{}{}",config.levels.common,".lua")));
        let file = File::open(&path).map_err(|e| LoadError::OpenFile(e)).unwrap();
        lua.execute_from_reader::<(),_>(file).map_err(|e| LoadError::Lua(e)).unwrap();

        // execute level script
        let path = Path::new(&*config.levels.dir).join(Path::new(&*format!("{}{}",level,".lua")));
        let file = File::open(&path).map_err(|e| LoadError::OpenFile(e)).unwrap();
        lua.execute_from_reader::<(),_>(file).map_err(|e| LoadError::Lua(e)).unwrap();

        visited.insert(level);
    }
}
