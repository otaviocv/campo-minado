use bevy::prelude::*;
use bevy::sprite::{MaterialMesh2dBundle, Mesh2dHandle};
use bevy::window::PrimaryWindow;

use rand::seq::SliceRandom;
use rand::thread_rng;
use std::fmt::Display;
use std::fmt::Formatter;

fn main() {
    App::new().add_plugins((DefaultPlugins, MyPlugin)).run();
}

const CELL_SIZE: f32 = 40.0;

#[derive(Debug, PartialEq, Eq, Clone)]
enum BoardValue {
    Empty,
    Bomb,
    Clue(i32),
}

#[derive(Debug, Resource)]
struct Board {
    height: i32,
    width: i32,
    values: Vec<Vec<BoardValue>>,
}

#[derive(Debug, Resource)]
struct BoardInstance;

#[derive(Debug, Resource)]
struct BoardMaskInstance;

impl Display for BoardValue {
    fn fmt(&self, formatter: &mut Formatter) -> Result<(), std::fmt::Error> {
        match self {
            BoardValue::Empty => write!(formatter, "."),
            BoardValue::Bomb => write!(formatter, "B"),
            BoardValue::Clue(clue) => write!(formatter, "{}", clue),
        }?;

        Ok(())
    }
}

impl Display for Board {
    fn fmt(&self, formatter: &mut Formatter) -> Result<(), std::fmt::Error> {
        write!(formatter, "{}x{}\n", self.height, self.width)?;

        for i in 0..self.height {
            for j in 0..self.width {
                write!(formatter, "{}", self.values[i as usize][j as usize])?;
            }
            write!(formatter, "\n")?;
        }
        Ok(())
    }
}

pub struct MyPlugin;

impl Plugin for MyPlugin {
    fn build(&self, app: &mut App) {
        let height = 9;
        let width = 9;
        let number_of_bombs = 10;

        let board = build_random_board(&height, &width, &number_of_bombs);
        let board_with_clues = build_clues(&board);
        println!("{}", board_with_clues);

        let board_mask = build_mask(&height, &width);
        println!("{}", board_mask);

        app.insert_resource(board_with_clues)
            .insert_resource(board_mask)
            .insert_resource(GridPosition { position: None })
            .add_systems(Startup, add_camera)
            .add_systems(Update, (update_grid_postition, draw_board).chain());
    }
}

fn add_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn build_random_board(height: &i32, width: &i32, number_of_bombs: &i32) -> Board {
    let mut coordinates: Vec<(i32, i32)> = Vec::new();

    for i in 0..*height {
        for j in 0..*width {
            coordinates.push((i, j));
        }
    }

    coordinates.shuffle(&mut thread_rng());
    let bombs = coordinates[0usize..*number_of_bombs as usize].to_vec();

    let mut board: Vec<Vec<BoardValue>> = (0..*height)
        .map(|_| (0..*width).map(|_| BoardValue::Empty).collect())
        .collect();

    for bomb in bombs.iter() {
        board[bomb.0 as usize][bomb.1 as usize] = BoardValue::Bomb;
    }

    Board {
        height: height.clone(),
        width: width.clone(),
        values: board,
    }
}

fn build_clues(board: &Board) -> Board {
    let mut new_board = board.values.clone();
    for i in 0..board.height {
        for j in 0..board.width {
            if board.values[i as usize][j as usize] == BoardValue::Bomb {
                continue;
            }
            let neighbors = get_neighbors(&i, &j);
            let mut clue = 0;
            for neighbor in neighbors.iter() {
                let row = board
                    .values
                    .get(neighbor.0 as usize)
                    .unwrap_or(&vec![])
                    .clone();
                let neighbor_value = row.get(neighbor.1 as usize).unwrap_or(&BoardValue::Empty);
                if neighbor_value == &BoardValue::Bomb {
                    clue += 1;
                }

                if clue > 0 {
                    new_board[i as usize][j as usize] = BoardValue::Clue(clue);
                }
            }
        }
    }

    Board {
        values: new_board,
        height: board.height.clone(),
        width: board.width.clone(),
    }
}

fn get_neighbors(row_index: &i32, column_index: &i32) -> Vec<(i32, i32)> {
    vec![
        (row_index - 1, column_index - 1),
        (row_index - 1, *column_index),
        (row_index - 1, column_index + 1),
        (*row_index, column_index - 1),
        (*row_index, column_index + 1),
        (row_index + 1, column_index - 1),
        (row_index + 1, *column_index),
        (row_index + 1, column_index + 1),
    ]
}

