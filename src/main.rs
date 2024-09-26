use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{read, Event, KeyCode, KeyEventKind},
    execute, queue,
    style::Print,
    terminal::{Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
};
use rand::{seq::SliceRandom, Rng};
use std::io::{self, Read, Write};

struct Coordinate {
    x: usize,
    y: usize,
}

enum FogOfWar {
    Coordinate(Coordinate),
}

struct Mob {
    tag: String,
    pos: Coordinate,
    strength: u8,
    hp: u8,
}

struct Vec2d {
    x: isize,
    y: isize,
}

struct DungeonFloor {
    height: usize,
    width: usize,
    fog_of_wars: Vec<FogOfWar>,
    mobs: Vec<Mob>,
}

impl DungeonFloor {
    fn fog_of_war_maskmap(&self) -> Vec<Vec<bool>> {
        let mut maskmap = vec![vec![false; self.height]; self.height];
        for fog_of_war in &self.fog_of_wars {
            match fog_of_war {
                FogOfWar::Coordinate(coord) => {
                    maskmap[coord.y][coord.x] = true;
                }
            }
        }
        maskmap
    }
    fn mob_maskmap(&self) -> Vec<Vec<bool>> {
        let mut maskmap = vec![vec![false; self.height]; self.height];
        for mob in &self.mobs {
            maskmap[mob.pos.y][mob.pos.x] = true;
        }
        maskmap
    }
}

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

fn main() {
    let mut stdout = io::stdout();
    println!("Ascension Army");
    println!("Created by elprebit");
    pause_for_input();
    let mut dungeon_floor = DungeonFloor {
        height: 16,
        width: 16,
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
    execute!(stdout, Hide, EnterAlternateScreen).unwrap();
    'game: loop {
        let event = read().unwrap();
        let enemy_map = dungeon_floor.mob_maskmap();
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
                for mob in &mut dungeon_floor.mobs {
                    if mob.tag == "player" {
                        let new_x =
                            if let Some(new_x) = mob.pos.x.checked_add_signed(player_movement.x) {
                                if new_x > dungeon_floor.width - 1 {
                                    dungeon_floor.width - 1
                                } else {
                                    new_x
                                }
                            } else {
                                mob.pos.x
                            };
                        let new_y =
                            if let Some(new_y) = mob.pos.y.checked_add_signed(player_movement.y) {
                                if new_y > dungeon_floor.height - 1 {
                                    dungeon_floor.height - 1
                                } else {
                                    new_y
                                }
                            } else {
                                mob.pos.y
                            };
                        if !enemy_map[new_y][new_x] {
                            mob.pos.y = new_y;
                            mob.pos.x = new_x;
                        }
                    }
                }
            }
            _ => (),
        }
        player_movement.x = 0;
        player_movement.y = 0;
        let mut map_display = vec![vec!['.'; dungeon_floor.width]; dungeon_floor.height];
        for mob in &dungeon_floor.mobs {
            match mob.tag.as_str() {
                "enemy" => {
                    map_display[mob.pos.y][mob.pos.x] = 'E';
                }
                "player" => {
                    map_display[mob.pos.y][mob.pos.x] = '@';
                }
                _ => (),
            }
        }
        queue!(stdout, Clear(ClearType::All)).unwrap();
        queue!(stdout, MoveTo(0, 0), Print("left: h, down: j, up: k, right: l, leftup: u, leftdown: b, rightup: y, rightdown: n")).unwrap();
        for (y, row) in map_display.iter().enumerate() {
            for (x, cell) in row.iter().enumerate() {
                queue!(stdout, MoveTo(x as u16, y as u16 + 1), Print(cell)).unwrap();
            }
        }
        stdout.flush().unwrap();
    }
    execute!(stdout, Show, LeaveAlternateScreen,).unwrap();
}
