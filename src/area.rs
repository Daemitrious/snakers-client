use crate::{Key, Key::*};

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

    pub fn can_move(&self, key: Key, position: usize) -> Option<usize> {
        let mut new = position.clone();

        match key {
            W => {
                if new / self.rows > 0 {
                    new -= self.rows;
                }
            }
            A => {
                if new % self.columns > 0 {
                    new -= 1;
                }
            }
            S => {
                if new / self.rows < self.rows - 1 {
                    new += self.rows;
                }
            }
            D => {
                if new % self.columns < self.columns - 1 {
                    new += 1;
                }
            }
            _ => unreachable!(),
        }

        if position != new {
            Some(new)
        } else {
            None
        }
    }
}
