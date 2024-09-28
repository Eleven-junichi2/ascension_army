// TODO: Entity management with id
mod game;
use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{read, Event, KeyCode, KeyEventKind},
    execute, queue,
    style::Print,
    terminal::{self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
};
use game::{Coordinate, DungeonFloor, Mob, Vec2d};
use rand::{seq::SliceRandom, Rng};
use std::io::{self, Read, Write};

fn pause_for_input() {
    let _ = io::stdin().read(&mut [0u8]).unwrap();
}

fn gen_rand_points_in_area(
    from_x: usize,
    from_y: usize,
    width: usize,
    height: usize,
    how_many_generate: Option<usize>,
    max_num_of_rand_gen_points: Option<usize>,
    exclusive_points: Vec<Coordinate>,
) -> Vec<Coordinate> {
    //! from_x <= x <= from_x + height、from_y <= y <= from_y + height
    //! from_x <= x <= from_x + height、from_y <= y <= from_y + height
    //! の範囲で 重複なく必要座標数分ランダムな座標のVecを生成する。
    //! 実装：領域の全ての座標を一つのVecに格納してシャッフル、必要な座標の数だけ取り出す。
    //! width * height - how_many_generate < exclusive_points.len()ならpanic!
    //! exclusive_pointsの実装
    //! pointsからexclusive_pointsの座標を除外すればよい
    //! # Panics
    //! If `width * height - how_many_generate < exclusive_points.len()` or `how_many_generate > width * height`.
    let mut rng = rand::thread_rng();
    let points_num = if let Some(how_many_generate) = how_many_generate {
        if width * height - how_many_generate < exclusive_points.len() {
            panic!("num of points to be excluded is too large to satisfy num of points to be generated")
        };
        if how_many_generate > width * height {
            panic!("points overflows area size")
        };
        how_many_generate
    } else {
        if let Some(max_num) = max_num_of_rand_gen_points {
            rng.gen_range(0..max_num)
        } else {
            rng.gen_range(0..width * height)
        }
    };
    let mut points: Vec<Coordinate> = Vec::new();
    for y in from_y..from_y + height {
        for x in from_x..from_x + width {
            if exclusive_points
                .iter()
                .any(|coord| coord.x == x && coord.y == y)
            {
                continue;
            }
            points.push(Coordinate { x: x, y: y });
        }
    }
    points.shuffle(&mut rng);
    points.truncate(points_num);
    points
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_random_points_in_area_valid_points_num() {
        assert_eq!(
            6,
            gen_rand_points_in_area(0, 0, 6, 6, Some(6), None, vec![]).len(),
        );
    }
    #[test]
    fn test_random_points_in_points_fullfil_area() {
        assert_eq!(
            9,
            gen_rand_points_in_area(0, 0, 3, 3, Some(9), None, vec![]).len()
        );
    }
    #[test]
    #[should_panic]
    fn test_random_points_in_area_points_num_overflows_area_size() {
        gen_rand_points_in_area(0, 0, 2, 2, Some(6), None, vec![]).len();
    }
}

const GAME_NAME: &str = "Ascension Army";

fn main() {
    // let mut stdout = io::stdout();


    // queue!(stdout, Hide, EnterAlternateScreen).unwrap();
    // let (terminal_width, terminal_height) = terminal::size().unwrap();
    // queue!(
    //     stdout,
    //     MoveTo(
    //         terminal_width / 2 - (GAME_NAME.len() / 2) as u16,
    //         terminal_height / 2 - 1
    //     ),
    //     Print("Ascension Army\n")
    // )
    // .unwrap();
    // let show_how_to_start = "- Press any key to start -";
    // queue!(
    //     stdout,
    //     MoveTo(
    //         terminal_width / 2 - (show_how_to_start.len() / 2) as u16,
    //         terminal_height / 2
    //     ),
    //     Print(show_how_to_start)
    // )
    // .unwrap();
    // stdout.flush().unwrap();
    // pause_for_input();
    let mut dungeon_floor = DungeonFloor {
        height: 32,
        width: 32,
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
    'game: loop {
        let event = read().unwrap();
        // let mut enemy_map = dungeon_floor.mut_mob_map(&["enemy"]);
        match event {
            Event::Key(event) => {
                if event.kind == KeyEventKind::Press {
                    match event.code {
                        KeyCode::Esc => break,
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
                                } else {
                                    player.hp -= 1;
                                }
                                if mob.hp > 0 {
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
        // queue!(stdout, Clear(ClearType::All)).unwrap();
        let mut dead = vec![];
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
                        // queue!(
                        //     stdout,
                        //     MoveTo(0, 1),
                        //     Print(format!(
                        //         "Player: hp={} pos=({},{})",
                        //         mob.hp, mob.pos.x, mob.pos.y
                        //     ))
                        // )
                        // .unwrap();
                    }
                    _ => (),
                }
            }
        }
        for i in dead.iter() {
            dungeon_floor.mobs.remove(*i);
        }
        // queue!(stdout, MoveTo(0, 0), Print("left: h, down: j, up: k, right: l, leftup: u, leftdown: b, rightup: y, rightdown: n")).unwrap();
        let offset_x_to_display = 0;
        let offset_y_to_display = 2;
        // for (y, row) in map_display.iter().enumerate() {
        //     queue!(
        //         stdout,
        //         MoveTo(offset_x_to_display, y as u16 + offset_y_to_display)
        //     )
        //     .unwrap();
        //     let mut row_string = String::new();
        //     for cell in row.iter() {
        //         row_string.push(*cell);
        //         row_string.push(' ');
        //     }
        //     queue!(stdout, Print(row_string)).unwrap();
        // }
        // stdout.flush().unwrap();
    }
    // execute!(stdout, Show, LeaveAlternateScreen,).unwrap();
}
