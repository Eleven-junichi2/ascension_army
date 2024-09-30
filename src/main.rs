// TODO: Entity management with id
// TODO: Turn-based game loop
mod game;
use crossterm::{
    event::{read, Event, KeyCode, KeyEventKind},
};
use game::*;
use rand::Rng;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    text::{Line, Text},
    widgets::Paragraph,
};
use std::{
    io::{self, Read},
};

fn pause_for_input() {
    let _ = io::stdin().read(&mut [0u8]).unwrap();
}



struct GameMessage {
    logs: Vec<String>,
    is_locking: bool,
}

impl GameMessage {
    fn send(&mut self, msg: &str) -> bool {
        if self.is_locking {
            false
        } else {
            self.logs.push(msg.to_string());
            true
        }
    }
    fn show(&mut self) -> Option<String> {
        self.logs.pop()
    }
    fn lock(&mut self) {
        self.is_locking = true
    }
    fn unlock(&mut self) {
        self.is_locking = false;
    }
}

impl Default for GameMessage {
    fn default() -> Self {
        Self {
            logs: vec![],
            is_locking: false,
        }
    }
}

const GAME_NAME: &str = "Ascension Army";

fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();
    terminal.draw(|frame| {
        let title = Paragraph::new(GAME_NAME).alignment(Alignment::Center);
        let how_to_start =
            Paragraph::new("- Press any key to start -").alignment(Alignment::Center);
        let chunks = Layout::vertical([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(frame.area());
        frame.render_widget(title, chunks[0]);
        frame.render_widget(how_to_start, chunks[1]);
    })?;
    pause_for_input();
    terminal.clear()?;
    let mut dungeon_floor = DungeonFloor {
        height: 24,
        width: 24,
        fog_of_wars: vec![],
        mobs: vec![],
    };
    let mut rng = rand::thread_rng();
    let initial_player_pos = (
        rng.gen_range(0..dungeon_floor.width),
        rng.gen_range(0..dungeon_floor.height),
    );
    dungeon_floor.mobs.push(Mob {
        tag: String::from("player"),
        pos: Coordinate {
            x: initial_player_pos.0,
            y: initial_player_pos.1,
        },
        strength: 2,
        hp: 3,
    });
    let enemy_coordinates = gen_rand_points_in_area(
        0,
        0,
        dungeon_floor.width,
        dungeon_floor.height,
        None,
        Some(dungeon_floor.width * dungeon_floor.height - 1),
        vec![Coordinate {
            x: initial_player_pos.0,
            y: initial_player_pos.1,
        }],
    );
    for coordinates in enemy_coordinates {
        dungeon_floor.mobs.push(Mob {
            tag: String::from("enemy"),
            pos: coordinates,
            strength: 1,
            hp: 1,
        });
    }
    let mut player_movement = Vec2d { x: 0, y: 0 };
    let mut game_msg = GameMessage::default();
    'game: loop {
        let event = read().unwrap();
        match event {
            Event::Key(event) => {
                if event.kind == KeyEventKind::Press {
                    match event.code {
                        KeyCode::Esc => break Ok(()),
                        KeyCode::Char('h') => {
                            player_movement.x = -1;
                        }
                        KeyCode::Char('l') => {
                            player_movement.x = 1;
                        }
                        KeyCode::Char('k') => {
                            player_movement.y = -1;
                        }
                        KeyCode::Char('j') => {
                            player_movement.y = 1;
                        }
                        KeyCode::Char('y') => {
                            player_movement.x = -1;
                            player_movement.y = -1;
                        }
                        KeyCode::Char('u') => {
                            player_movement.x = 1;
                            player_movement.y = -1;
                        }
                        KeyCode::Char('b') => {
                            player_movement.x = -1;
                            player_movement.y = 1;
                        }
                        KeyCode::Char('n') => {
                            player_movement.x = 1;
                            player_movement.y = 1;
                        }
                        KeyCode::Left => {
                            player_movement.x = -1;
                        }
                        KeyCode::Right => {
                            player_movement.x = 1;
                        }
                        KeyCode::Up => {
                            player_movement.y = -1;
                        }
                        KeyCode::Down => {
                            player_movement.y = 1;
                        }
                        _ => (),
                    }
                }
                if let Some(player) = dungeon_floor.mob_index_by_tag("player") {
                    let (player_mobs, other_mobs) = dungeon_floor.mobs.split_at_mut(player + 1);
                    let player = &mut player_mobs[0];
                    let mut new_x =
                        if let Some(new_x) = player.pos.x.checked_add_signed(player_movement.x) {
                            if new_x > dungeon_floor.width - 1 {
                                dungeon_floor.width - 1
                            } else {
                                new_x
                            }
                        } else {
                            player.pos.x
                        };
                    let mut new_y =
                        if let Some(new_y) = player.pos.y.checked_add_signed(player_movement.y) {
                            if new_y > dungeon_floor.height - 1 {
                                dungeon_floor.height - 1
                            } else {
                                new_y
                            }
                        } else {
                            player.pos.y
                        };
                    for mob in other_mobs {
                        if mob.tag == "enemy" {
                            if mob.pos.x == new_x && mob.pos.y == new_y {
                                if player.calc_combat(mob) {
                                    mob.hp -= 1;
                                    game_msg.send(format!(
                                        "Player attacked the enemy! the foe's hp is now {}",
                                        mob.hp
                                    ).as_str());
                                } else {
                                    player.hp -= 1;
                                }
                                if mob.hp > 0 {
                                    game_msg.send(format!(
                                        "Enemy attacked Player! Player's hp is now {}",
                                        player.hp
                                    ).as_str());
                                    new_x = player.pos.x;
                                    new_y = player.pos.y;
                                }
                            }
                        }
                    }
                    player.pos.x = new_x;
                    player.pos.y = new_y;
                }
            }
            _ => (),
        }
        player_movement.x = 0;
        player_movement.y = 0;
        let mut map_display = vec![vec!['.'; dungeon_floor.width]; dungeon_floor.height];
        let mut dead = vec![];
        let mut player_status_text = String::new();
        for (i, mob) in dungeon_floor.mobs.iter().enumerate() {
            if mob.hp == 0 {
                dead.push(i);
            } else {
                match mob.tag.as_str() {
                    "enemy" => {
                        map_display[mob.pos.y][mob.pos.x] = 'E';
                    }
                    "player" => {
                        map_display[mob.pos.y][mob.pos.x] = '@';
                        player_status_text =
                            format!("Player: hp={} pos=({},{})", mob.hp, mob.pos.x, mob.pos.y);
                    }
                    _ => (),
                }
            }
        }
        for i in dead.iter() {
            dungeon_floor.mobs.remove(*i);
        }
        terminal.draw(|frame| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                Constraint::Percentage(5),
                Constraint::Percentage(5),
                Constraint::Percentage(80),
                Constraint::Percentage(10)])
                .split(frame.area());
            let howtoplay = Paragraph::new(
                "esc: exit game, left: h, down: j, up: k, right: l, leftup: u, leftdown: b, rightup: y, rightdown: n"
            ).alignment(Alignment::Center);
            let game_msg_lines = if let Some(lines) = game_msg.logs.last_chunk::<2>() {
                let mut lines_vec = vec![];
                for msg in lines.iter() {
                    lines_vec.push(Line::from(msg.clone()));
                }
                lines_vec
            } else {
                vec![Line::from(game_msg.logs.last().unwrap_or(&String::new()).to_string())]
            };
            // dbg!(&game_msg_lines);
            let message = Paragraph::new(Text::from(game_msg_lines)).alignment(Alignment::Center);
            let mut lines = vec![];
            for row in map_display.iter() {
                let mut row_string = String::new();
                for cell in row.iter() {
                    row_string.push(*cell);
                    row_string.push(' ');
                }
                lines.push(Line::from(row_string));
            }
            let map_widget = Paragraph::new(Text::from(lines)).alignment(Alignment::Center);
            let player_status = Paragraph::new(player_status_text).alignment(Alignment::Center);
            frame.render_widget(howtoplay, chunks[0]);
            frame.render_widget(message, chunks[1]);
            frame.render_widget(map_widget, chunks[2]);
            frame.render_widget(player_status, chunks[3]);
        })?;
    }
}
