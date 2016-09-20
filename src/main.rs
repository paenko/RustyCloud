#[macro_use]
extern crate rustful;
extern crate rustc_serialize;

use rustful::{Server, Context, Response, TreeRouter, Handler};
use rustc_serialize::json;
use std::fmt::{Display, Formatter, Result};

fn get_files(context: Context, response: Response) {
    let fileId = match context.variables.get("fileId") {
        Some(id) => {
            let files = File::get_all();
            response.send(format!("test"))
            //            response.send(format!("{:?}",
        }
        None => response.send(format!("Hello, {}!", "test")),
    };
}

fn main() {
    let server = Server {
            host: 8080.into(),
            handlers: insert_routes!{
            TreeRouter::new() => {
                "/files" => Get: Route_Handler(Route_Handler_Methods::get_all),
                "/files/:fileId" => Get: Route_Handler(Route_Handler_Methods::get),
                "/file" => Post: Route_Handler(Route_Handler_Methods::post),
                "/files/:fileId" => Delete: Route_Handler(Route_Handler_Methods::delete)
            }
        },
            ..Server::default()
        }
        .run();

    println!("Server started on port 8080");
}

#[derive(RustcEncodable,RustcDecodable)]
struct File {
    filename: String,
    path: String,
}

enum Route_Handler_Methods {
    get_all,
    get,
    post,
    delete,
}

struct Route_Handler(Route_Handler_Methods);

impl File {
    fn get_all() -> Vec<File> {
        let files: Vec<File> = Vec::new();

        files
    }

    fn get(fileId: &str) -> File {
        File {
            filename: "test".to_string(),
            path: "test".to_string(),
        }
    }

    fn post() {}

    fn delete() {}
}

impl Handler for Route_Handler {
    fn handle_request(&self, context: Context, response: Response) {
        match self.0 {
            Route_Handler_Methods::get_all => {
                let json = json::encode(&File::get_all()).unwrap();

                response.send(json);
            }
            Route_Handler_Methods::get => {
                let json = json::encode(&File::get(&context.variables.get("fileid").unwrap()))
                    .unwrap();

                response.send(json);
            }
            Route_Handler_Methods::post => {}
            Route_Handler_Methods::delete => {}
        }
    }
}
