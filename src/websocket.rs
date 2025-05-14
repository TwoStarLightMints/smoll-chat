use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use crate::http::{HttpRequest, HttpResponse};

pub struct WebSocketPool {
    socket_streams: Vec<WebSocket>,
}

impl WebSocketPool {
    pub fn new() -> Self {
        Self {
            socket_streams: Vec::new(),
        }
    }

    pub fn with_capacity(num_clients: usize) -> Self {
        let mut avail_streams = Vec::with_capacity(num_clients);

        for id in 0..num_clients {
            avail_streams.push(WebSocket::new(id));
        }

        Self {
            socket_streams: avail_streams,
        }
    }

    pub fn client_join(&mut self, client: TcpStream) {
        for ele in self.socket_streams.iter() {
            println!("{}", ele.is_ready());
        }
        match self
            .socket_streams
            .iter_mut()
            .filter(|w| w.is_ready())
            .next()
        {
            Some(socket) => socket.handle_client(client),
            None => eprintln!("No streams available"),
        }
    }

    pub fn run(&mut self) {
        self.socket_streams.iter().for_each(|s| match s.poll() {
            Ok(message) => {
                println!("{message}");

                self.socket_streams
                    .iter()
                    .filter(|socket| s.id != socket.id)
                    .for_each(|receiver| receiver.send_message(message.clone()));
            }
            Err(_) => (),
        });
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WebSocketState {
    Ready,
    InUse,
}

struct WebSocket {
    id: usize,
    state: Arc<Mutex<WebSocketState>>,
    sender: Sender<TcpStream>,
    message_sender: Sender<String>,
    receiver: Receiver<String>,
}

impl WebSocket {
    fn new(id: usize) -> Self {
        let (websocket_sender, thread_receiver) = mpsc::channel::<TcpStream>();
        let (thread_sender, websocket_receiver) = mpsc::channel::<String>();
        let (websocket_message_sender, thread_message_receiver) = mpsc::channel::<String>();
        let state = Arc::new(Mutex::new(WebSocketState::Ready));
        let sock_state = Arc::clone(&state);

        thread::spawn(move || {
            let recv = thread_receiver;
            let mut send = thread_sender;
            let message_recv = thread_message_receiver;

            loop {
                match recv.try_recv() {
                    Ok(mut stream) => {
                        websocket_initial_handshake(&mut stream);

                        println!("Performed handshake");

                        websocket_handle_connection(stream, &mut send, &message_recv);

                        let mut sock_state = sock_state.lock().unwrap();

                        *sock_state = WebSocketState::Ready;
                    }
                    Err(_) => (),
                }

                thread::sleep(Duration::from_millis(500));
            }
        });

        Self {
            id,
            state,
            sender: websocket_sender,
            message_sender: websocket_message_sender,
            receiver: websocket_receiver,
        }
    }

    fn is_ready(&self) -> bool {
        *self.state.lock().unwrap() == WebSocketState::Ready
    }

    fn handle_client(&mut self, stream: TcpStream) {
        println!("Client info sent");

        self.sender.send(stream).expect("Error with client joining");

        *self.state.lock().unwrap() = WebSocketState::InUse;

        println!("Client info received");
    }

    fn poll(&self) -> Result<String, std::sync::mpsc::TryRecvError> {
        self.receiver.try_recv()
    }

    fn send_message(&self, message: String) {
        self.message_sender.send(message).unwrap();
    }
}

fn calculate_accept_key(mut client_key: String) -> String {
    client_key = client_key.trim().to_string();
    client_key.push_str("258EAFA5-E914-47DA-95CA-C5AB0DC85B11");

    let hash = openssl::sha::sha1(client_key.as_bytes());

    openssl::base64::encode_block(&hash)
}

fn websocket_initial_handshake(stream: &mut TcpStream) {
    let mut buf = [0; 2048];

    stream.read(&mut buf).unwrap();

    let request = HttpRequest::parse(&String::from_utf8(buf.to_vec()).unwrap());

    let accept_key = calculate_accept_key(request.get_header("Sec-WebSocket-Key").unwrap().clone());

    let response = HttpResponse::builder()
        .http_version("HTTP/1.1")
        .status_code(101)
        .status_message("Switching Protocols")
        .add_header("Upgrade", "websocket")
        .add_header("Connection", "Upgrade")
        .add_header("Sec-WebSocket-Accept", &accept_key)
        .build();

    stream.write(response.to_string().as_bytes()).unwrap();
    stream.flush().unwrap();
}

fn get_message_length(message: &[u8]) -> (usize, usize) {
    //! Returns: (length, read_offset)

    let length_indicator = message[1] & 0b01111111;

    if length_indicator <= 125 {
        return (length_indicator as usize, 2);
    } else if length_indicator == 126 {
        return (
            (((message[2] as usize) << 8) | message[3] as usize) as usize,
            4,
        );
    } else {
        let mut length = 0;

        for i in 2..10 {
            length = ((length << 8) as usize) | message[i] as usize;
        }

        return (length, 9);
    }
}

fn parse_message(message: &[u8]) -> String {
    let (message_length, mut offset) = get_message_length(message);

    let masking_key = &message[offset..(offset + 4)];

    offset += 4;

    let payload = &message[offset..(offset + message_length)];

    let mut decoded = Vec::new();

    for (index, byte) in payload.iter().enumerate() {
        decoded.push(byte ^ masking_key[index % 4]);
    }

    let mut parsed = String::new();

    match String::from_utf8(decoded.clone()) {
        Ok(s) => {
            parsed = s;
        }
        Err(_) => {
            message[..7].iter().for_each(|b| println!("{b:x}"));
        }
    }

    parsed
}

fn websocket_handle_connection(
    mut stream: TcpStream,
    send: &mut Sender<String>,
    receive: &Receiver<String>,
) {
    stream
        .set_nonblocking(true)
        .expect("Error setting nonblocking");

    loop {
        let mut buf = [0; 2048];

        match stream.read(&mut buf) {
            Ok(_) => {
                if buf[0] == 0x88 {
                    // Encountered close frame
                    stream.shutdown(std::net::Shutdown::Both).unwrap();

                    println!("A client has disconnected");

                    break;
                } else {
                    send.send(parse_message(&buf)).unwrap();
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                std::thread::sleep(std::time::Duration::from_millis(500))
            }
            Err(e) => eprintln!("Error encountered: {e}"),
        }

        match receive.try_recv() {
            Ok(message) => {
                let byte1: u8 = 0b10000001;
                let mut byte2: u8 = 0b00000000;

                let message_len = message.len();

                if message_len <= 125 {
                    byte2 |= message_len as u8;
                } else if message_len > u16::MAX as usize {
                    byte2 |= 126;
                } else {
                    byte2 |= 127;
                }

                let mut encoded = Vec::new();

                encoded.push(byte1);
                encoded.push(byte2);
                encoded.extend_from_slice(message.as_bytes());

                stream.write(&encoded).unwrap();
            }
            Err(e) if e == std::sync::mpsc::TryRecvError::Empty => (),
            Err(_) => panic!("Panic!"),
        }
    }
}
