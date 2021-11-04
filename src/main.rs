use std::{thread, time};

use rand::prelude::IteratorRandom;
use rand::rngs::ThreadRng;
use rand::{thread_rng, Rng};
use terminal_size::{terminal_size, Height, Width};

const FISH_BREED_INTERVAL: u8 = 10;
const SHARK_BREED_INTERVAL: u8 = 14;
const SHARK_STARVE_INTERVAL: u8 = 8;
const WRAP_WORLD: bool = true;
const MS_BETWEEN_CHRONON: u64 = 5;

#[derive(Default, Clone, Copy)]
struct FishAttr {
    lived_chronons: usize,
    since_reproduced: u8,
}

#[derive(Default, Clone, Copy)]
struct SharkAttr {
    lived_chronons: usize,
    since_reproduced: u8,
    since_ate: u8,
}

#[derive(Clone)]
enum Cell {
    Empty,
    Fish(FishAttr),
    Shark(SharkAttr),
}

struct Stats {
    chronon: usize,
    min_fish: usize,
    max_fish: usize,
    min_shark: usize,
    max_shark: usize,
}

fn main() {
    // Get terminal size
    let size = terminal_size();
    let (world_width, world_height) = match size {
        Some((Width(w), Height(h))) => (w / 2, h - 2),
        None => (80, 40),
    };
    let world_width = world_width as usize;
    let world_height = world_height as usize;

    // Construct world
    let nr_of_cells = world_width * world_height;
    let mut world = vec![Cell::Empty; nr_of_cells];

    // Randomly setup the world
    let mut rng = thread_rng();
    let distro = rand::distributions::Uniform::new_inclusive(0, 100);
    for c in &mut world {
        *c = match rng.sample(distro) {
            p if p < 10 => Cell::Shark(SharkAttr {
                // 10% chance its a shark
                ..Default::default()
            }),
            p if p < 50 => Cell::Fish(FishAttr {
                // 40% chance its a fish
                ..Default::default()
            }),
            _ => Cell::Empty, // 50% chance its empty
        }
    }

    // Setup stats and loop
    let mut stats = Stats {
        chronon: 1,
        min_fish: usize::MAX,
        max_fish: 0,
        min_shark: usize::MAX,
        max_shark: 0,
    };

    loop {
        animate(&mut world, world_width);

        let nr_sharks = print_world(&world, world_width, world_height, &mut stats);
        if nr_sharks == 0 {
            break;
        }

        let sleep_time = time::Duration::from_millis(MS_BETWEEN_CHRONON);
        thread::sleep(sleep_time);
        stats.chronon += 1;
    }
    println!("All sharks died!")
}

fn animate(world: &mut [Cell], world_width: usize) {
    let mut rng = thread_rng();

    for i in 0..world.len() {
        match world[i] {
            Cell::Fish(attr) => animate_fish(&mut rng, world, world_width, i, attr),
            Cell::Shark(attr) => animate_shark(&mut rng, world, world_width, i, attr),
            _ => (),
        }
    }
}

fn animate_fish(
    rng: &mut ThreadRng,
    world: &mut [Cell],
    world_width: usize,
    i: usize,
    mut attr: FishAttr,
) {
    // Find an empty cell close by
    let neighbours = find_neighbours(world, world_width, i, WRAP_WORLD);
    let empty_cell = empty(rng, &neighbours);

    // Update attributes
    attr.lived_chronons += 1;
    attr.since_reproduced += 1;

    match empty_cell {
        None => {
            // No place to move to, so just save the attributes
            world[i] = Cell::Fish(attr);
        }
        Some((c, _)) => {
            // We can move somewhere
            let old_cell_index = i;
            let new_cell_index = *c;

            // Are we creating a baby?
            let maybe_baby = match attr.since_reproduced > FISH_BREED_INTERVAL {
                true => {
                    attr.since_reproduced = 0;
                    Cell::Fish(FishAttr {
                        ..Default::default()
                    })
                }
                false => Cell::Empty,
            };

            // Save updates to world
            world[old_cell_index] = maybe_baby;
            world[new_cell_index] = Cell::Fish(attr);
        }
    }
}

fn animate_shark(
    rng: &mut ThreadRng,
    world: &mut [Cell],
    world_width: usize,
    i: usize,
    mut attr: SharkAttr,
) {
    // Find an prey close by
    let neighbours = find_neighbours(world, world_width, i, WRAP_WORLD);
    let prey_cell = prey(rng, &neighbours);

    // Update attributes
    attr.lived_chronons += 1;
    attr.since_reproduced += 1;
    attr.since_ate += 1;

    let old_cell_index = i;

    // Get index of where we are moving next
    let new_cell_index = match prey_cell {
        Some((c, _)) => {
            // Yay, we found some prey, we are moving there
            attr.since_ate = 0;
            Some(*c)
        }
        None => {
            // No prey, but lets try to find an empty cell then
            empty(rng, &neighbours).map(|(c, _)| *c)
        }
    };

    // Are we dying of starvation?
    if attr.since_ate > SHARK_STARVE_INTERVAL {
        world[old_cell_index] = Cell::Empty;
        return;
    }

    match new_cell_index {
        None => {
            // No place to move to, so just save the attributes
            world[old_cell_index] = Cell::Shark(attr);
        }
        Some(n) => {
            // We can move somewhere !

            // Are we creating a baby?
            let maybe_baby = match attr.since_reproduced > SHARK_BREED_INTERVAL {
                true => {
                    attr.since_reproduced = 0;
                    Cell::Shark(SharkAttr {
                        ..Default::default()
                    })
                }
                false => Cell::Empty,
            };

            // Save updates to world
            world[old_cell_index] = maybe_baby;
            world[n] = Cell::Shark(attr);
        }
    }
}

