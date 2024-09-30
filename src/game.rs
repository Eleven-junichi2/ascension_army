use rand::{seq::SliceRandom, Rng};

#[derive(Debug, PartialEq)]
pub struct Coordinate {
    pub x: usize,
    pub y: usize,
}

pub enum FogOfWar {
    Coordinate(Coordinate),
}

pub struct Mob {
    pub tag: String,
    pub pos: Coordinate,
    pub strength: u8,
    pub hp: u8,
}

impl Mob {
    pub fn calc_combat(&self, other: &Mob) -> bool {
        //! Return true if this mob wins the combat, false otherwise.
        let mut rng = rand::thread_rng();
        let result = rng.gen_ratio(
            self.strength as u32,
            (self.strength + other.strength) as u32,
        );
        result
    }
}

pub struct Vec2d {
    pub x: isize,
    pub y: isize,
}

pub struct DungeonFloor {
    pub height: usize,
    pub width: usize,
    pub fog_of_wars: Vec<FogOfWar>,
    pub mobs: Vec<Mob>,
}

impl DungeonFloor {
    pub fn fog_of_war_maskmap(&self) -> Vec<Vec<bool>> {
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
    pub fn mob_maskmap(&self, tags: &[&str]) -> Vec<Vec<bool>> {
        let mut map = vec![vec![false; self.height]; self.height];
        for mob in &self.mobs {
            if tags.contains(&&mob.tag.as_str()) {
                map[mob.pos.y][mob.pos.y] = true;
            }
        }
        map
    }
    pub fn mob_index_by_tag(&self, tag: &str) -> Option<usize> {
        for i in 0..self.mobs.len() {
            if self.mobs[i].tag == tag {
                return Some(i);
            }
        }
        None
    }
}

pub fn gen_rand_points_in_area(
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
