use std::io::prelude::*;
use std::net::TcpStream;

const ADDRESS: &str = "127.0.0.1:38451";

fn main() {
    let mut send_stream = TcpStream::connect(ADDRESS).unwrap();
    let mut recv_stream = TcpStream::connect(ADDRESS).unwrap();
    let mut bin = String::new();
    let args: Vec<u8> = std::io::stdin()
        .lock()
        .split(b'\n')
        .filter_map(Result::ok)
        .map(|mut s| {
            s.push(b'\n');
            s
        })
        .flatten()
        .collect();

    send_stream.write_all(&args).unwrap();
    send_stream.flush().unwrap();
    drop(send_stream);
    recv_stream.read_to_string(&mut bin).unwrap();
    if !bin.is_empty() {
        println!("{}", &bin);
    }
}
