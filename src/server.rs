use std::io::{ErrorKind, Read, Write};
use std::net::TcpListener;
use std::sync::mpsc;
use std::thread;

const ADDRESS: &str = "127.0.0.1:6000";
const BUFFER: usize = 32;

// TODO: Implement cleanup

fn main() {
    let server = TcpListener::bind(ADDRESS).expect("Failed to bind");
    server.set_nonblocking(true).expect("Failed non-blocking");

    let mut clients = vec![];
    let (sender, receiver) = mpsc::channel::<String>();
    loop {
        if let Ok((mut socket, address)) = server.accept() {
            println!("Client {} connected", address);

            let sender = sender.clone();
            clients.push(socket.try_clone().expect("failed to clone"));

            thread::spawn(move || loop {
                let mut buff = vec![0; BUFFER];

                match socket.read_exact(&mut buff) {
                    Ok(_) => {
                        let msg = buff.into_iter().take_while(|&x| x != 0).collect::<Vec<_>>();
                        let msg = String::from_utf8(msg).expect("Invalid utf8 message");

                        println!("{}: {:?}", address, msg);
                        sender.send(msg).expect("failed to send msg to receiver");
                    }, 
                    Err(ref err) if err.kind() == ErrorKind::WouldBlock => (),
                    Err(_) => {
                        println!("closing connection with: {}", address);
                        break;
                    }
                }
                thread::sleep(::std::time::Duration::from_millis(100));
            });
        }

        if let Ok(msg) = receiver.try_recv() {
            // TODO: Filter client from which msg is received
            clients = clients.into_iter().filter_map(|mut client| {
                let mut buff = msg.clone().into_bytes();
                buff.resize(BUFFER, 0);

                client.write_all(&buff).map(|_| client).ok()
            }).collect::<Vec<_>>();
        }
        thread::sleep(::std::time::Duration::from_millis(100));
    }
}