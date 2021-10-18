use {
    console::Term,
    std::{
        io::{ErrorKind::WouldBlock, Read, Result, Write},
        net::TcpStream,
        sync::{Arc, RwLock},
        thread::{sleep, spawn},
        time::Duration,
    },
};

const W: u8 = 119;
const A: u8 = 97;
const S: u8 = 115;
const D: u8 = 100;
const Q: u8 = 113;

const EMPTY: u8 = 32;
const BORDER: u8 = 45;
const NEWLINE: u8 = 10;

struct Area {
    rows: usize,
    columns: usize,
    data: Vec<u8>,
}

struct Client {
    stream: TcpStream,
    area: Area,
}

impl Client {
    //  Format `self.area`
    fn format(&self) -> Vec<u8> {
        let l = (self.area.columns + 2) * 2;
        let a = (self.area.rows + 2) * l;

        let er = a - l;
        let ec = l - 1;

        let mut i = 0;

        (1..a)
            .map(|n| {
                if n < l || n > er {
                    if n % 2 == 0 {
                        EMPTY
                    } else {
                        BORDER
                    }
                } else {
                    let m = n % l;

                    if m == 0 {
                        NEWLINE
                    } else if m == 1 || m == ec {
                        BORDER
                    } else {
                        if n % 2 == 0 {
                            EMPTY
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

fn handle_client(
    client: Arc<RwLock<Client>>,
    open: Arc<RwLock<bool>>,
    mut stdout: Term,
) -> Result<()> {
    stdout.clear_screen()?;
    stdout.write_all(&client.read().unwrap().format())?;
    stdout.flush()?;

    loop {
        if let Ok(open_guard) = open.read() {
            if *open_guard {
                drop(open_guard);

                if let Ok(mut client_guard) = client.write() {
                    let area = &mut [0; 25];
                    match client_guard.stream.read_exact(area) {
                        Ok(()) => {
                            client_guard.area.data = area.to_vec();

                            stdout.clear_screen()?;
                            stdout.write_all(&client_guard.format())?;
                            stdout.flush()?;
                        }
                        Err(e) => match e.kind() {
                            WouldBlock => sleep(Duration::from_micros(1)), //  Temporary CPU Usage fix
                            _ => (),
                        },
                    }
                }
            }
        }
    }
}

fn main() -> Result<()> {
    let mut stdout = Term::buffered_stdout();

    stdout.set_title("snakers-client");
    stdout.hide_cursor()?;
    stdout.write_all(b"Connecting...")?;
    stdout.flush()?;

    let client = Arc::new(RwLock::new((|| -> Result<Client> {
        let mut stream = TcpStream::connect("127.0.0.1:6969")?;

        let rows = &mut [0; 1];
        stream.read_exact(rows)?;

        let columns = &mut [0; 1];
        stream.read_exact(columns)?;

        let data = &mut [0; 25];
        stream.read_exact(data)?;

        stream.set_nonblocking(true)?;

        Ok(Client {
            stream,
            area: Area {
                rows: rows[0] as usize,
                columns: columns[0] as usize,
                data: data.to_vec(),
            },
        })
    })()?));
    let open = Arc::new(RwLock::new(true));

    let thread_client = client.clone();
    let thread_open = open.clone();

    spawn(move || handle_client(thread_client, thread_open, stdout));

    loop {
        if let key @ (W | A | S | D | Q) = Term::stdout().read_char()? as u8 {
            if key == Q {
                break Ok(());
            }

            //  Lock
            if let Ok(mut open_guard) = open.write() {
                *open_guard = false
            }

            if let Ok(mut client_guard) = client.write() {
                client_guard.stream.write_all(&[key])?
            }

            //  Unlock
            if let Ok(mut open_guard) = open.write() {
                *open_guard = true
            }
        }
    }
}
