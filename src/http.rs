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
    headers: Vec<String>,
    body: String,
}

impl HttpRequest {
    fn parse(raw_request: &str) -> Self {
        let lines: Vec<_> = raw_request.lines().collect();

        let mut request_line = lines[0].split_whitespace();

        let body = lines.last().unwrap().to_string();

        let headers: Vec<String> = lines
            .iter()
            .skip(1)
            .take(lines.len() - 2)
            .map(|h| h.to_string())
            .collect();

        let content_length = match headers.iter().position(|h| h.contains("Content-length")) {
            Some(ind) => headers[ind]
                .split("=")
                .last()
                .unwrap()
                .parse::<usize>()
                .unwrap(),
        };

        Self {
            method: request_line.next().unwrap().to_string(),
            resource: request_line.next().unwrap().to_string(),
            http_version: request_line.next().unwrap().to_string(),
            headers,
            body: body.chars().take(content_length).collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_request_from_firefox() {
        let request = "GET /favicon.ico HTTP/1.1
Host: 192.168.4.28:1234
User-Agent: Mozilla/5.0 (X11; Linux x86_64; rv:131.0) Gecko/20100101 Firefox/131.0
Accept: image/avif,image/webp,image/png,image/svg+xml,image/*;q=0.8,*/*;q=0.5
Accept-Language: en-US,en;q=0.5
Accept-Encoding: gzip, deflate
Connection: keep-alive
Referer: http://192.168.4.28:1234/
Priority: u=6";
    }
}
