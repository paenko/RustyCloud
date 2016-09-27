#[macro_use]
extern crate rustful;
extern crate rustc_serialize;
extern crate uuid;
extern crate bincode;

use std::io;
use std::io::prelude::*;

use rustful::{Server, Context, Response, TreeRouter, Handler};

use rustc_serialize::Encodable;
use rustc_serialize::json;
use bincode::rustc_serialize::{encode_into, encode, decode, decode_from};
use bincode::SizeLimit;

use std::fmt::{Display, Formatter};

use std::fs::{File, read_dir, remove_file};

use uuid::Uuid;

fn get_files(context: Context, response: Response) {
    let fileId = match context.variables.get("fileId") {
        Some(id) => {
            let files = rcFile::get_all();
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
struct rcFile {
    filename: String,
    fileId: Uuid,
    payload: Vec<u8>,
}

enum Route_Handler_Methods {
    get_all,
    get,
    post,
    delete,
}

struct Route_Handler(Route_Handler_Methods);

impl rcFile {
    fn new(filename: String, fileId: Uuid, payload: Vec<u8>) -> Self {
        rcFile {
            filename: filename,
            fileId: fileId,
            payload: payload,
        }
    }

    fn get_all() -> Vec<String> {
        let mut files: Vec<String> = Vec::new();
        let paths = read_dir("./data").unwrap();

        for path in paths {
            let file: String = path.unwrap().path().to_str().unwrap().to_string();
            files.push(file);
        }

        files
    }

    fn get(fileId: Uuid) -> rcFile {
        let file: rcFile = decode_from(&mut File::open(format!("./data/{}", fileId)).unwrap(),
                                       SizeLimit::Infinite)
            .unwrap();

        file
    }

    fn post(fileId: Uuid, filename: String, payload: Vec<u8>) {
        let mut f = File::open(format!("./data/{}", filename)).unwrap();
        let rc = rcFile::new(filename, fileId, payload);

        encode_into(&rc, &mut f, SizeLimit::Infinite);
    }

    fn delete(fileId: Uuid) {
        remove_file(format!("./data/{}", fileId));
    }
}

impl Handler for Route_Handler {
    fn handle_request(&self, context: Context, response: Response) {
        match self.0 {
            Route_Handler_Methods::get_all => {
                let json = json::encode(&rcFile::get_all()).unwrap();

                response.send(json);
            }
            Route_Handler_Methods::get => {
                let json = json::encode(&rcFile::get(Uuid::parse_str(&context.variables
                            .get("fileid")
                            .unwrap())
                        .unwrap()))
                    .unwrap();

                response.send(json);
            }
            Route_Handler_Methods::post => {}
            Route_Handler_Methods::delete => {}
        }
    }
}
