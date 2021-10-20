use crate::{area::Area, TcpStream};

pub struct Client {
    pub stream: TcpStream,
    pub area: Area,
}

impl Client {
    pub fn format(&self) -> Vec<u8> {
        let l = (self.area.columns + 2) * 2;
        let a = (self.area.rows + 2) * l;

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
                            let v = self.area.data[i];
                            i += 1;
                            v
                        }
                    }
                }
            })
            .collect()
    }
}
