use std::io::{ErrorKind, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::os::fd::AsRawFd;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();
    listener.set_nonblocking(true).unwrap();
    let mut clients: Vec<TcpStream> = Vec::new();

    loop {
        let mut fds = vec![libc::pollfd {
            fd: listener.as_raw_fd(),
            events: libc::POLLIN,
            revents: 0,
        }];
        for client in &clients {
            fds.push(libc::pollfd {
                fd: client.as_raw_fd(),
                events: libc::POLLIN,
                revents: 0,
            });
        }

        unsafe { libc::poll(fds.as_mut_ptr(), fds.len() as libc::nfds_t, -1); }

        // Clients accepted below were not in this poll, so fds has no slot for them yet.
        let polled = clients.len();

        if (fds[0].revents & libc::POLLIN) != 0 {
            loop {
                match listener.accept() {
                    Ok((stream, _)) => {
                        stream.set_nonblocking(true).unwrap();
                        clients.push(stream);
                    }
                    Err(e) if e.kind() == ErrorKind::WouldBlock => break,
                    Err(e) => {
                        println!("error: {e}");
                        break;
                    }
                }
            }
        }

        let mut alive = Vec::new();
        for (i, mut client) in clients.into_iter().enumerate() {
            if i >= polled {
                alive.push(client);
                continue;
            }
            let ready = (fds[i + 1].revents & (libc::POLLIN | libc::POLLHUP)) != 0;
            if !ready {
                alive.push(client);
                continue;
            }
            let mut buffer = [0u8; 1024];
            match client.read(&mut buffer) {
                Ok(0) => {} // drop it: peer closed
                Ok(_) => {
                    client.write_all(b"+PONG\r\n").unwrap();
                    alive.push(client);
                }
                Err(e) if e.kind() == ErrorKind::WouldBlock => alive.push(client),
                Err(e) => println!("error: {e}"),
            }
        }
        clients = alive;
    }
}