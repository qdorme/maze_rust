use image::{ImageBuffer, Rgb, RgbImage, Rgba};
use macroquad::prelude::{
    BLACK, FilterMode, Texture2D, WHITE, clear_background, draw_texture, next_frame,
};
use rand::RngExt;
use std::env;
use std::ptr::slice_from_raw_parts;
use tracing::{info, debug};

const NORTH: u8 = 0b00000001;
const EAST: u8 = 0b00000010;
const SOUTH: u8 = 0b00000100;
const WEST: u8 = 0b00001000;
const ALL_WALLS: u8 = 0b00001111;
const VISITED: u8 = 0b00010000;
const CURRENT: u8 = 0b00100000;
const CLEAR: u8 = 0b11001111;
const ENTRY: u8 = 0b01000000;

struct Maze {
    width: u32,
    height: u32,
    walls: Vec<Vec<u8>>,
    paths: Vec<(u32, u32)>,
    structure_done: bool,
    entry: Option<(u32, u32)>,
    exit: Option<(u32, u32)>,
    candidate: Option<(u32, u32)>,
}

impl Maze {
    fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            walls: vec![vec![0b00001111; width as usize]; height as usize],
            paths: vec![],
            structure_done: false,
            entry: None,
            exit: None,
            candidate: None,
        }
    }

    pub fn generate_maze_by_step(&mut self) {
        if let Some(cell) = self.choose_isolated_neighbor() {
            if let Some(&(x, y)) = self.paths.last() {
                let (new_x, new_y, direction) = cell;
                // on enlève le mur entre les deux cellules
                match direction {
                    NORTH => {
                        self.walls[y as usize][x as usize] &= !NORTH;
                        self.walls[new_y as usize][new_x as usize] &= !SOUTH;
                    }
                    SOUTH => {
                        self.walls[y as usize][x as usize] &= !SOUTH;
                        self.walls[new_y as usize][new_x as usize] &= !NORTH;
                    }
                    EAST => {
                        self.walls[y as usize][x as usize] &= !EAST;
                        self.walls[new_y as usize][new_x as usize] &= !WEST;
                    }
                    WEST => {
                        self.walls[y as usize][x as usize] &= !WEST;
                        self.walls[new_y as usize][new_x as usize] &= !EAST;
                    }
                    _ => {}
                }
            }
            self.paths.push((cell.0, cell.1));
        } else {
            self.paths.pop();
            if self.paths.is_empty() {
                self.structure_done = true;
                self.paths.push((0, 0));
                self.candidate = None;
            }
        }
    }

    fn choose_isolated_neighbor(&self) -> Option<(u32, u32, u8)> {
        match self.paths.last() {
            Some(&(x, y)) => {
                let mut neighbors = vec![];
                // 1. on vérifie quel mur est intact excepté pour les côtés du labyrinthe
                // 2. on vérifie si le voisin est non visité 0b00001111
                // 3. on ajoute à la liste des voisins
                // 4. on choisit un voisin au hasard s'il y en a
                if y > 0 && self.walls[(y - 1) as usize][x as usize] & ALL_WALLS == ALL_WALLS {
                    neighbors.push((x, y - 1, NORTH)); // NORTH
                }
                if y + 1 < self.height
                    && self.walls[(y + 1) as usize][x as usize] & ALL_WALLS == ALL_WALLS
                {
                    neighbors.push((x, y + 1, SOUTH)); // SOUTH
                }
                if x > 0 && self.walls[y as usize][(x - 1) as usize] & ALL_WALLS == ALL_WALLS {
                    neighbors.push((x - 1, y, WEST)); // WEST
                }
                if x + 1 < self.width
                    && self.walls[y as usize][(x + 1) as usize] & ALL_WALLS == ALL_WALLS
                {
                    neighbors.push((x + 1, y, EAST)); // EAST
                }

                if !neighbors.is_empty() {
                    Some(neighbors[rand::rng().random_range(0..neighbors.len())])
                } else {
                    None
                }
            }
            None => None,
        }
    }

    pub fn find_entry_by_step(&mut self) -> Option<(u32, u32)> {
        let mut next_cells = vec![];
        let mut entry_candidates = vec![];

        self.paths.clone().iter().for_each(|c| {
            self.walls[c.1 as usize][c.0 as usize] &= !CURRENT;
            self.choose_unvisited_neighbor((*c).clone())
                .iter()
                .for_each(|n| {
                    next_cells.push((*n).clone());
                });
        });
        self.paths.clear();
        next_cells.iter().for_each(|c| {
            self.walls[c.1 as usize][c.0 as usize] |= CURRENT;
            self.walls[c.1 as usize][c.0 as usize] |= VISITED;
            self.paths.push((*c).clone());
            if c.0 == 0 || c.1 == 0 || c.0 == self.width - 1 || c.1 == self.height - 1 {
                entry_candidates.push((*c).clone());
            }
        });
        if !entry_candidates.is_empty() {
            self.candidate =
                Some(entry_candidates[rand::rng().random_range(0..entry_candidates.len())]);
        }
        if self.paths.is_empty() {
            let found = self.candidate.take();
            info!("Entry found {},{}", found.unwrap().0, found.unwrap().1);
            self.paths.push(found.unwrap());
            self.candidate = None;
            self.walls.iter_mut().for_each(|row| {
                row.iter_mut().for_each(|cell| {
                    *cell &= !VISITED;
                })
            });
            self.walls[found.unwrap().1 as usize][found.unwrap().0 as usize] |= ENTRY;
            found
        } else {
            None
        }
    }

    fn choose_unvisited_neighbor(&self, current_cell: (u32, u32)) -> Vec<(u32, u32)> {
        let mut neighbors = vec![];

        let (x, y) = current_cell;

        debug!("Choosing unvisited neighbors for cell ({}, {})", x, y);
        debug!("self.walls[y as usize][x as usize] & NORTH: {:08b}", self.walls[y as usize][x as usize] & NORTH);
        debug!("self.walls[y as usize][x as usize] & NORTH != NORTH {}", self.walls[y as usize][x as usize] & NORTH != NORTH);
        debug!("self.walls[y as usize][x as usize] & SOUTH: {:08b}", self.walls[y as usize][x as usize] & SOUTH);
        debug!("self.walls[y as usize][x as usize] & SOUTH != SOUTH {}", self.walls[y as usize][x as usize] & SOUTH != SOUTH);
        debug!("self.walls[y as usize][x as usize] & EAST: {:08b}", self.walls[y as usize][x as usize] & EAST);
        debug!("self.walls[y as usize][x as usize] & EAST != EAST {}", self.walls[y as usize][x as usize] & EAST != EAST);
        debug!("self.walls[y as usize][x as usize] & WEST: {:08b}", self.walls[y as usize][x as usize] & WEST);
        debug!("self.walls[y as usize][x as usize] & WEST != WEST {}", self.walls[y as usize][x as usize] & WEST != WEST);

        if (self.walls[y as usize][x as usize] & NORTH != NORTH)
            && (self.walls[(y-1) as usize][x as usize] & VISITED != VISITED)
        {
            neighbors.push((x, y - 1));
        }
        if (self.walls[y as usize][x as usize] & SOUTH != SOUTH)
            && (self.walls[(y+1) as usize][x as usize] & VISITED != VISITED)
        {
            neighbors.push((x, y + 1));
        }
        if (self.walls[y as usize][x as usize] & WEST != WEST)
            && (self.walls[y as usize][(x-1) as usize] & VISITED != VISITED)
        {
            neighbors.push((x - 1, y));
        }
        if (self.walls[y as usize][x as usize] & EAST != EAST)
            && (self.walls[y as usize][(x+1) as usize] & VISITED != VISITED)
        {
            neighbors.push((x + 1, y));
        }

        neighbors
    }
}

