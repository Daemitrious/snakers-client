use crate::{Direction, Direction::*};

pub struct Area {
    pub rows: usize,
    pub columns: usize,
    pub data: Vec<u8>,
}

impl Area {
    pub fn format(&self) -> Vec<u8> {
        let l = (self.columns + 2) * 2;
        let a = (self.rows + 2) * l;

        let er = a - l;
        let ec = l - 1;

        let mut i = 0;

        (1..a)
            .map(|n| {
                if n < l || n > er {
                    if n % 2 == 0 {
                        32
                    } else {
                        45
                    }
                } else {
                    let m = n % l;

                    if m == 0 {
                        10
                    } else if m == 1 || m == ec {
                        45
                    } else {
                        if n % 2 == 0 {
                            32
                        } else {
                            let v = self.data[i];
                            i += 1;
                            v
                        }
                    }
                }
            })
            .collect()
    }

    pub fn can_move(&self, direction: Direction, area_i: usize) -> Option<usize> {
        let mut p = area_i.clone();

        match direction {
            W => {
                if p / self.rows > 0 {
                    p -= self.rows;
                }
            }
            A => {
                if p % self.columns > 0 {
                    p -= 1;
                }
            }
            S => {
                if p / self.rows < self.rows - 1 {
                    p += self.rows;
                }
            }
            D => {
                if p % self.columns < self.columns - 1 {
                    p += 1;
                }
            }
        }

        if p != area_i && self.data[p] != 111 {
            Some(p)
        } else {
            None
        }
    }
}
