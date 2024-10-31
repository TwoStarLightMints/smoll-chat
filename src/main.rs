// Data to be sent with qr code
// Server's ip and port information

use smoll_chat::http::{get_mime_type, HttpRequest, HttpResponse};
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

    let room_name = "My Room";

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
                    Ok(mut content) => {
                        let mut content_split: Vec<&str> = content.split("{{}}").collect();

                        content_split.insert(1, &room_name);

                        content = content_split.join("");

                        let response = HttpResponse::builder()
                            .add_header("Content-Length", &format!("{}", content.len()))
                            .add_header("Content-Type", "text/html")
                            .body(&content);

                        inc.write(response.build().to_string().as_bytes()).unwrap();
                    }
                    Err(e) => eprintln!("Encountered error retrieving resource: {e}"),
                }
            } else if request.resource == "/chat" {
                match std::fs::read_to_string(format!("{}/resources/chat.html", static_dir)) {
                    Ok(mut content) => {
                        let mut content_split: Vec<&str> = content.split("{{}}").collect();

                        content_split.insert(1, &room_name);

                        content = content_split.join("");

                        let response = HttpResponse::builder()
                            .add_header("Content-Length", &format!("{}", content.len()))
                            .add_header("Content-Type", "txt/html")
                            .body(&content);

                        inc.write(response.build().to_string().as_bytes()).unwrap();
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
                                let json = format!(
                                    "{{\"username\": \"{}\", \"message\": \"{}\"}}",
                                    message.username, message.message
                                );

                                let response = HttpResponse::builder()
                                    .http_version("HTTP/1.1")
                                    .status_code(200)
                                    .status_message("OK")
                                    .add_header("Content-Type", "application/json")
                                    .add_header("Content-Length", &format!("{}", json.len()))
                                    .body(&json);

                                client
                                    .write(response.build().to_string().as_bytes())
                                    .unwrap();
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
                        let response = HttpResponse::builder()
                            .http_version("HTTP/1.1")
                            .status_code(200)
                            .status_message("OK")
                            .add_header("Content-Type", &get_mime_type(&request.resource))
                            .add_header("Content-Length", &format!("{}", content.len()))
                            .body(&content);

                        inc.write(response.build().to_string().as_bytes()).unwrap();
                    }
                    Err(e) => eprintln!("Encountered error retrieving resource: {e}"),
                }
            }
        } else if request.method == "POST" {
            if request.resource == "/login" {
                let response = HttpResponse::builder()
                    .http_version("HTTP/1.1")
                    .status_code(303)
                    .status_message("See Other")
                    .add_header("Content-Type", "text/html")
                    .add_header("Content-Length", "0")
                    .add_header("Location", &format!("http://{}/chat", address))
                    .add_cookie(request.body.as_ref().unwrap());

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

                inc.write(response.build().to_string().as_bytes()).unwrap();
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

                let response = HttpResponse::builder()
                    .http_version("HTTP/1.1")
                    .status_code(200)
                    .status_message("OK")
                    .add_header("Content-Length", "0");

                inc.write(response.build().to_string().as_bytes()).unwrap();
            }
        } else {
            let response = HttpResponse::builder()
                .http_version("HTTP/1.1")
                .status_code(404)
                .status_message("Not Found")
                .add_header("Content-Length", "0");

            inc.write(response.build().to_string().as_bytes()).unwrap();
        }
    }
}