fn generate_image(maze: &Maze) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
    let square_size = 10;
    let mut img = RgbImage::new(
        (maze.width + 2) * square_size,
        (maze.height + 2) * square_size,
    );
    img.fill(255); // fond blanc

    maze.walls.iter().enumerate().for_each(|(y, row)| {
        row.iter().enumerate().for_each(|(x, cell)| {
            let corners = (
                ((x + 1) as u32 * square_size, (y + 1) as u32 * square_size),
                ((x + 2) as u32 * square_size, (y + 1) as u32 * square_size),
                ((x + 2) as u32 * square_size, (y + 2) as u32 * square_size),
                ((x + 1) as u32 * square_size, (y + 2) as u32 * square_size),
            );

            if *cell & NORTH == NORTH {
                (corners.0.0..corners.1.0).for_each(|x| {
                    img.put_pixel(x, corners.0.1, Rgb([0, 0, 0]));
                })
            }
            if *cell & SOUTH == SOUTH {
                (corners.3.0..corners.2.0).for_each(|x| {
                    img.put_pixel(x, corners.2.1, Rgb([0, 0, 0]));
                })
            }
            if *cell & EAST == EAST {
                (corners.1.1..corners.2.1).for_each(|y| {
                    img.put_pixel(corners.1.0, y, Rgb([0, 0, 0]));
                })
            }
            if *cell & WEST == WEST {
                (corners.0.1..corners.3.1).for_each(|y| {
                    img.put_pixel(corners.0.0, y, Rgb([0, 0, 0]));
                })
            }
            if *cell & VISITED == VISITED {
                (corners.0.0 + 1..corners.2.0).for_each(|x| {
                    (corners.0.1 + 1..corners.2.1).for_each(|y| {
                        img.put_pixel(x, y, Rgb([200, 200, 200]));
                    })
                })
            }
            if *cell & ENTRY == ENTRY {
                (corners.0.0 + 1..corners.2.0).for_each(|x| {
                    (corners.0.1 + 1..corners.2.1).for_each(|y| {
                        img.put_pixel(x, y, Rgb([255, 200, 200]));
                    })
                });
            }
        })
    });

    img.save("maze.png").expect("Échec");
    img
}