fn empty<'a>(r: &mut ThreadRng, n: &'a [(usize, &Cell)]) -> Option<&'a (usize, &'a Cell)> {
    n.iter().filter(|(_, c)| matches!(c, Cell::Empty)).choose(r)
}

fn prey<'a>(r: &mut ThreadRng, n: &'a [(usize, &Cell)]) -> Option<&'a (usize, &'a Cell)> {
    n.iter()
        .filter(|(_, c)| matches!(c, Cell::Fish(_)))
        .choose(r)
}

fn print_world(
    world: &[Cell],
    world_width: usize,
    world_height: usize,
    stats: &mut Stats,
) -> usize {
    // Clear terminal and move cursor to top left
    print!("\x1B[2J\x1B[1;1H");

    let mut fish: usize = 0;
    let mut sharks: usize = 0;

    // Draw Cells
    for (i, cell) in world.iter().enumerate() {
        let symbol = match cell {
            Cell::Empty => "  ",
            Cell::Fish(_) => {
                fish += 1;
                "🐋"
            }
            Cell::Shark(attr) => {
                sharks += 1;
                match attr.lived_chronons {
                    a if a > 50 => "🐙", // For fun, distinguish really old sharks
                    _ => "🐠",
                }
            }
        };
        print!("{}", symbol);

        if i % world_width == world_width - 1 {
            println!()
        }
    }

    // Update stats
    if fish > stats.max_fish {
        stats.max_fish = fish;
    }
    if fish < stats.min_fish {
        stats.min_fish = fish;
    }
    if sharks > stats.max_shark {
        stats.max_shark = sharks;
    }
    if sharks < stats.min_shark {
        stats.min_shark = sharks;
    }

    println!(
        "Chronon: {} Fish: {} (min: {}, max: {}) Sharks: {} (min: {}, max: {}) W: {} H: {}",
        stats.chronon,
        fish,
        stats.min_fish,
        stats.max_fish,
        sharks,
        stats.min_shark,
        stats.max_shark,
        world_width,
        world_height,
    );

    sharks
}

fn find_neighbours<T>(arr: &[T], width: usize, i: usize, wrap: bool) -> Vec<(usize, &T)> {
    let mut n: Vec<(usize, &T)> = Vec::with_capacity(4);
    let c = i % width;

    if wrap {
        // Left
        let index = match i % width == 0 {
            true => i + width - 1, // We are on the far left
            false => i - 1,
        };
        n.push((index, &arr[index]));

        // Right
        let index = match i % width == width - 1 {
            true => i + 1 - width, // We are on the far right
            false => i + 1,
        };
        n.push((index, &arr[index]));

        // Up
        let index = match i >= width {
            true => i - width, // We are not in the first row
            false => arr.len() - width + i,
        };
        n.push((index, &arr[index]));

        // Down
        let index = match i >= arr.len() - width {
            true => i % width, // We are in the last row
            false => i + width,
        };
        n.push((index, &arr[index]));
    } else {
        // Left available?
        if c > 0 {
            let index = i - 1;
            n.push((index, &arr[index]));
        }

        // Right available?
        if c < width - 1 {
            let index = i + 1;
            n.push((index, &arr[index]));
        }

        // Up available ?
        if i >= width {
            let index = i - width;
            n.push((index, &arr[index]));
        }

        // Down available ?
        let index = i + width;
        if index < arr.len() {
            n.push((index, &arr[index]));
        }
    }

    n
}

#[test]
fn test_neighbour() {
    // 0 1 2
    // 3 4 5
    // 6 7 8
    let world = vec![0, 1, 2, 3, 4, 5, 6, 7, 8];
    let res = vec![
        vec![(1, &1), (3, &3)],                   // 0
        vec![(0, &0), (2, &2), (4, &4)],          // 1
        vec![(1, &1), (5, &5)],                   // 2
        vec![(4, &4), (0, &0), (6, &6)],          // 3
        vec![(3, &3), (5, &5), (1, &1), (7, &7)], // 4
        vec![(4, &4), (2, &2), (8, &8)],          // 5
        vec![(7, &7), (3, &3)],                   // 6
        vec![(6, &6), (8, &8), (4, &4)],          // 7
        vec![(7, &7), (5, &5)],                   // 8
    ];
    for i in 0..9 {
        let n = find_neighbours(&world, 3, i, false);
        assert_eq!(res[i], n);
    }
}

#[test]
fn test_neighbour_wrap() {
    // 0 1 2
    // 3 4 5
    // 6 7 8
    let world = vec![0, 1, 2, 3, 4, 5, 6, 7, 8];
    let res = vec![
        vec![(2, &2), (1, &1), (6, &6), (3, &3)], // 0
        vec![(0, &0), (2, &2), (7, &7), (4, &4)], // 1
        vec![(1, &1), (0, &0), (8, &8), (5, &5)], // 2
        vec![(5, &5), (4, &4), (0, &0), (6, &6)], // 3
        vec![(3, &3), (5, &5), (1, &1), (7, &7)], // 4
        vec![(4, &4), (3, &3), (2, &2), (8, &8)], // 5
        vec![(8, &8), (7, &7), (3, &3), (0, &0)], // 6
        vec![(6, &6), (8, &8), (4, &4), (1, &1)], // 7
        vec![(7, &7), (6, &6), (5, &5), (2, &2)], // 8
    ];
    for i in 0..9 {
        let n = find_neighbours(&world, 3, i, true);
        assert_eq!(res[i], n);
    }
}
