mod area;
mod client;
mod info;
mod intention;
mod key;
mod player;

use {
    area::Area,
    client::Client,
    console::Term,
    info::Info,
    intention::{Intention, Intention::*},
    key::Key,
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

type Lock<T> = Arc<RwLock<T>>;

fn lock<T>(element: T) -> Lock<T> {
    Arc::new(RwLock::new(element))
}

fn main() -> Result<()> {
    //  Dedicated `stdout` to the later initialized child thread
    let mut stdout = Term::buffered_stdout();

    stdout.set_title("snakers-client");
    stdout.hide_cursor()?;
    stdout.write_all(b"Connecting...")?;
    stdout.flush()?;

    let client = lock((|| -> Result<Client> {
        let mut stream = TcpStream::connect(
            if let Some(tsa) = (|| -> Option<String> {
                let mut arguments = args();
                if arguments.len() == 3 {
                    let mut tsa = "".to_owned();
                    tsa.push_str(&arguments.nth(1)?);
                    tsa.push(':');
                    tsa.push_str(&arguments.next()?);
                    Some(tsa)
                } else {
                    None
                }
            })() {
                tsa
            } else {
                String::from("127.0.0.1:8080")
            },
        )?;

        let buf = &mut [0; 1];

        // Player starting position
        stream.read_exact(buf)?;
        let position = buf[0] as usize;

        // Area `rows`
        stream.read_exact(buf)?;
        let rows = buf[0] as usize;

        // Area `columns`
        stream.read_exact(buf)?;
        let columns = buf[0] as usize;

        //  Allow for more fluid concurrency
        stream.set_nonblocking(true)?;

        Ok(Client {
            stream,
            area: Area {
                rows,
                columns,
                data: Vec::new(),
            },
            player: Player { position },
        })
    })()?);

    let open = lock(false);

    let thread_client = client.clone();
    let thread_open = open.clone();

    spawn(move || -> Result<()> {
        stdout.clear_screen()?;

        //  Start screen
        stdout.write_all(
            &Info::new("Snake-rs")
                .new_category("Controls")
                .new_section()
                .add_pair("W", "Up")
                .add_pair("A", "Left")
                .add_pair("S", "Down")
                .add_pair("D", "Right")
                .finalize()
                .finalize()
                .new_category("Misc")
                .new_section()
                .add_pair("Q", "Exit")
                .finalize()
                .finalize()
                .finalize(Some("Press a Movement key to continue..."))
                .into_bytes(),
        )?;
        stdout.flush()?;
        stdout.move_cursor_to(0, 0)?;

        let time = Duration::from_micros(1);

        loop {
            if let Ok(open_guard) = thread_open.read() {
                if *open_guard {
                    drop(open_guard);

                    if let Ok(mut client_guard) = thread_client.write() {
                        let buf = &mut [0; 100];

                        if let Err(error) = client_guard.stream.read_exact(buf) {
                            drop(client_guard);
                            if let WouldBlock = error.kind() {
                                sleep(time)
                            }
                        } else {
                            client_guard.area.data = buf.to_vec();
                            stdout.clear_last_lines(client_guard.area.rows + 1)?;
                            stdout.write_all(&client_guard.area.format())?;
                            stdout.flush()?;
                        }
                    }
                } else {
                    //  Prevents the start screen from destroying the cpu
                    sleep(time)
                }
            }
        }
    });

    //  Dedicated `stdout` to main function
    let stdout = Term::buffered_stdout();

    loop {
        if let Ok(c) = stdout.read_char() {
            if let Some(key) = Key::from_char(c) {
                match Intention::from(key.clone()) {
                    Move(direction) => {
                        if let Ok(client_guard) = client.read() {
                            //  Won't write to client if player is moving into barrier
                            if let Some(new) = client_guard
                                .area
                                .can_move(direction.clone(), client_guard.player.position)
                            {
                                drop(client_guard);

                                if let Ok(mut open_guard) = open.write() {
                                    //  Lock
                                    *open_guard = false;

                                    if let Ok(mut client_guard) = client.write() {
                                        client_guard.stream.write_all(&direction.to_byte())?;

                                        client_guard.stream.set_nonblocking(false)?;

                                        let buf = &mut [0; 1];
                                        client_guard.stream.read_exact(buf)?;

                                        client_guard.stream.set_nonblocking(true)?;

                                        if buf[0] == 0 {
                                            client_guard.player.position = new
                                        }
                                    }
                                    //  Unlock
                                    *open_guard = true
                                }
                            }
                        }
                    }
                    Exit => {
                        stdout.clear_screen()?;
                        stdout.flush()?;

                        if let Ok(mut open_guard) = open.write() {
                            *open_guard = false;

                            if let Ok(mut client_guard) = client.write() {
                                client_guard.stream.set_nonblocking(false)?;
                                client_guard.stream.write_all(&key.to_byte())?;

                                let buf = &mut [0; 1];
                                client_guard.stream.read_exact(buf)?;

                                break Ok(());
                            }
                        }
                    }
                }
            }
        }
    }
}
