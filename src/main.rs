use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{read, Event, KeyCode, KeyEventKind},
    execute, queue,
    style::Print,
    terminal::{Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
};
use rand::{rngs::ThreadRng, Rng};
use std::io::{self, Read, Write};
use thiserror::Error;

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
    //! の座標平面上の矩形において、y=from_y..heightを各rowに分割し、それぞれの列に対し
    //! 合計すると必要座標数をを満たすようにランダムで座標数を配分する。
    //! 配分する座標数how_many_point_each_rowの決め方
    //! 000
    //! 000
    //! 000 のような矩形を例にする。
    //! min_points_numを8とする
    //! r=0で010の時残りmin_points_numは7 次の列以降の座標空き数は6 このとき 残りmin_points_num <= 座標空き数を満たさないから、r=0の施行をやり直す。
    //! ここで、次回の施行は座標空き数-残りmin_points_numをhow_many_points_in_rowの最小値とする。
    //! how_many_points_in_row==widthとなるなら、乱数生成をスキップして直接how_many_points_in_row=widthとする。
    //! min_points_numを3とする。
    //! r=0で000の時 「残りmin_points_num」=min_points_num-points.len() は3 次の列以降の座標空き数は6 このとき 残りmin_points_num <= 座標空き数を満たすから、r=1へ
    //! r=1で011の時残りmin_points_numは1 次の列以降の座標空き数は3 このとき 残りmin_points_num <= 座標空き数を満たすから、r=2へ
    //! r=2で111の時残りmin_points_numは-2 残りmin_points_num >= 0を満たさないから、how_many_points_in_rowはmin_points_num-points.len()を割り当てる
    let mut rng: ThreadRng;
    let mut points = Vec::new();
    let min_points_num = if let Some(num) = min_points_num {
        num
    } else {
        rng = rand::thread_rng();
        rng.gen_range(0..width * height)
    };
    if min_points_num > width * height {
        panic!("The given number of points overflows the area size.");
    } else if min_points_num != 0 {
        rng = rand::thread_rng();
        'generate_points: for row in 0..height {
            let mut how_many_points_in_row;
            let mut min_how_many_points_in_row = 0;
            loop {
                let avaliable_points_left_from_next_row = width * (height - (row + 1));
                if min_how_many_points_in_row == width {
                    how_many_points_in_row = width;
                } else {
                    how_many_points_in_row = rng.gen_range(min_how_many_points_in_row..=width);
                }
                dbg!(
                    min_points_num,
                    points.len(),
                    how_many_points_in_row,
                    avaliable_points_left_from_next_row
                );
                let remaining_unset_points = if let Some(result) =
                    min_points_num.checked_sub(points.len() + how_many_points_in_row)
                {
                    result
                } else {
                    min_points_num - points.len()
                };
                if remaining_unset_points <= avaliable_points_left_from_next_row {
                    break;
                } else {
                    min_how_many_points_in_row = width;
                }
            }
            for _ in 0..how_many_points_in_row {
                let x = rng.gen_range(from_x..from_x + width);
                let y = from_y + row;
                points.push(Coordinate { x, y });
                if points.len() == min_points_num {
                    break 'generate_points;
                }
            }
        }
    }
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
    let enemy_coordinates = gen_rand_points_in_area(
        None,
        0,
        0,
        dungeon_floor.width,
        dungeon_floor.height,
    );
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
