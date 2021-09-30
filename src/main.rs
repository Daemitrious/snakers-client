use std::{
    io::{stdin, Error, Read, Write},
    net::TcpStream,
};

fn main() -> Result<(), Error> {
    let mut stream = TcpStream::connect("127.0.0.1:6969")?;

    loop {
        println!("Waiting for input...");
        let buf = &mut [0; 1];
        stdin().read_exact(buf)?;

        stream.write_all(buf)?;

        println!("Reading...");
        let buf = &mut [0; 1];
        stream.read_exact(buf)?;

        println!("{}", buf[0])
    }
}
