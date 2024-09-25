use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{read, Event, KeyCode, KeyEventKind},
    execute, queue,
    style::Print,
    terminal::{Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
};
use rand::{rngs::ThreadRng, seq::SliceRandom, Rng};
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
    min_points_num: Option<usize>,
    from_x: usize,
    from_y: usize,
    width: usize,
    height: usize,
) -> Vec<Coordinate> {
    //! from_x <= x <= from_x + height、from_y <= y <= from_y + height
    //! from_x <= x <= from_x + height、from_y <= y <= from_y + height
    //! の範囲で 重複なく必要座標数分ランダムな座標のVecを生成する。
    //! 実装：領域の全ての座標を一つのVecに格納してシャッフル、必要な座標の数だけ取り出す。
    let mut rng = rand::thread_rng();
    let points_num = if let Some(min_points_num) = min_points_num {
        if min_points_num > width * height { panic!("points overflows area size") };
        min_points_num
    } else {
        rng.gen_range(0..width * height)
    };
    let mut points: Vec<Coordinate> = Vec::new();
    for y in from_y..from_y+height {
        for x in from_x..from_x+width {
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
        assert_eq!(6, gen_rand_points_in_area(Some(6), 0, 0, 6, 6).len());
        // assert_eq!(0, gen_rand_points_in_area(0, 0, 0, 0, 0).len());
    }
    #[test]
    fn test_random_points_in_points_fullfil_area() {
        // let mut rng = rand::thread_rng();
        // rng.gen_range(15..0);
        assert_eq!(9, gen_rand_points_in_area(Some(9), 0, 0, 3, 3).len());
    }
    #[test]
    #[should_panic]
    fn test_random_points_in_area_points_num_overflows_area_size() {
        gen_rand_points_in_area(Some(6), 0, 0, 2, 2).len();
    }
}

fn main() {
    let mut stdout = io::stdout();
    println!("Ascension Army");
    println!("Created by elprebit");
    pause_for_input();
    let dungeon_floor = DungeonFloor {
        height: 16,
        width: 16,
        fog_of_wars: vec![],
        mobs: vec![],
    };
    let mut mob_list: Vec<Mob> = Vec::new();
    let mut rng = rand::thread_rng();
    let enemy_coordinates =
        gen_rand_points_in_area(None, 0, 0, dungeon_floor.width, dungeon_floor.height);
    for coordinates in enemy_coordinates {
        mob_list.push(Mob {
            tag: String::from("enemy"),
            pos: coordinates,
            strength: 1,
            hp: 1,
        });
    }
    let mut player = Mob {
        tag: String::from("Player"),
        pos: Coordinate {
            x: rng.gen_range(0..dungeon_floor.width),
            y: rng.gen_range(0..dungeon_floor.height),
        },
        strength: 2,
        hp: 3,
    };
    let mut player_movement = Vec2d { x: 0, y: 0 };
    execute!(stdout, Hide, EnterAlternateScreen).unwrap();
    'game: loop {
        let event = read().unwrap();
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
                        _ => (),
                    }
                }
                if let Some(new_x) = player.pos.x.checked_add_signed(player_movement.x) {
                    player.pos.x = if new_x >= dungeon_floor.width {
                        dungeon_floor.width - 1
                    } else {
                        new_x
                    }
                }
                if let Some(new_y) = player.pos.y.checked_add_signed(player_movement.y) {
                    player.pos.y = if new_y >= dungeon_floor.height {
                        dungeon_floor.height - 1
                    } else {
                        new_y
                    }
                }
            }
            _ => (),
        }
        player_movement.x = 0;
        player_movement.y = 0;
        let mut map_display = vec![vec!['.'; dungeon_floor.width]; dungeon_floor.height];
        for mob in &mut mob_list {
            match mob.tag.as_str() {
                "enemy" => {
                    map_display[mob.pos.y][mob.pos.x] = 'E';
                }
                _ => (),
            }
        }
        map_display[player.pos.y][player.pos.x] = '@';
        queue!(stdout, Clear(ClearType::All)).unwrap();
        for (y, row) in map_display.iter().enumerate() {
            for (x, cell) in row.iter().enumerate() {
                queue!(stdout, MoveTo(x as u16, y as u16 + 1), Print(cell)).unwrap();
            }
        }
        stdout.flush().unwrap();
    }
    execute!(stdout, Show, LeaveAlternateScreen,).unwrap();
}