#[macroquad::main("Maze")]
async fn main() {
    tracing_subscriber::fmt::fmt().with_env_filter("info").init();

    let mut width = 10;
    let mut height = 10;

    let args: Vec<String> = env::args().collect();
    if args.len() == 3 {
        width = args[1].parse().expect("Invalid width");
        height = args[2].parse().expect("Invalid height");
    }

    info!("Starting maze generation ({}, {})", width, height);
    info!("Initialization phase");

    let mut random = rand::rng();

    let mut maze = Maze::new(width, height);
    let starting_cell = (
        random.random_range(0..maze.width),
        random.random_range(0..maze.height),
    );

    debug!("Starting at cell: {:?}", starting_cell);
    maze.paths.push(starting_cell);

    loop {
        if !maze.structure_done {

            maze.generate_maze_by_step();
        } else if maze.entry.is_none() {

            maze.entry = maze.find_entry_by_step();
        } else if maze.exit.is_none() {

            maze.exit = maze.find_entry_by_step();
        }

        let img = generate_image(&maze);

        let texture = convert_to_texture(img);

        clear_background(BLACK);
        draw_texture(&texture, 0.0, 0.0, WHITE);
        next_frame().await;
    }
}

fn convert_to_texture(img: ImageBuffer<Rgb<u8>, Vec<u8>>) -> Texture2D {
    let (w, h) = img.dimensions();
    let raw = img.into_raw(); // Vec<u8> RGB

    // Convertir en RGBA pour macroquad
    let rgba: Vec<u8> = raw
        .chunks(3)
        .flat_map(|p| [p[0], p[1], p[2], 255])
        .collect();

    let texture = Texture2D::from_rgba8(w as u16, h as u16, &rgba);
    texture.set_filter(FilterMode::Nearest);
    texture
}
