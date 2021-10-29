mod area;
mod client;
mod direction;
mod player;

use {
    area::Area,
    client::Client,
    console::Term,
    direction::Direction,
    player::Player,
    std::{
        env::args,
        io::{ErrorKind::WouldBlock, Read, Result, Write},
        net::TcpStream,
        sync::{Arc, RwLock},
        thread::{sleep, spawn},
        time::Duration,
    },
};

fn main() -> Result<()> {
    let mut stdout = Term::buffered_stdout();

    stdout.set_title("snakers-client");
    stdout.hide_cursor()?;
    stdout.write_all(b"Connecting...")?;
    stdout.flush()?;

    let client = Arc::new(RwLock::new((|| -> Result<Client> {
        let mut stream = TcpStream::connect(
            if let Some(tsa) = (|| -> Option<String> {
                let mut a = args();
                if a.len() == 3 {
                    let mut tsa = "".to_owned();
                    tsa.push_str(&a.nth(1)?);
                    tsa.push(':');
                    tsa.push_str(&a.next()?);
                    Some(tsa)
                } else {
                    None
                }
            })() {
                tsa
            } else {
                String::from("127.0.0.1:6969")
            },
        )?;

        let mut buf = [0; 1];

        // Player starting position
        stream.read_exact(&mut buf)?;
        let position = buf[0] as usize;

        // Area `rows`
        stream.read_exact(&mut buf)?;
        let rows = buf[0] as usize;

        // Area `columns`
        stream.read_exact(&mut buf)?;
        let columns = buf[0] as usize;

        let mut buf = [0; 100];

        //  Area `data`
        stream.read_exact(&mut buf)?;
        let data = buf.to_vec();

        //  Allow for more fluid concurrency
        stream.set_nonblocking(true)?;

        Ok(Client {
            stream,
            area: Area {
                rows,
                columns,
                data,
            },
            player: Player { position },
        })
    })()?));

    let open = Arc::new(RwLock::new(true));

    let thread_client = client.clone();
    let thread_open = open.clone();

    spawn(move || -> Result<()> {
        stdout.clear_screen()?;
        stdout.write_all(&thread_client.read().unwrap().area.format())?;
        stdout.flush()?;

        loop {
            if let Ok(open_guard) = thread_open.read() {
                if *open_guard {
                    drop(open_guard);

                    if let Ok(mut client_guard) = thread_client.write() {
                        let buf = &mut [0; 100];

                        match client_guard.stream.read_exact(buf) {
                            Ok(()) => {
                                client_guard.area.data = buf.to_vec();

                                stdout.clear_last_lines(client_guard.area.rows + 1)?;
                                stdout.write_all(&client_guard.area.format())?;
                                stdout.flush()?;
                            }
                            Err(e) => {
                                drop(client_guard);
                                if let WouldBlock = e.kind() {
                                    sleep(Duration::from_micros(1))
                                }
                            }
                        }
                    }
                }
            }
        }
    });

    loop {
        if let Ok(c) = Term::buffered_stdout().read_char() {
            if let key @ (119 | 97 | 115 | 100 | 113) = c as u8 {
                if key == 113 {
                    break Ok(());
                }

                if let Ok(client_guard) = client.read() {
                    if let Some(direction) = Direction::from_key(key) {
                        // This check is possible because players can't be moved by anything other than themselves
                        if let Some(np) = client_guard
                            .area
                            .can_move(direction, client_guard.player.position)
                        {
                            drop(client_guard);

                            if let Ok(mut open_guard) = open.write() {
                                //  Lock
                                *open_guard = false;

                                if let Ok(mut client_guard) = client.write() {
                                    client_guard.stream.write_all(&[key])?;

                                    // This might cause the client-side to break if the connection to the host is inconsistent or extremely slow
                                    client_guard.player.position = np
                                }
                                //  Unlock
                                *open_guard = true
                            }
                        }
                    }
                }
            }
        }
    }
}
