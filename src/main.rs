#![allow(unused_imports)]
use std::net::TcpListener;
// read/write TCP stream
use std::io::{Read, Write;};

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();
    for stream in listener.incoming() {
        match stream {
            // mut to change stream
            Ok(mut stream) => {
                let mut buffer = [0u8; 1024];
                loop {
                    let n = stream.read(&mut buffer).unwrap();
                    // no data coming in
                    if n == 0 {
                        break; // end of stream
                    }
                    // write response
                    stream.write_all(b"+PONG\r\n").unwrap();
                }
                println!("Received data: {}", String::from_utf8_lossy(&buffer[..]));
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
