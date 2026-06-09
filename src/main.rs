use image::{ImageBuffer, Rgb, RgbImage, Rgba};
use rand::RngExt;
use tracing::info;

const NORTH: u8 = 0b00000001;
const EAST: u8 = 0b00000010;
const SOUTH: u8 = 0b00000100;
const WEST: u8 = 0b00001000;
const ALL_WALLS: u8 = 0b00001111;

struct Maze {
    width: u32,
    height: u32,
    walls: Vec<Vec<u8>>,
    paths: Vec<(u32, u32)>,
}

impl Maze {
    fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            walls: vec![vec![0b00001111; width as usize]; height as usize],
            paths: vec![],
        }
    }

    fn generate_maze(&mut self, start: (u32, u32)) {
        self.paths.push(start);
        info!("Starting maze generation at cell: {:?}", start);

        while !self.paths.is_empty() {
            if let Some(cell) = self.choose_unvisited_neighbor() {
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
            }
        }
    }

    fn choose_unvisited_neighbor(&self) -> Option<(u32, u32, u8)> {

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
}

fn generate_image(maze: &Maze) {
    let square_size = 10;
    let mut img = RgbImage::new(
        (maze.width+2)*square_size, (maze.height+2)*square_size);
    img.fill(255); // fond blanc

    maze.walls.iter().enumerate().for_each(|(y, row)| {
        row.iter().enumerate().for_each(|(x, cell)| {
            let corners = (
                ((x+1) as u32 * square_size, (y+1) as u32 * square_size ),
                ((x+2) as u32 * square_size, (y+1) as u32 * square_size),
                ((x+2) as u32 * square_size, (y+2) as u32 * square_size),
                ((x+1) as u32 * square_size, (y+2) as u32 * square_size ));

            if *cell & NORTH == NORTH {
                (corners.0.0 .. corners.1.0).for_each(|x| {
                    img.put_pixel(x, corners.0.1, Rgb([0, 0, 0]));
                })
            }
            if *cell & SOUTH == SOUTH {
                (corners.3.0 .. corners.2.0).for_each(|x| {
                    img.put_pixel(x, corners.2.1, Rgb([0, 0, 0]));
                })
            }
            if *cell & EAST == EAST {
                (corners.1.1 .. corners.2.1).for_each(|y| {
                    img.put_pixel(corners.1.0, y, Rgb([0, 0, 0]));
                })
            }
            if *cell & WEST == WEST {
                (corners.0.1 .. corners.3.1).for_each(|y| {
                    img.put_pixel(corners.0.0, y, Rgb([0, 0, 0]));
                })
            }
        })
    });

    img.save("maze.png").expect("Échec");
}

fn main() {

    tracing_subscriber::fmt::fmt().init();


    let mut random = rand::rng();

    let mut maze = Maze::new(30, 30);
    let starting_cell = (
        random.random_range(0..maze.width),
        random.random_range(0..maze.height),
    );

    maze.generate_maze(starting_cell);

    generate_image(&maze);

    info!(
        "Maze created with width: {} and height: {}",
        maze.width, maze.height
    );
}
