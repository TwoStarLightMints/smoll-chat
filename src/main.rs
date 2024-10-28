// Data to be sent with qr code
// Server's ip and port information

use smoll_chat::http::{HttpRequest, HttpResponse};
use std::env;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::mpsc::{self, Sender};
use std::thread;
use std::time::Duration;

use local_ip_address::local_ip;
use qrcode::QrCode;

struct UserMessage {
    pub username: String,
    pub message: String,
}

impl UserMessage {
    pub fn new(username: String, message: String) -> Self {
        Self { username, message }
    }
}

fn render_server_qr_code(address: &str) {
    let code = QrCode::new(format!("http://{}", address)).unwrap();

    let qr_term = code
        .render()
        .light_color("\u{001b}[1;34;37;47m  \u{001b}[0m")
        .dark_color("\u{001b}[1;34;37;40m  \u{001b}[0m")
        .build();

    println!("{qr_term}");
}

fn main() {
    let args: Vec<_> = env::args().collect();

    let static_dir = env::current_dir().unwrap().to_str().unwrap().to_string();

    if args.len() < 3 || !(&args[1] == "--port" || &args[1] == "-p") {
        eprintln!("No port provided, please provide a port with -p or --port");
        return;
    }

    let port = &args[2];
    let addr = local_ip().unwrap().to_string();

    let address = format!("{}:{}", addr, port);

    let listener = TcpListener::bind(&address).expect("Failed to initialize server");

    println!("Server now running at http://{}", address);

    if args.len() > 3 && args[3] == "--qrcode" {
        render_server_qr_code(&address);
    }

    let mut message_queue: Vec<Sender<UserMessage>> = Vec::new();

    for stream in listener.incoming() {
        let mut inc = stream.unwrap();

        let mut request = [0; 1024];

        inc.read(&mut request).unwrap();

        let raw_request = String::from_utf8(request.to_vec()).unwrap();

        let request = HttpRequest::parse(&raw_request);

        if request.method == "GET" {
            if request.resource == "/" {
                match std::fs::read_to_string(format!("{}/resources/index.html", static_dir)) {
                    Ok(content) => {
                        let mut response =
                            HttpResponse::new("HTTP/1.1".to_string(), 200, "OK".to_string());

                        response.set_content_len(content.len());
                        response.set_content_type("text/html");

                        response.add_body(content);

                        inc.write(response.to_string().as_bytes()).unwrap();
                    }
                    Err(e) => eprintln!("Encountered error retrieving resource: {e}"),
                }
            } else if request.resource == "/chat" {
                match std::fs::read_to_string(format!("{}/resources/chat.html", static_dir)) {
                    Ok(content) => {
                        let mut response =
                            HttpResponse::new("HTTP/1.1".to_string(), 200, "OK".to_string());

                        response.set_content_len(content.len());
                        response.set_content_type("text/html");

                        response.add_body(content);

                        inc.write(response.to_string().as_bytes()).unwrap();
                    }
                    Err(e) => eprintln!("Encountered error retrieving resource: {e}"),
                }
            } else if request.resource == "/new-message" {
                let (s, r) = mpsc::channel();

                message_queue.push(s);

                thread::spawn(move || {
                    let receiver: mpsc::Receiver<UserMessage> = r;
                    let mut client = inc;

                    loop {
                        match receiver.try_recv() {
                            Ok(message) => {
                                let mut response = HttpResponse::new(
                                    "HTTP/1.1".to_string(),
                                    200,
                                    "OK".to_string(),
                                );

                                let json = format!(
                                    "{{\"sender\": \"{}\", \"message\": \"{}\"}}",
                                    message.username, message.message
                                );

                                response.add_header(
                                    "Content-Type".to_string(),
                                    "application/json".to_string(),
                                );

                                response.set_content_len(json.len());

                                response.add_body(json);

                                client.write(response.to_string().as_bytes()).unwrap();
                                client.flush().unwrap();

                                break;
                            }
                            Err(_) => (),
                        }

                        thread::sleep(Duration::from_secs(1));
                    }
                });
            } else if request.resource.starts_with("/static/") {
                match std::fs::read_to_string(format!(
                    "{}/resources/{}",
                    static_dir,
                    request.resource.splitn(3, "/").skip(2).next().unwrap()
                )) {
                    Ok(content) => {
                        let mut response =
                            HttpResponse::new("HTTP/1.1".to_string(), 200, "OK".to_string());

                        response.set_content_type("text/css");
                        response.set_content_len(content.len());

                        response.add_body(content);

                        inc.write(response.to_string().as_bytes()).unwrap();
                    }
                    Err(e) => eprintln!("Encountered error retrieving resource: {e}"),
                }
            }
        } else if request.method == "POST" {
            if request.resource == "/login" {
                let mut response =
                    HttpResponse::new("HTTP/1.1".to_string(), 303, "See other".to_string());

                println!("{}", request.body.as_ref().unwrap());

                println!(
                    "User {} has joined the chat.",
                    request
                        .body
                        .as_ref()
                        .unwrap()
                        .split("=")
                        .skip(1)
                        .next()
                        .unwrap()
                );

                response.add_header(
                    "Content-Type".to_string(),
                    "text/html; charset=utf-8".to_string(),
                );
                response.add_header("Location".to_string(), format!("http://{}/chat", address));
                response.set_cookie(request.body.clone().unwrap());
                response.set_content_len(0);

                inc.write(response.to_string().as_bytes()).unwrap();
            } else if request.resource == "/message" {
                while let Some(sender) = message_queue.pop() {
                    sender
                        .send(UserMessage::new(
                            request
                                .get_header("Cookie")
                                .unwrap()
                                .split("=")
                                .skip(1)
                                .next()
                                .unwrap()
                                .to_string(),
                            request.body.clone().unwrap(),
                        ))
                        .unwrap();
                }

                let mut response =
                    HttpResponse::new("HTTP/1.1".to_string(), 204, "No Content".to_string());

                response.set_content_len(0);

                inc.write(response.to_string().as_bytes()).unwrap();
            }
        } else {
            let mut response =
                HttpResponse::new("HTTP/1.1".to_string(), 404, "Not Found".to_string());

            response.set_content_len(0);

            inc.write(response.to_string().as_bytes()).unwrap();
        }
    }
}
