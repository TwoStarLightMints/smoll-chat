enum HttpStatus {
    OK,
    Found,
    SeeOther,
    NotFound,
}

#[derive(Debug, PartialEq, Eq)]
struct HttpRequest {
    method: String,
    resource: String,
    http_version: String,
    headers: Vec<String>,
    body: Option<String>,
}

impl HttpRequest {
    fn parse(raw_request: &str) -> Self {
        let sections = raw_request.split("\r\n").collect::<Vec<&str>>();

        let mut request_line = sections[0].split_whitespace();

        let body: Option<String>;

        if sections.last().unwrap().starts_with('\0') || sections.last().unwrap().is_empty() {
            body = None;
        } else {
            let content_len_row = sections
                .iter()
                .position(|h| h.to_lowercase().starts_with("content-length"))
                .expect("Body was found yet no content length was given");

            let content_len = sections[content_len_row]
                .split_whitespace()
                .last()
                .unwrap()
                .parse()
                .unwrap();

            body = Some(sections.last().unwrap()[0..content_len].to_string());
        }

        let headers = sections
            .iter()
            .skip(1)
            .take(if body.is_none() {
                sections.len() - 1
            } else {
                sections.len() - 2
            })
            .map(|s| s.to_string())
            .filter(|h| !h.is_empty())
            .collect();

        Self {
            method: request_line.next().unwrap().to_string(),
            resource: request_line.next().unwrap().to_string(),
            http_version: request_line.next().unwrap().to_string(),
            headers,
            body,
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

        request_str.push_str("\r\n");

        let control = HttpRequest {
            method: "GET".to_string(),
            resource: "/favicon.ico".to_string(),
            http_version: "HTTP/1.1".to_string(),
            headers: request[1..(request.len())]
                .iter()
                .map(|l| l.to_string())
                .collect(),
            body: None,
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

        let control = HttpRequest {
            method: "POST".to_string(),
            resource: "/login".to_string(),
            http_version: "HTTP/1.1".to_string(),
            headers: request[1..(request.len())]
                .iter()
                .map(|l| l.to_string())
                .collect(),
            body: Some("username=asd".to_string()),
        };

        assert_eq!(control, HttpRequest::parse(&request_str));
    }
}
