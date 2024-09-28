use rand::Rng;

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
