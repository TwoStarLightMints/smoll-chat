use crate::error::RouteAlreadyRegistered;
use std::collections::HashMap;
use std::env;
use std::net::TcpListener;
use std::path::PathBuf;

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

    fn register_route(
        &mut self,
        route: String,
        resource: String,
    ) -> Result<(), RouteAlreadyRegistered> {
        if !self.routes.contains_key(&route) {
            self.routes.insert(route, resource);
            return Ok(());
        }

        Err(RouteAlreadyRegistered)
    }
}
