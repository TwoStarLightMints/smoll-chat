use std::collections::HashMap;
use std::fmt::Display;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

pub const MIME_MAP: &[(&str, &str)] = &[("js", "text/javascript"), ("css", "text/css")];

pub fn get_mime_type(file_name: &str) -> String {
    MIME_MAP
        .iter()
        .find(|kv| kv.0 == file_name.split(".").skip(1).next().unwrap())
        .expect(&format!("Could not determine MIME type for {file_name}"))
        .1
        .to_string()
}

#[derive(Debug, PartialEq, Eq)]
pub struct HttpRequest {
    pub method: String,
    pub resource: String,
    pub http_version: String,
    headers: HashMap<String, String>,
    pub body: Option<String>,
    querys: Option<HashMap<String, String>>,
}

impl HttpRequest {
    pub fn parse(raw_request: &str) -> Self {
        let sections = raw_request.split("\r\n").collect::<Vec<&str>>();

        let mut request_line = sections[0].split_whitespace();

        let mut headers: HashMap<String, String> = HashMap::new();

        sections
            .iter()
            .skip(1)
            .take_while(|s| !s.is_empty())
            .map(|s| s.to_string())
            .filter(|h| !h.is_empty())
            .for_each(|h| {
                let mut pieces = h.split(": ").map(|p| p.to_string());

                headers.insert(
                    pieces.next().unwrap().to_string(),
                    pieces.next().unwrap().to_string(),
                );
            });

        let body: Option<String>;

        match headers.get("Content-Length") {
            Some(len) => body = Some(sections.last().unwrap()[0..len.parse().unwrap()].to_string()),
            None => body = None,
        }

        let method = request_line.next().unwrap().to_string();

        let mut resource_set = request_line.next().unwrap().split("?");
        let resource = resource_set.next().unwrap().to_string();

        if let Some(query_set) = resource_set.next() {
            let query_set_iter = query_set.split("&");

            let mut querys = HashMap::new();

            query_set_iter.for_each(|q| {
                let mut key_val = q.split("=");

                querys.insert(
                    key_val.next().unwrap().to_string(),
                    key_val.next().unwrap().to_string(),
                );
            });

            let http_version = request_line.next().unwrap().to_string();

            Self {
                method,
                resource,
                http_version,
                headers,
                body,
                querys: Some(querys),
            }
        } else {
            let http_version = request_line.next().unwrap().to_string();

            Self {
                method,
                resource,
                http_version,
                headers,
                body,
                querys: None,
            }
        }
    }

    pub fn get_header(&self, header_name: &str) -> Option<&String> {
        self.headers.get(header_name)
    }
}

#[derive(Debug)]
pub struct HttpResponse {
    pub http_version: String,
    pub status_code: u32,
    pub status_message: String,
    headers: HashMap<String, String>,
    body: Option<String>,
}

impl HttpResponse {
    pub fn new(http_version: String, status_code: u32, status_message: String) -> Self {
        Self {
            http_version,
            status_code,
            status_message,
            headers: HashMap::new(),
            body: None,
        }
    }

    pub fn builder() -> HttpResponseBuilder {
        HttpResponseBuilder::default()
    }
}

impl Display for HttpResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let status_line = format!(
            "{} {} {}\r\n",
            self.http_version, self.status_code, self.status_message
        );

        let mut headers = String::new();

        for (k, v) in self.headers.iter() {
            headers.push_str(&format!("{}: {}\r\n", k, v));
        }

        match self.body.as_ref() {
            Some(b) => write!(f, "{}{}\r\n{}", status_line, headers, b),
            None => write!(f, "{}{}\r\n", status_line, headers),
        }
    }
}

#[derive(Default)]
pub struct HttpResponseBuilder {
    http_version: Option<String>,
    status_code: Option<u32>,
    status_message: Option<String>,
    headers: HashMap<String, String>,
    body: Option<String>,
}

impl HttpResponseBuilder {
    pub fn http_version(mut self, http_version: &str) -> Self {
        self.http_version = Some(http_version.to_string());

        self
    }

    pub fn status_code(mut self, status_code: u32) -> Self {
        self.status_code = Some(status_code);

        self
    }

    pub fn status_message(mut self, status_message: &str) -> Self {
        self.status_message = Some(status_message.to_string());

        self
    }

    pub fn add_header(mut self, key: &str, value: &str) -> Self {
        self.headers.insert(key.to_string(), value.to_string());

        self
    }

    pub fn body(mut self, body: &str) -> Self {
        self.body = Some(body.to_string());

        self
    }

    pub fn add_cookie(self, cookie: &str) -> Self {
        self.add_header("Set-Cookie", cookie)
    }

