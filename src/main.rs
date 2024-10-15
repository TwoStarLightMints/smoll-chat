// Data to be sent with qr code
// Server's ip and port information

use std::env;
use std::fs::File;
use std::io::{BufRead, Read, Write};
use std::net::TcpListener;

use local_ip_address::local_ip;
use qrcode::QrCode;

fn main() {
    let args: Vec<_> = env::args().collect();

    if args.len() != 3 || !(&args[1] == "--port" || &args[1] == "-p") {
        return;
    }

    let mut addr = local_ip().unwrap().to_string();
    addr.push_str(":");
    addr.push_str(&args[2]);

    println!("{addr}");

    let listener = TcpListener::bind(&addr).unwrap();

    let code = QrCode::new(format!("http://{addr}")).unwrap();

    let qr_term = code
        .render()
        .light_color("\u{001b}[1;34;37;47m \u{001b}[0m")
        .dark_color("\u{001b}[1;34;37;40m \u{001b}[0m")
        .build();

    println!("Server now running at http://{}", &addr);
    println!("{qr_term}");

    for stream in listener.incoming() {
        let mut inc = stream.unwrap();

        let mut request = [0; 1024];

        let _ = inc.read(&mut request).unwrap();

        // Ex. GET / HTTP/1.1
        let request_lines = request.lines().next().unwrap().unwrap();

        let mut request_first_line = request_lines.split(' ').into_iter();

        let method = request_first_line.next().unwrap();
        let resource = request_first_line.next().unwrap();

        if method == "GET" {
            if resource == "/" {
                let mut index_page = File::open(format!(
                    "{}/resources/index.html",
                    env::current_dir().unwrap().to_str().unwrap()
                ))
                .unwrap();

                let mut index_content = String::new();

                index_page.read_to_string(&mut index_content).unwrap();

                inc.write(format!("HTTP/1.1 200 OK\r\n").as_bytes())
                    .unwrap();
                inc.write(format!("Content-Type: text/html; charset=UTF-8\r\n").as_bytes())
                    .unwrap();
                inc.write(format!("Content-Length: \r\n").as_bytes())
                    .unwrap();
                inc.write(format!("\r\n").as_bytes()).unwrap();
                inc.write(index_content.as_bytes()).unwrap();
            }
        }
    }
}
