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
use bincode::rustc_serialize::{encode_into, encode, decode, decode_from, EncodingError,
                               DecodingError};
use bincode::SizeLimit;

use std::fmt::{Display, Formatter};

use std::fs::{File, read_dir, remove_file};

use uuid::Uuid;

use std::error::Error;
use std::fs::OpenOptions;

use std::io::Error as IoError;

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

    fn get(fileId: Uuid) -> Result<rcFile, DecodingError> {
        let file: rcFile = try!(decode_from(&mut File::open(format!("./data/{}", fileId))
                                                .unwrap(),
                                            SizeLimit::Infinite));

        Ok(file)
    }

    // TODO error_handling for OpenOptions
    fn post(fileId: Uuid, filename: String, payload: Vec<u8>) -> Result<rcFile, EncodingError> {
        let mut f =
            OpenOptions::new().write(true).create(true).open(format!("./data/{}", fileId)).unwrap();
        let rc = rcFile::new(filename, fileId, payload);

        try!(encode_into(&rc, &mut f, SizeLimit::Infinite));

        Ok(rc)
    }

    fn delete(fileId: Uuid) -> Result<(), IoError> {
        try!(remove_file(format!("./data/{}", fileId)));

        Ok(())
    }
}

impl Handler for Route_Handler {
    fn handle_request(&self, mut context: Context, mut response: Response) {
        match self.0 {
            Route_Handler_Methods::get_all => {
                let json = json::encode(&rcFile::get_all()).unwrap();

                response.send(json);
            }
            Route_Handler_Methods::get => {
                let fileId = Uuid::parse_str(&context.variables.get("fileId").unwrap()).unwrap();

                match rcFile::get(fileId) {
                    Ok(d) => response.send(json::encode(&d).unwrap()),
                    Err(err) => response.send(err.description()),
                }
            }
            Route_Handler_Methods::post => {
                let body = match context.body.read_json_body() {
                    Ok(body) => {
                        let fileId = Uuid::new_v4();
                        let filename =
                            body.find("filename").and_then(|s| s.as_string()).unwrap().to_string();
                        let payload = match body.find("payload")
                            .and_then(|s| s.as_string()) {
                            Some(s) => s.as_bytes().to_vec(),
                            None => Vec::new(),
                        };

                        match rcFile::post(fileId, filename, payload) {
                            Ok(f) => response.send(format!("{}", fileId)),
                            Err(err) => response.send(err.description()),
                        }

                    }
                    Err(err) => {
                        response.send("Could not read body! Are you sure that it is valid json?")
                    }
                };
            }
            Route_Handler_Methods::delete => {
                match Uuid::parse_str(&context.variables
                    .get("fileId")
                    .unwrap()) {
                    Ok(id) => {
                        match rcFile::delete(id) {
                            Ok(_) => response.send("Ok"),
                            Err(err) => response.send(err.description()),
                        }
                    }
                    Err(err) => response.send("Could not parse FileId"),
                }
            }
        }
    }
}