    pub fn build(self) -> HttpResponse {
        HttpResponse {
            http_version: self.http_version.unwrap_or("HTTP/1.1".to_string()),
            status_code: self.status_code.unwrap_or(404),
            status_message: self.status_message.unwrap_or("Not Found".to_string()),
            headers: self.headers,
            body: self.body,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_request_no_body() {
        let request = [
            "GET /favicon.ico HTTP/1.1",
            "Host: 192.168.4.28:1234",
            "User-Agent: Mozilla/5.0 (X11; Linux x86_64; rv:131.0) Gecko/20100101 Firefox/131.0",
            "Accept: image/avif,image/webp,image/png,image/svg+xml,image/*;q=0.8,*/*;q=0.5",
            "Accept-Language: en-US,en;q=0.5",
            "Accept-Encoding: gzip, deflate",
            "Connection: keep-alive",
            "Referer: http://192.168.4.28:1234/",
            "Priority: u=6",
        ];

        let mut request_str = String::new();

        for line in request {
            request_str.push_str(line);
            request_str.push_str("\r\n");
        }

        let mut headers = HashMap::new();

        headers.insert("Host".to_string(), "192.168.4.28:1234".to_string());
        headers.insert(
            "User-Agent".to_string(),
            "Mozilla/5.0 (X11; Linux x86_64; rv:131.0) Gecko/20100101 Firefox/131.0".to_string(),
        );
        headers.insert(
            "Accept".to_string(),
            "image/avif,image/webp,image/png,image/svg+xml,image/*;q=0.8,*/*;q=0.5".to_string(),
        );
        headers.insert("Accept-Language".to_string(), "en-US,en;q=0.5".to_string());
        headers.insert("Accept-Encoding".to_string(), "gzip, deflate".to_string());
        headers.insert("Connection".to_string(), "keep-alive".to_string());
        headers.insert("Connection".to_string(), "keep-alive".to_string());
        headers.insert(
            "Referer".to_string(),
            "http://192.168.4.28:1234/".to_string(),
        );
        headers.insert("Priority".to_string(), "u=6".to_string());

        request_str.push_str("\r\n");

        let control = HttpRequest {
            method: "GET".to_string(),
            resource: "/favicon.ico".to_string(),
            http_version: "HTTP/1.1".to_string(),
            headers,
            body: None,
            querys: None,
        };

        assert_eq!(control, HttpRequest::parse(&request_str));
    }

    #[test]
    fn test_parse_request_with_body() {
        let request = [
            "POST /login HTTP/1.1",
            "Host: 192.168.4.28:1234",
            "User-Agent: Mozilla/5.0 (X11; Linux x86_64; rv:131.0) Gecko/20100101 Firefox/131.0",
            "Accept: text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/png,image/svg+xml,*/*;q=0.8",
            "Accept-Language: en-US,en;q=0.5",
            "Accept-Encoding: gzip, deflate",
            "Content-Type: application/x-www-form-urlencoded",
            "Content-Length: 12",
            "Origin: http://192.168.4.28:1234",
            "Connection: keep-alive",
            "Referer: http://192.168.4.28:1234/",
            "Upgrade-Insecure-Requests: 1",
            "Priority: u=0, i",
        ];

        let mut request_str = String::new();

        for line in request {
            request_str.push_str(line);
            request_str.push_str("\r\n");
        }

        request_str.push_str("\r\n");

        request_str.push_str("username=asd");

        let mut headers = HashMap::new();

        headers.insert("Host".to_string(), "192.168.4.28:1234".to_string());
        headers.insert(
            "User-Agent".to_string(),
            "Mozilla/5.0 (X11; Linux x86_64; rv:131.0) Gecko/20100101 Firefox/131.0".to_string(),
        );
        headers.insert("Accept".to_string(), "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/png,image/svg+xml,*/*;q=0.8".to_string());
        headers.insert("Accept-Language".to_string(), "en-US,en;q=0.5".to_string());
        headers.insert("Accept-Encoding".to_string(), "gzip, deflate".to_string());
        headers.insert(
            "Content-Type".to_string(),
            "application/x-www-form-urlencoded".to_string(),
        );
        headers.insert("Content-Length".to_string(), "12".to_string());
        headers.insert("Origin".to_string(), "http://192.168.4.28:1234".to_string());
        headers.insert("Connection".to_string(), "keep-alive".to_string());
        headers.insert(
            "Referer".to_string(),
            "http://192.168.4.28:1234/".to_string(),
        );
        headers.insert("Upgrade-Insecure-Requests".to_string(), "1".to_string());
        headers.insert("Priority".to_string(), "u=0, i".to_string());

        let control = HttpRequest {
            method: "POST".to_string(),
            resource: "/login".to_string(),
            http_version: "HTTP/1.1".to_string(),
            headers,
            body: Some("username=asd".to_string()),
            querys: None,
        };

        assert_eq!(control, HttpRequest::parse(&request_str));
    }

    #[test]
    fn test_parse_request_with_query_parameter() {
        let request = [
            "GET /favicon.ico?user=unknown HTTP/1.1",
            "Host: 192.168.4.28:1234",
            "User-Agent: Mozilla/5.0 (X11; Linux x86_64; rv:131.0) Gecko/20100101 Firefox/131.0",
            "Accept: image/avif,image/webp,image/png,image/svg+xml,image/*;q=0.8,*/*;q=0.5",
            "Accept-Language: en-US,en;q=0.5",
            "Accept-Encoding: gzip, deflate",
            "Connection: keep-alive",
            "Referer: http://192.168.4.28:1234/",
            "Priority: u=6",
        ];

        let mut request_str = String::new();

        for line in request {
            request_str.push_str(line);
            request_str.push_str("\r\n");
        }

        let mut headers = HashMap::new();

        headers.insert("Host".to_string(), "192.168.4.28:1234".to_string());
        headers.insert(
            "User-Agent".to_string(),
            "Mozilla/5.0 (X11; Linux x86_64; rv:131.0) Gecko/20100101 Firefox/131.0".to_string(),
        );
        headers.insert(
            "Accept".to_string(),
            "image/avif,image/webp,image/png,image/svg+xml,image/*;q=0.8,*/*;q=0.5".to_string(),
        );
        headers.insert("Accept-Language".to_string(), "en-US,en;q=0.5".to_string());
        headers.insert("Accept-Encoding".to_string(), "gzip, deflate".to_string());
        headers.insert("Connection".to_string(), "keep-alive".to_string());
        headers.insert("Connection".to_string(), "keep-alive".to_string());
        headers.insert(
            "Referer".to_string(),
            "http://192.168.4.28:1234/".to_string(),
        );
        headers.insert("Priority".to_string(), "u=6".to_string());

        request_str.push_str("\r\n");

        let mut querys = HashMap::new();

        querys.insert("user".to_string(), "unknown".to_string());

        let control = HttpRequest {
            method: "GET".to_string(),
            resource: "/favicon.ico".to_string(),
            http_version: "HTTP/1.1".to_string(),
            headers,
            body: None,
            querys: Some(querys),
        };

        assert_eq!(control, HttpRequest::parse(&request_str));
    }

    #[test]
    fn test_parse_request_with_query_parameters() {
        let request = [
            "GET /favicon.ico?user=unknown&time=now HTTP/1.1",
            "Host: 192.168.4.28:1234",
            "User-Agent: Mozilla/5.0 (X11; Linux x86_64; rv:131.0) Gecko/20100101 Firefox/131.0",
            "Accept: image/avif,image/webp,image/png,image/svg+xml,image/*;q=0.8,*/*;q=0.5",
            "Accept-Language: en-US,en;q=0.5",
            "Accept-Encoding: gzip, deflate",
            "Connection: keep-alive",
            "Referer: http://192.168.4.28:1234/",
            "Priority: u=6",
        ];

        let mut request_str = String::new();

        for line in request {
            request_str.push_str(line);
            request_str.push_str("\r\n");
        }

        let mut headers = HashMap::new();

        headers.insert("Host".to_string(), "192.168.4.28:1234".to_string());
        headers.insert(
            "User-Agent".to_string(),
            "Mozilla/5.0 (X11; Linux x86_64; rv:131.0) Gecko/20100101 Firefox/131.0".to_string(),
        );
        headers.insert(
            "Accept".to_string(),
            "image/avif,image/webp,image/png,image/svg+xml,image/*;q=0.8,*/*;q=0.5".to_string(),
        );
        headers.insert("Accept-Language".to_string(), "en-US,en;q=0.5".to_string());
        headers.insert("Accept-Encoding".to_string(), "gzip, deflate".to_string());
        headers.insert("Connection".to_string(), "keep-alive".to_string());
        headers.insert("Connection".to_string(), "keep-alive".to_string());
        headers.insert(
            "Referer".to_string(),
            "http://192.168.4.28:1234/".to_string(),
        );
        headers.insert("Priority".to_string(), "u=6".to_string());

        request_str.push_str("\r\n");

        let mut querys = HashMap::new();

        querys.insert("user".to_string(), "unknown".to_string());
        querys.insert("time".to_string(), "now".to_string());

        let control = HttpRequest {
            method: "GET".to_string(),
            resource: "/favicon.ico".to_string(),
            http_version: "HTTP/1.1".to_string(),
            headers,
            body: None,
            querys: Some(querys),
        };

        assert_eq!(control, HttpRequest::parse(&request_str));
    }

    #[test]
    fn test_proper_http_response_formatting() {
        let http_response = HttpResponse::builder()
            .http_version("HTTP/1.1")
            .status_code(200)
            .status_message("OK")
            .add_cookie("username=asdf")
            .add_header("Content-Length", "22")
            .body("Simulated file content");

        let control = "HTTP/1.1 200 OK\r\nSet-Cookie: username=asdf\r\nContent-Length: 22\r\n\r\nSimulated file content".to_string();

        assert_eq!(control, http_response.build().to_string());
    }
}
