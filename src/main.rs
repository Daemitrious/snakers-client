mod area;
mod client;

use {
    area::Area,
    client::Client,
    console::Term,
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

        let rows = &mut [0; 1];
        stream.read_exact(rows)?;

        let columns = &mut [0; 1];
        stream.read_exact(columns)?;

        let data = &mut [0; 100];
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

    spawn(move || -> Result<()> {
        stdout.clear_screen()?;
        stdout.write_all(&thread_client.read().unwrap().format())?;
        stdout.flush()?;

        loop {
            if let Ok(open_guard) = thread_open.read() {
                if *open_guard {
                    drop(open_guard);

                    if let Ok(mut client_guard) = thread_client.write() {
                        let area = &mut [0; 100];

                        match client_guard.stream.read_exact(area) {
                            Ok(()) => {
                                client_guard.area.data = area.to_vec();

                                stdout.clear_last_lines(client_guard.area.rows + 1)?;
                                stdout.write_all(&client_guard.format())?;
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

                if let Ok(mut open_guard) = open.write() {
                    //  Lock
                    *open_guard = false;

                    if let Ok(mut client_guard) = client.write() {
                        client_guard.stream.write_all(&[key])?
                    }
                    //  Unlock
                    *open_guard = true
                }
            }
        }
    }
}
