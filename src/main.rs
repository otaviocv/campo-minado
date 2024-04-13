use bevy::prelude::*;

use rand::seq::SliceRandom;
use rand::thread_rng;
use std::fmt::Display;
use std::fmt::Formatter;

fn main() {
    App::new().add_plugins((DefaultPlugins, HelloPlugin)).run();
}

#[derive(Component)]
struct Person;

#[derive(Component)]
struct Name(String);

pub struct HelloPlugin;

impl Plugin for HelloPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (add_people, build_board));
    }
}

fn add_people(mut commands: Commands) {
    commands.spawn((Person, Name("Elaina Proctor".to_string())));
    commands.spawn((Person, Name("Renzo Hume".to_string())));
    commands.spawn((Person, Name("Zayna Nieves".to_string())));
}

fn update_people(mut query: Query<&mut Name, With<Person>>) {
    for mut name in &mut query {
        if name.0 == "Elaina Proctor" {
            name.0 = "Elaina Hume".to_string();
            break; // We don’t need to change any other names
        }
    }
}

fn hello_world() {
    println!("hello world!");
}

fn greet_people(query: Query<&Name, With<Person>>) {
    for name in &query {
        println!("hello {}!", name.0);
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
enum Value {
    Empty,
    Bomb,
    Clue(i32),
}

#[derive(Debug)]
struct Board {
    height: i32,
    width: i32,
    values: Vec<Vec<Value>>,
}

impl Display for Value {
    fn fmt(&self, formatter: &mut Formatter) -> Result<(), std::fmt::Error> {
        match self {
            Value::Empty => write!(formatter, "."),
            Value::Bomb => write!(formatter, "B"),
            Value::Clue(clue) => write!(formatter, "{}", clue),
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

fn build_board() {
    let height = 9;
    let width = 9;
    let number_of_bombs = 10;

    let board = build_random_board(&height, &width, &number_of_bombs);
    let board_with_clues = build_clues(board);
    println!("{}", board_with_clues)
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

    let mut board: Vec<Vec<Value>> = (0..*height)
        .map(|_| (0..*width).map(|_| Value::Empty).collect())
        .collect();

    for bomb in bombs.iter() {
        board[bomb.0 as usize][bomb.1 as usize] = Value::Bomb;
    }

    Board {
        height: height.clone(),
        width: width.clone(),
        values: board,
    }
}

fn build_clues(board: Board) -> Board {
    let mut new_board = board.values.clone();
    for i in 0..board.height {
        for j in 0..board.width {
            if board.values[i as usize][j as usize] == Value::Bomb {
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
                let neighbor_value = row.get(neighbor.1 as usize).unwrap_or(&Value::Empty);
                if neighbor_value == &Value::Bomb {
                    clue += 1;
                }

                if clue > 0 {
                    new_board[i as usize][j as usize] = Value::Clue(clue);
                }
            }
        }
    }

    Board {
        values: new_board,
        ..board
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
