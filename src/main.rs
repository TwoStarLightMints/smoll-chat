// Data to be sent with qr code
// Server's ip and port information

use smoll_chat::http::{HttpRequest, HttpResponse};
use std::env;
use std::fs::File;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::PathBuf;
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

struct SmollChatOpts {
    pub port: u32,
    pub qrcode: bool,
    pub static_dir: PathBuf,
    pub room_name: String,
}

impl SmollChatOpts {
    pub fn default() -> Self {
        Self {
            port: 8080,
            qrcode: false,
            static_dir: env::current_dir().unwrap(),
            room_name: String::from("Room"),
        }
    }

    pub fn parse() -> Self {
        let env_file = File::open(".env");

        match env_file {
            Ok(file) => Self::parse_env(file),
            Err(_) => Self::parse_args(env::args()),
        }
    }

    pub fn parse_args(mut args: env::Args) -> Self {
        let mut opts_parsed = Self::default();

        while let Some(opt) = args.next() {
            match opt.as_str() {
                "--port" | "-p" => {
                    opts_parsed.port = args
                        .next()
                        .expect("Not enough arguments passed")
                        .parse::<u32>()
                        .expect("Invalid port number passed");
                }
                "--qrcode" => {
                    opts_parsed.qrcode = args
                        .next()
                        .expect("Not enough arguments passed")
                        .parse::<bool>()
                        .expect("Invalid boolean passed")
                }
                "--static-dir" => {
                    opts_parsed.static_dir =
                        PathBuf::from(args.next().expect("Not enough arguments passed"))
                }
                "--room-name" => {
                    opts_parsed.room_name = args.next().expect("Not enough arguments passed")
                }
                _ => (),
            }
        }

        opts_parsed
    }

    pub fn parse_env(mut env_file: File) -> Self {
        let mut opts_parsed = Self::default();

        let mut buf = String::new();

        env_file
            .read_to_string(&mut buf)
            .expect("Error reading opts file");

        buf.lines().for_each(|l| {
            let mut line = l.split("=");

            match line.next().expect("Error in env file formatting") {
                "port" => {
                    opts_parsed.port = line
                        .next()
                        .expect("No value provided in env file")
                        .parse::<u32>()
                        .expect("Invalid port number passed")
                }
                "qrcode" => {
                    opts_parsed.qrcode = line
                        .next()
                        .expect("No value provided in env file")
                        .parse::<bool>()
                        .expect("Invalid boolean passed")
                }
                "static-dir" => {
                    opts_parsed.static_dir =
                        PathBuf::from(line.next().expect("No value provided in env file"))
                }
                "room-name" => {
                    opts_parsed.room_name = line
                        .next()
                        .expect("No value provided in env file")
                        .to_string()
                }
                _ => (),
            }
        });

        opts_parsed
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
    let options = SmollChatOpts::parse();

    let address = format!("{}:{}", local_ip().unwrap().to_string(), options.port);

    let listener = TcpListener::bind(&address).expect("Failed to initialize server");

    println!("Server now running at http://{}", address);

    if options.qrcode {
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
                match std::fs::read_to_string(format!(
                    "{}/index.html",
                    options.static_dir.display()
                )) {
                    Ok(mut content) => {
                        let mut content_split: Vec<&str> = content.split("{{}}").collect();

                        content_split.insert(1, &options.room_name);

                        content = content_split.join("");

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
                match std::fs::read_to_string(format!("{}/chat.html", options.static_dir.display()))
                {
                    Ok(mut content) => {
                        let mut content_split: Vec<&str> = content.split("{{}}").collect();

                        content_split.insert(1, &options.room_name);

                        content = content_split.join("");

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
                                    "{{\"username\": \"{}\", \"message\": \"{}\"}}",
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
                    "{}/{}",
                    options.static_dir.display(),
                    request.resource.splitn(3, "/").skip(2).next().unwrap()
                )) {
                    Ok(content) => {
                        let mut response =
                            HttpResponse::new("HTTP/1.1".to_string(), 200, "OK".to_string());

                        if request.resource.ends_with(".css") {
                            response.set_content_type("text/css");
                        } else if request.resource.ends_with(".js") {
                            response.set_content_type("text/javascript");
                        }
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