#[derive(Clone, PartialEq, Eq)]
enum MaskValue {
    Closed,
    Open,
    Flagged,
    Question,
}

#[derive(Resource)]
struct BoardMask {
    height: i32,
    width: i32,
    values: Vec<Vec<MaskValue>>,
}

fn build_mask(height: &i32, width: &i32) -> BoardMask {
    let board: Vec<Vec<MaskValue>> = (0..*height)
        .map(|_| (0..*width).map(|_| MaskValue::Closed).collect())
        .collect();

    BoardMask {
        height: height.clone(),
        width: width.clone(),
        values: board,
    }
}

impl Display for MaskValue {
    fn fmt(&self, formatter: &mut Formatter) -> Result<(), std::fmt::Error> {
        match self {
            MaskValue::Closed => write!(formatter, "."),
            MaskValue::Open => write!(formatter, " "),
            MaskValue::Flagged => write!(formatter, "F"),
            MaskValue::Question => write!(formatter, "?"),
        }?;

        Ok(())
    }
}

impl Display for BoardMask {
    fn fmt(&self, formatter: &mut Formatter) -> Result<(), std::fmt::Error> {
        write!(formatter, "{}x{}\n", self.height, self.width)?;

        for i in 0..self.height {
            for j in 0..self.width {
                write!(formatter, "{}", self.values[i as usize][j as usize])?;
            }
            write!(formatter, "\n")?;
        }
        Ok(())
    }
}

fn draw_board(
    mut commands: Commands,
    mut board: Res<Board>,
    mut mask: Res<BoardMask>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
    grid_position: Res<GridPosition>,
) {
    let green = Color::hsl(90.0, 0.95, 0.7);
    let red = Color::hsl(0.0, 0.95, 0.7);
    let padding = 0.0;

    let font = asset_server.load("fonts/Swansea.ttf");
    let text_justification = JustifyText::Center;
    let text_style = TextStyle {
        font: font.clone(),
        font_size: 10.0,
        color: Color::WHITE,
    };
    let start_x = -CELL_SIZE * board.width as f32 / 2.0;
    let start_y = CELL_SIZE * board.height as f32 / 2.0;
    for i in 0..board.height {
        for j in 0..board.width {
            let value = board.values[i as usize][j as usize].clone();
            let mask_value = mask.values[i as usize][j as usize].clone();

            if mask_value == MaskValue::Open {
                commands.spawn(Text2dBundle {
                    text: Text::from_section(value.to_string(), text_style.clone())
                        .with_justify(text_justification),
                    transform: Transform::from_xyz(
                        start_x + j as f32 * (CELL_SIZE + padding),
                        start_y - i as f32 * (CELL_SIZE + padding),
                        1.0,
                    ),
                    ..default()
                });
            }

            let mut color = green;
            if let Some(grid_coordinates) = &grid_position.position {
                if grid_coordinates.x == j && grid_coordinates.y == i {
                    color = red;
                    println!("Position {:?} is red", grid_coordinates);
                }
            }

            let shape = Mesh2dHandle(meshes.add(Rectangle::new(CELL_SIZE, CELL_SIZE)));
            commands.spawn(MaterialMesh2dBundle {
                mesh: shape,
                material: materials.add(color),
                transform: Transform::from_xyz(
                    start_x + j as f32 * (CELL_SIZE + padding),
                    start_y - i as f32 * (CELL_SIZE + padding),
                    0.0,
                ),
                ..default()
            });
        }
    }
}

#[derive(Debug)]
struct GridCoordinates {
    x: i32,
    y: i32,
}

#[derive(Resource, Debug)]
struct GridPosition {
    position: Option<GridCoordinates>,
}

fn update_grid_postition(
    mut position: ResMut<GridPosition>,
    board: Res<Board>,
    q_windows: Query<&Window, With<PrimaryWindow>>,
) {
    let window_size = Vec2::new(q_windows.single().width(), q_windows.single().height());
    if let Some(cursor_position) = q_windows.single().cursor_position() {
        let world_position = Vec2::new(
            cursor_position.x - window_size.x / 2.,
            cursor_position.y - window_size.y / 2.,
        );
        let board_coordinates = GridCoordinates {
            x: (0.5 + (board.width as f32 / 2.0) + world_position.x / CELL_SIZE) as i32,
            y: (0.5 + (board.height as f32 / 2.0) + world_position.y / CELL_SIZE) as i32,
        };
        position.position = Some(board_coordinates);
    } else {
        position.position = None;
    }
}
