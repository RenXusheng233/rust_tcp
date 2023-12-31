use http::{
    http_request::{Request, Resource},
    http_response::Response,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, env, fs};

pub trait Handler {
    fn handle(req: &Request) -> Response;
    fn load_file(file_name: &str) -> Option<String> {
        let default_path = format!("{}/public", env!("CARGO_MANIFEST_DIR"));
        let public_path = env::var("PUBLIC_PATH").unwrap_or(default_path);
        let full_path = format!("{}/{}", public_path, file_name);

        let contents = fs::read_to_string(full_path);

        contents.ok()
    }
}
pub struct PageNotFoundHandler;
pub struct StaticPageHandler;
pub struct WebServiceHandler;

#[derive(Serialize, Deserialize)]
pub struct OrderStatus {
    order_id: i32,
    order_date: String,
    status: String,
}

impl Handler for PageNotFoundHandler {
    fn handle(_req: &Request) -> Response {
        Response::new("404", None, Self::load_file("404.html"))
    }
}

impl Handler for StaticPageHandler {
    fn handle(req: &Request) -> Response {
        let Resource::Path(path) = &req.resource;
        let route: Vec<&str> = path.split("/").collect();

        match route[1] {
            "" => Response::new("200", None, Self::load_file("index.html")),
            "health" => Response::new("200", None, Self::load_file("health.html")),
            path => match Self::load_file(path) {
                Some(contents) => {
                    let mut map: HashMap<&str, &str> = HashMap::new();
                    if contents.ends_with(".css") {
                        map.insert("Content-Type", "text/css");
                    } else if contents.ends_with("js") {
                        map.insert("Content-Type", "text/javascript");
                    } else {
                        map.insert("Content-Type", "text/html");
                    }
                    Response::new("200", Some(map), Some(contents))
                }
                None => Response::new("404", None, Self::load_file("404.html")),
            },
        }
    }
}

impl WebServiceHandler {
    fn load_json() -> Vec<OrderStatus> {
        let default_path = format!("{}/data", env!("CARGO_MANIFEST_DIR"));
        let data_path = env::var("DATA_PATH").unwrap_or(default_path);
        let full_path = format!("{}/{}", data_path, "orders.json");

        let json_contents = fs::read_to_string(full_path).unwrap();
        let orders = serde_json::from_str(&json_contents).unwrap();

        orders
    }
}

impl Handler for WebServiceHandler {
    fn handle(req: &Request) -> Response {
        let Resource::Path(path) = &req.resource;
        let route: Vec<&str> = path.split("/").collect();

        match route[2] {
            // http://localhost:3000/api/shipping/orders
            "shipping" if route.len() > 3 && route[3] == "orders" => {
                let body = serde_json::to_string(&Self::load_json()).unwrap();
                let mut headers: HashMap<&str, &str> = HashMap::new();
                headers.insert("Content-Type", "application/json");

                Response::new("200", Some(headers), Some(body))
            }
            _ => Response::new("404", None, Self::load_file("404.html")),
        }
    }
}
