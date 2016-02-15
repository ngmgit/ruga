//! https://en.wikipedia.org/wiki/Maze_generation_algorithm#Randomized_Kruskal.27s_algorithm

extern crate rand;

use util::direction::Direction;
use self::rand::distributions::{IndependentSample, Range};
use world::World;

const AVERAGE_BOID_PER_UNIT: u32 = 1;

#[derive(Debug)]
enum Wall {
    Vertical(usize,usize),
    Horizontal(usize,usize),
}

fn generate_partial_reverse_randomized_kruskal(width: usize, height: usize, percent: f64) -> Vec<bool> {
    assert_eq!(width.wrapping_rem(2), 1);
    assert_eq!(height.wrapping_rem(2), 1);

    let index = |x: usize, y: usize| y*width+x;

    let mut grid = Vec::with_capacity(width*height);
    for i in 0..width*height {
        grid.push((false,i));
    }

    for i in 0..width {
        grid[i] = (true, i);
        let j = height*(width-1)+i;
        grid[j] = (true, j);
    }

    for i in 0..height {
        grid[i*width] = (true, i*width);
        let j = (i+1)*width - 1;
        grid[j] = (true,j);
    }

    let horizontal_wall = (width-5)/2 * (height-3)/2;
    let vertical_wall = (width-3)/2 * (height-5)/2;
    let horizontal_wall_width = (width-5)/2;
    let vertical_wall_width = (width-3)/2;

    let mut walls = Vec::with_capacity(horizontal_wall+vertical_wall);
    for i in 0..vertical_wall {
        walls.push(Wall::Vertical(i.wrapping_rem(vertical_wall_width)*2+2, (i/vertical_wall_width)*2+3));
    }
    for i in 0..horizontal_wall {
        walls.push(Wall::Horizontal(i.wrapping_rem(horizontal_wall_width)*2+3, (i/horizontal_wall_width)*2+2));
    }

    let mut rng = rand::thread_rng();

    let stop = ((walls.len() as f64)*(1.-percent/100.)) as usize;

    while walls.len() > stop {
        let i = Range::new(0,walls.len()).ind_sample(&mut rng);
        assert!(i<walls.len());
        let (c1,c2,c3) = match walls.swap_remove(i) {
            Wall::Vertical(x,y) => {
                (index(x,y-1), index(x,y), index(x,y+1))
            },
            Wall::Horizontal(x,y) => {
                (index(x-1,y), index(x,y), index(x+1,y))
            },
        };

        let ((_,s1),(_,s2),(_,s3)) = (grid[c1],grid[c2],grid[c3]);

        if s1 != s3 {
            grid[c1] = (true,s1);
            grid[c2] = (true,s2);
            grid[c3] = (true,s3);
            for &mut(_,ref mut s) in &mut grid {
                if *s == s2 || *s == s3 {
                    *s = s1;
                }
            }
        }
    }

    grid.iter().map(|&(b,_)| b).collect::<Vec<bool>>()
}

pub fn generate() -> World {
    let width = 17;
    let height = 17;
    let unit = 16.;
    let percent = 30.;

    let maze = generate_partial_reverse_randomized_kruskal(width,height,percent);

    let mut world = World::new(unit);

    let mut rng = rand::thread_rng();

    let boid_range = Range::new(0,AVERAGE_BOID_PER_UNIT*2);//.ind_sample(&mut rng);
    let unit_range = Range::new(0.,unit);
    let angle_range = Range::new(0.,6.28);

    for i in 0..maze.len() {
        let x = (i.wrapping_rem(width)) as i32;
        let y = (i/width) as i32;

        if maze[i] {
            world.insert_wall(x,y);
        } else {
            // insert boids
            let nbr_of_boid = boid_range.ind_sample(&mut rng);
            for _ in 0..nbr_of_boid {
                let boid_x = unit_range.ind_sample(&mut rng) + x as f64 * unit;
                let boid_y = unit_range.ind_sample(&mut rng) + y as f64 * unit;
                let boid_angle = angle_range.ind_sample(&mut rng);
                world.insert_boid(boid_x,boid_y,boid_angle);
            }
        }
    }

    world.insert_armory(1,1);
    //world.insert_moving_wall(width as i32 - 1,height as i32 - 2,Direction::Left);
    world.insert_character(unit*1.5,unit*1.5,0.);

    world
}
