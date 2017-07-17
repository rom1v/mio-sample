extern crate mio;
use std::io::{ErrorKind, Read, Write};
use std::net::{self, Ipv4Addr, SocketAddr};
use std::time;
use std::thread;
use mio::{Events, Ready, Poll, PollOpt, Token};

const PORT: u16 = 1234;

fn client() {
    let addr = SocketAddr::new(Ipv4Addr::new(127, 0, 0, 1).into(), PORT);
    let mut stream = mio::tcp::TcpStream::connect(&addr).unwrap();

    let poll = Poll::new().unwrap();
    let mut events = Events::with_capacity(128);

    poll.register(&stream, Token(0), Ready::readable(), PollOpt::level())
        .unwrap();

    let mut buf = [0; 0x10000];
    loop {
        poll.poll(&mut events, None).unwrap();

        for event in &events {
            println!("CLIENT (receiver): event={:?}", event);
            if event.readiness().is_readable() {
                match stream.read(&mut buf) {
                    Ok(0) => {
                        println!("CLIENT (receiver): eof");
                        return;
                    }
                    Ok(len) => {
                        let content = String::from_utf8_lossy(&buf[..len]);
                        println!("CLIENT (receiver): read {} bytes: [{}]", len, content);
                    }
                    Err(err) => {
                        if cfg!(feature = "workaround") && cfg!(windows) &&
                            err.kind() == ErrorKind::WouldBlock
                        {
                            println!("CLIENT (receiver): spurious event, ignoring");
                        } else {
                            println!("CLIENT (receiver): error [{:?}]: {}", err.kind(), err);
                            return;
                        }
                    }
                }
            }
        }
    }
}

fn server() {
    let addr = SocketAddr::new(Ipv4Addr::new(127, 0, 0, 1).into(), PORT);
    let server = net::TcpListener::bind(&addr).unwrap();
    println!("SERVER (sender): listening on 127.0.0.1:{}", PORT);
    let (mut stream, _) = server.accept().unwrap();
    println!("SERVER (sender): writing 'Hello'");
    stream.write("Hello".as_bytes()).unwrap();
    println!("SERVER (sender): writing ', '");
    stream.write(", ".as_bytes()).unwrap();
    thread::sleep(time::Duration::from_secs(1));
    println!("SERVER (sender): writing 'world!'");
    stream.write("world!".as_bytes()).unwrap();
    println!("SERVER (sender): closing");
}

fn main() {
    thread::spawn(server);
    thread::sleep(time::Duration::from_secs(1));
    client();
}
