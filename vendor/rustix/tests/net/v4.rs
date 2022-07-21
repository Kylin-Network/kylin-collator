//! Test a simple IPv4 socket server and client. The client send a
//! message and the server sends one back.

#![cfg(not(any(target_os = "redox", target_os = "wasi")))]

use rustix::net::{
    accept, bind_v4, connect_v4, getsockname, listen, recv, send, socket, AddressFamily, Ipv4Addr,
    Protocol, RecvFlags, SendFlags, SocketAddrAny, SocketAddrV4, SocketType,
};
use std::sync::{Arc, Condvar, Mutex};
use std::thread;

const BUFFER_SIZE: usize = 20;

fn server(ready: Arc<(Mutex<u16>, Condvar)>) {
    let connection_socket =
        socket(AddressFamily::INET, SocketType::STREAM, Protocol::default()).unwrap();

    let name = SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 0);
    bind_v4(&connection_socket, &name).unwrap();

    let who = match getsockname(&connection_socket).unwrap() {
        SocketAddrAny::V4(addr) => addr,
        _ => panic!(),
    };

    listen(&connection_socket, 1).unwrap();

    {
        let (lock, cvar) = &*ready;
        let mut port = lock.lock().unwrap();
        *port = who.port();
        cvar.notify_all();
    }

    let mut buffer = vec![0; BUFFER_SIZE];
    let data_socket = accept(&connection_socket).unwrap();
    let nread = recv(&data_socket, &mut buffer, RecvFlags::empty()).unwrap();
    assert_eq!(String::from_utf8_lossy(&buffer[..nread]), "hello, world");

    send(&data_socket, b"goodnight, moon", SendFlags::empty()).unwrap();
}

fn client(ready: Arc<(Mutex<u16>, Condvar)>) {
    let port = {
        let (lock, cvar) = &*ready;
        let mut port = lock.lock().unwrap();
        while *port == 0 {
            port = cvar.wait(port).unwrap();
        }
        *port
    };

    let addr = SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), port);
    let mut buffer = vec![0; BUFFER_SIZE];

    let data_socket = socket(AddressFamily::INET, SocketType::STREAM, Protocol::default()).unwrap();
    connect_v4(&data_socket, &addr).unwrap();

    send(&data_socket, b"hello, world", SendFlags::empty()).unwrap();

    let nread = recv(&data_socket, &mut buffer, RecvFlags::empty()).unwrap();
    assert_eq!(String::from_utf8_lossy(&buffer[..nread]), "goodnight, moon");
}

#[test]
fn test_v4() {
    let ready = Arc::new((Mutex::new(0_u16), Condvar::new()));
    let ready_clone = Arc::clone(&ready);

    let server = thread::Builder::new()
        .name("server".to_string())
        .spawn(move || {
            server(ready);
        })
        .unwrap();
    let client = thread::Builder::new()
        .name("client".to_string())
        .spawn(move || {
            client(ready_clone);
        })
        .unwrap();
    client.join().unwrap();
    server.join().unwrap();
}
