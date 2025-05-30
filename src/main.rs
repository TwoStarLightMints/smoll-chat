use smoll_chat::{
    http::{get_mime_type, HttpRequest, HttpResponse},
    option_parse::SmollChatOpts,
    websocket::{websocket_initial_handshake, WebSocketPool},
};

use std::net::TcpListener;
use std::time::Duration;
use std::{io::Write, net::TcpStream};

use local_ip_address::local_ip;
use qrcode::QrCode;

fn render_server_qr_code(address: &str) {
    let code = QrCode::new(format!("http://{}", address)).unwrap();

    let qr_term = code
        .render()
        .light_color("\u{001b}[1;34;37;47m  \u{001b}[0m")
        .dark_color("\u{001b}[1;34;37;40m  \u{001b}[0m")
        .build();

    println!("{qr_term}");
}

fn handle_client(
    mut client: TcpStream,
    socket_pool: &mut WebSocketPool,
    options: &SmollChatOpts,
    address: &str,
) {
    let request = HttpRequest::from(&mut client);

    println!("[{}] {}", request.method, request.resource);

    if request.method == "GET" {
        match request.resource.as_str() {
            "/" => {
                match std::fs::read_to_string(format!(
                    "{}/index.html",
                    options.static_dir.display()
                )) {
                    Ok(mut content) => {
                        let mut content_split: Vec<&str> = content.split("{{}}").collect();

                        content_split.insert(1, &options.room_name);

                        content = content_split.join("");

                        let response = HttpResponse::builder()
                            .add_header("Content-Length", &format!("{}", content.len()))
                            .add_header("Content-Type", "text/html")
                            .body(&content);

                        client
                            .write(response.build().to_string().as_bytes())
                            .unwrap();
                    }
                    Err(e) => {
                        eprintln!("Error retrieving resource: {} == {}", request.resource, e)
                    }
                }
            }
            "/chat" => {
                match std::fs::read_to_string(format!("{}/chat.html", options.static_dir.display()))
                {
                    Ok(mut content) => {
                        let mut content_split: Vec<&str> = content.split("{{}}").collect();

                        content_split.insert(1, &options.room_name);

                        content = content_split.join("");

                        let response = HttpResponse::builder()
                            .add_header("Content-Length", &format!("{}", content.len()))
                            .add_header("Content-Type", "text/html")
                            .body(&content);

                        client
                            .write(response.build().to_string().as_bytes())
                            .unwrap();
                    }
                    Err(e) => {
                        eprintln!("Error retrieving resource: {} == {}", request.resource, e)
                    }
                }
            }
            "/socket" => {
                websocket_initial_handshake(&mut client, request);

                socket_pool.client_join(client);
            }
            resource if resource.starts_with("/static/") => {
                match std::fs::read_to_string(format!(
                    "{}/{}",
                    options.static_dir.display(),
                    request.resource.splitn(3, "/").skip(2).next().unwrap()
                )) {
                    Ok(mut content) => {
                        if content.find("{{}}").is_some() {
                            let mut content_split: Vec<&str> = content.split("{{}}").collect();
                            let addr = format!("{}:8080", local_ip().unwrap().to_string());

                            content_split.insert(1, &addr);

                            content = content_split.join("");
                        }

                        let response = HttpResponse::builder()
                            .http_version("HTTP/1.1")
                            .status_code(200)
                            .status_message("OK")
                            .add_header("Content-Type", &get_mime_type(&request.resource))
                            .add_header("Content-Length", &format!("{}", content.len()))
                            .body(&content);

                        client
                            .write(response.build().to_string().as_bytes())
                            .unwrap();
                    }
                    Err(e) => eprintln!("Error retrieving resource: {} == {}", request.resource, e),
                }
            }
            _ => {
                let response = HttpResponse::builder()
                    .http_version("HTTP/1.1")
                    .status_code(404)
                    .status_message("Not found");

                client
                    .write(response.build().to_string().as_bytes())
                    .unwrap();
            }
        }
    } else if request.method == "POST" {
        match request.resource.as_str() {
            "/login" => {
                let response = HttpResponse::builder()
                    .http_version("HTTP/1.1")
                    .status_code(303)
                    .status_message("See Other")
                    .add_header("Content-Type", "text/html")
                    .add_header("Content-Length", "0")
                    .add_header("Location", &format!("http://{}/chat", address))
                    .add_cookie(request.body.as_ref().unwrap());

                println!(
                    "[ User join ] {}",
                    request
                        .body
                        .as_ref()
                        .unwrap()
                        .split("=")
                        .skip(1)
                        .next()
                        .unwrap()
                );

                client
                    .write(response.build().to_string().as_bytes())
                    .unwrap();
            }
            _ => {
                let response = HttpResponse::builder()
                    .http_version("HTTP/1.1")
                    .status_code(404)
                    .status_message("Not Found")
                    .add_header("Content-Length", "0");

                client
                    .write(response.build().to_string().as_bytes())
                    .unwrap();
            }
        }
    }
}

fn main() {
    let options = SmollChatOpts::parse().unwrap();

    println!("{}", options.to_string());

    let address = format!("{}:{}", local_ip().unwrap().to_string(), options.port);

    let listener = TcpListener::bind(&address).expect("Failed to initialize server");
    listener
        .set_nonblocking(true)
        .expect("Error settings nonblocking");

    let mut websocket_pool = WebSocketPool::with_capacity(options.max_clients);

    if options.qrcode {
        render_server_qr_code(&address);
    }

    println!("Server now running at http://{}", address);

    for incoming in listener.incoming() {
        match incoming {
            Ok(s) => handle_client(s, &mut websocket_pool, &options, &address),
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => (),
            Err(e) => eprintln!("Something went wrong getting stream: {e}"),
        }

        websocket_pool.run();

        std::thread::sleep(Duration::from_millis(500));
    }
}
