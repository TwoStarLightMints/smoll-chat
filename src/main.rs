// Data to be sent with qr code
// Server's ip and port information

use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::{BufWriter, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;

use local_ip_address::local_ip;
use qrcode::QrCode;

enum HttpStatus {
    OK,
    Found,
    SeeOther,
    NotFound,
}

struct HttpRequest {
    method: String,
    resource: String,
    http_version: String,
    body: String,
}

impl HttpRequest {
    fn parse(raw_request: &str) -> Self {
        let lines: Vec<_> = raw_request.lines().collect();

        let mut start_line = lines[0].split_whitespace();

        Self {
            method: start_line.next().unwrap().to_string(),
            resource: start_line.next().unwrap().to_string(),
            http_version: start_line.next().unwrap().to_string(),
            body: lines.last().unwrap().replace("\0", "").to_string(),
        }
    }
}

struct Server {
    listener: TcpListener,
    static_files_dir: PathBuf,
    routes: HashMap<String, String>,
}

impl Server {
    fn new(address: &str, static_files_dir: Option<&str>) -> Self {
        let mut routes = HashMap::new();

        routes.insert("/".to_string(), "index.html".to_string());

        Self {
            listener: TcpListener::bind(address).expect("Error binding server to address"),
            static_files_dir: static_files_dir
                .unwrap_or(env::current_dir().unwrap().to_str().unwrap())
                .parse()
                .expect("Failed parsing static files directory"),
            routes,
        }
    }

    fn register_route(&mut self, route: String, resource: String) {
        if !self.routes.contains_key(&route) {
            self.routes.insert(route, resource);
        }
    }
}

fn render_server_qr_code(addr: &str, port: &str) {
    let code = QrCode::new(format!("http://{}", format!("{}:{}", addr, port))).unwrap();

    let qr_term = code
        .render()
        .light_color("\u{001b}[1;34;37;47m \u{001b}[0m")
        .dark_color("\u{001b}[1;34;37;40m \u{001b}[0m")
        .build();

    println!("{qr_term}");
}

fn mime_type(file: &str) -> Result<String, String> {
    match file.split(".").last() {
        Some("html") => Ok("text/html".to_string()),
        Some("css") => Ok("text/css".to_string()),
        Some("js") => Ok("text/javascript".to_string()),
        _ => Err("Mime type could not be determined".to_string()),
    }
}

fn retrieve_resource(locator: &str, stream: TcpStream) {
    let mut resource_path = env::current_dir().unwrap();
    resource_path.push(locator);

    let resource = File::open(resource_path);

    match resource {
        Ok(mut f) => {
            let mut resource_content = String::new();

            f.read_to_string(&mut resource_content)
                .expect("Could not read from resource");

            let mut response = BufWriter::new(stream);

            response
                .write(format!("HTTP/1.1 200 OK\r\n",).as_bytes())
                .unwrap();
            response
                .write_fmt(format_args!(
                    "Content-type: {}\r\n",
                    mime_type(locator.split("/").last().unwrap()).unwrap(),
                ))
                .unwrap();
            response
                .write_fmt(format_args!(
                    "Content-length: {}\r\n\r\n",
                    resource_content.len()
                ))
                .unwrap();
            response.write(resource_content.as_bytes()).unwrap();

            response.flush().unwrap();
        }
        Err(_) => {
            let mut resource_content = String::new();

            File::open(format!(
                "{}/resources/404.html",
                env::current_dir().unwrap().to_str().unwrap()
            ))
            .unwrap()
            .read_to_string(&mut resource_content)
            .unwrap();

            let mut response = BufWriter::new(stream);

            response
                .write("HTTP/1.1 404 Not Found\r\n".as_bytes())
                .unwrap();
            response
                .write("Content-type: text/html\r\n".as_bytes())
                .unwrap();
            response
                .write_fmt(format_args!(
                    "Content-length: {}\r\n\r\n",
                    resource_content.len()
                ))
                .unwrap();
            response.write(resource_content.as_bytes()).unwrap();

            response.flush().unwrap();
        }
    }
}

fn main() {
    let args: Vec<_> = env::args().collect();

    if args.len() != 3 || !(&args[1] == "--port" || &args[1] == "-p") {
        eprintln!("No port provided, please provide a port with -p or --port");
        return;
    }

    let port = &args[2];
    let addr = local_ip().unwrap().to_string();

    let listener =
        TcpListener::bind(format!("{}:{}", addr, port)).expect("Failed to initialize server");

    println!("Server now running at http://{}:{}", addr, port);

    if args.len() > 3 && args[3] == "--qrcode" {
        render_server_qr_code(&addr, port);
    }

    for stream in listener.incoming() {
        let mut inc = stream.unwrap();

        let mut request = [0; 1024];

        let _ = inc.read(&mut request).unwrap();

        let request = String::from_utf8(request.to_vec()).unwrap();

        // Ex. GET / HTTP/1.1
        let request_lines: Vec<_> = request.lines().collect();

        let mut request_first_line = request_lines[0].split(' ').into_iter();

        let method = request_first_line.next().unwrap();
        let resource = request_first_line.next().unwrap();

        println!("{} - {}", &method, &resource);

        if method == "GET" {
            if resource == "/" {
                retrieve_resource(
                    &format!(
                        "{}/resources/index.html",
                        env::current_dir().unwrap().to_str().unwrap()
                    ),
                    inc,
                );
            } else if resource == "/chat" {
                retrieve_resource(
                    &format!(
                        "{}/resources/chat.html",
                        env::current_dir().unwrap().to_str().unwrap()
                    ),
                    inc,
                );
            } else if resource == "/new-message" {
                let new_message = "{\"message\": \"This is another message for you\"}";

                inc.write(format!("HTTP/1.1 200 OK\r\n").as_bytes())
                    .unwrap();
                inc.write(format!("Content-Type: application/json\r\n").as_bytes())
                    .unwrap();
                inc.write(format!("Content-Length: {}\r\n", new_message.len()).as_bytes())
                    .unwrap();
                inc.write(format!("\r\n").as_bytes()).unwrap();
                inc.write(new_message.as_bytes()).unwrap();
            }
        } else if method == "POST" {
            if resource == "/login" {
                let username = &request_lines[request_lines
                    .iter()
                    .position(|l| l.starts_with("username"))
                    .unwrap()]
                .split("=")
                .last()
                .unwrap()
                .replace("\0", "");

                inc.write("HTTP/1.1 303 See other\r\n".as_bytes()).unwrap();
                inc.write("Content-type: text/html; charset=utf-8\r\n".as_bytes())
                    .unwrap();
                inc.write(
                    format!("Location: http://{}/chat\r\n", format!("{}:{}", addr, port))
                        .as_bytes(),
                )
                .unwrap();
                inc.write_all(
                    format!("Set-Cookie: username={username}; Path=/; HttpOnly").as_bytes(),
                )
                .unwrap();
                inc.write("Content-length: 0\r\n\r\n".as_bytes()).unwrap();
            }
        }
    }
}
