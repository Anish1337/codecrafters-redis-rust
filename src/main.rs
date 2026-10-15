// WouldBlock(),.read(),.write_all()
use std::io::{ErrorKind, Read, Write};
// socket to accept/establish TCP conn
use std::net::{TcpListener, TcpStream};
// Access to a unix file descriptor
use std::os::fd::AsRawFd;

fn main() {
    // extacts succesful value or panics if there is an error
    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();
    // returns WouldBlock error if no pending connection
    listener.set_nonblocking(true).unwrap();

    // empty vector of client connections
    let mut clients: Vec<TcpStream> = Vec::new();
    // event loop
    loop {
        //
        let mut fds = vec![libc::pollfd {
            fd: listener.as_raw_fd(), // file descriptor to monitor
            events: libc::POLLIN,     // event to check
            revents: 0,               // event to report
        }];

        // add every client to the poll list
        for client in &clients {
            fds.push(libc::pollfd {
                fd: client.as_raw_fd(),
                events: libc::POLLIN,
                revents: 0,
            });
        }

        unsafe {
            // mutable raw ptr to array, wait indefinitely until event occurs or interrupt
            libc::poll(fds.as_mut_ptr(), fds.len() as libc::nfds_t, -1);
        }

        // Clients accepted below were not in this poll, so fds has no slot for them yet.
        let polled = clients.len();
        // nonblocking networking, remainder clients
        if (fds[0].revents & libc::POLLIN) != 0 {
            loop {
                match listener.accept() {
                    // succesful connection
                    Ok((stream, _)) => {
                        stream.set_nonblocking(true).unwrap();
                        clients.push(stream);
                    }
                    // stop accepting if no conn rdy
                    Err(e) if e.kind() == ErrorKind::WouldBlock => break,
                    // other error
                    Err(e) => {
                        println!("error: {e}");
                        break;
                    }
                }
            }
        }

        let mut alive = Vec::new();
        // move each client out of the vector
        for (i, mut client) in clients.into_iter().enumerate() {
            if i >= polled {
                alive.push(client);
                continue;
            }
            // fds[0] is the listener so + 1
            let ready = (fds[i + 1].revents & (libc::POLLIN | libc::POLLHUP)) != 0;
            if !ready {
                alive.push(client);
                continue;
            }
            // createa a buffer with 1024 zero-init bytes
            let mut buffer = [0u8; 1024];
            match client.read(&mut buffer) {
                Ok(0) => {} // drop it: peer closed
                // read buffer
                Ok(_) => {
                    // byte string literal
                    client.write_all(b"+PONG\r\n").unwrap();
                    alive.push(client);
                }
                // no data
                Err(e) if e.kind() == ErrorKind::WouldBlock => alive.push(client),
                // some other io error
                Err(e) => println!("error: {e}"),
            }
        }
        clients = alive;
    }
}
