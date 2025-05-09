use std::net::TcpStream;
use std::sync::mpsc::{self, Sender, Receiver}
use std::thread::{self, JoinHandle};
use std::time::Duration;

struct WebSocketPool {
    socket_streams: Vec<WebSocket>,
}

impl WebSocketPool {
    fn new() -> Self {
        Self {
            socket_streams: Vec::new(),
        }
    }

    fn with_capacity(num_streams: usize) -> Self {
        let avail_streams = Vec::with_capacity(num_streams);

        for _ in 0..num_streams {
            let thread = thread::spawn(|| {

            })
        }

        Self {
            socket_streams: avail_streams,
        }
    }

    fn client_join(&mut self, client: TcpStream) {}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WebSocketState {
    Ready,
    InUse,
}

struct WebSocket {
    state: WebSocketState,
    sender: Sender<TcpStream>,
    receiver: Receiver<WebSocketState>,
    thread: JoinHandle<()>,
}

impl WebSocket {
    fn new() -> Self {
        let (websocket_sender, thread_receiver) = mpsc::channel::<TcpStream>();
        let (thread_sender, websocket_receiver) = mpsc::channel::<WebSocketState>();

        let thread = thread::spawn(move || {
            let recv = thread_receiver;
            let send = thread_sender;

            loop {
                match recv.try_recv() {
                    Ok(mut stream) => {
                        websocket_initial_handshake(&mut stream);

                        websocket_handle_connection(stream);
                    }
                    Err(_) => (),
                }

                thread::sleep(Duration::from_secs(1));
            }
        });

        Self {
            state: WebSocketState::Ready,
            sender: websocket_sender,
            receiver: websocket_receiver,
            thread,
        }
    }
}

fn websocket_initial_handshake(stream: &mut TcpStream) {}

fn websocket_handle_connection(stream: TcpStream) {}
