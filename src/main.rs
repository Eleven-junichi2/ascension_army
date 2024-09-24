use crossterm::{
    event::{read, Event, KeyCode, KeyEventKind},
    execute,
    style::Print,
};
use std::io::{self, Read};

struct Coordinate {
    x: usize,
    y: usize,
}

enum FogOfWar {
    Coordinate(Coordinate),
}

struct Mob<'a> {
    kind: &'a str,
    pos: Coordinate,
    strength: u8,
    hp: u8,
}

struct Vec2d {
    x: isize,
    y: isize,
}

fn pause_for_input() {
    let _ = io::stdin().read(&mut [0u8]).unwrap();
}

fn main() {
    let mut stdout = io::stdout();
    println!("Ascension Army");
    println!("Created by elprebit");
    pause_for_input();

    let mut mob_list: Vec<Mob> = Vec::new();
    let mut player = Mob {
        kind: "player",
        pos: Coordinate { x: 0, y: 0 },
        strength: 2,
        hp: 3,
    };
    let mut player_movement = Vec2d { x: 0, y: 0 };
    'game: loop {
        let event = read().unwrap();
        match event {
            Event::Key(event) if event.kind == KeyEventKind::Press => {
                println!("{}", event.code);
                if event.code == KeyCode::Esc {
                    break;
                }
                match event.code {
                    KeyCode::Esc => break,
                    KeyCode::Char('h') => {
                        player_movement.x -= 1;
                    }
                    KeyCode::Char('l') => {
                        player_movement.x += 1;
                    }
                    KeyCode::Char('k') => {
                        player_movement.y -= 1;
                    }
                    KeyCode::Char('j') => {
                        player_movement.y += 1;
                    }
                    KeyCode::Char('y') => {
                        player_movement.x -= 1;
                        player_movement.y -= 1;
                    }
                    KeyCode::Char('u') => {
                        player_movement.x += 1;
                        player_movement.y -= 1;
                    },
                    KeyCode::Char('b') => {
                        player_movement.x -= 1;
                        player_movement.y += 1;
                    }
                    KeyCode::Char('n') => {
                        player_movement.x += 1;
                        player_movement.y += 1;
                    }
                    _ => (),
                }
                if let Some(new_x) = player.pos.x.checked_add_signed(player_movement.x) {
                    player.pos.x = new_x;
                }
                if let Some(new_y) = player.pos.y.checked_add_signed(player_movement.y) {
                    player.pos.y = new_y;
                }
            }
            _ => (),
        }
        player_movement.x = 0;
        player_movement.y = 0;
        execute!(
            stdout,
            Print(format!("Player: x={}, y={}\n", player.pos.x, player.pos.y))
        )
        .unwrap();
    }
}
