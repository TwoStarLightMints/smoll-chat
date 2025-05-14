use std::net::TcpListener;

use smoll_chat::websocket::WebSocketPool;

use local_ip_address::local_ip;

fn main() {
    let mut websocket_pool = WebSocketPool::with_capacity(10);

    let listener = TcpListener::bind(format!("{}:8081", local_ip().unwrap())).unwrap();
    listener.set_nonblocking(true).unwrap();

    println!(
        "{}",
        format!(
            "WebSocket server listening at: {}:8081",
            local_ip().unwrap()
        )
    );

    for inc in listener.incoming() {
        match inc {
            Ok(stream) => websocket_pool.client_join(stream),
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => (),
            Err(_) => panic!("Oh my!"),
        }

        websocket_pool.run();

        std::thread::sleep(std::time::Duration::from_millis(1500));
    }
}
