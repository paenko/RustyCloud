#[macro_use]
extern crate rustc_serialize;
extern crate uuid;
extern crate bincode;

extern crate iron;
extern crate router;
extern crate params;

extern crate byteorder;

use iron::status;
use router::Router;
use iron::prelude::*;
use params::{Params, Value};

use std::io;
use std::io::prelude::*;

use rustc_serialize::Encodable;
use rustc_serialize::json::{self, ToJson, Json};
use bincode::rustc_serialize::{encode_into, encode, decode, decode_from, EncodingError,
                               DecodingError};
use bincode::SizeLimit;

use std::fmt::{Display, Formatter};

use std::fs::{File, read_dir, remove_file};

use uuid::Uuid;

use std::error::Error;
use std::fs::OpenOptions;

use std::io::Error as IoError;

use std::net::{SocketAddrV4, Ipv4Addr};

use std::io::Cursor;
use byteorder::{BigEndian, ReadBytesExt};

use std::mem::transmute;

fn http_all_get(req: &mut Request) -> IronResult<Response> {
    let files = rcFile::get_all();

    let json = json::encode(&files).unwrap();

    Ok(Response::with((status::Ok, json)))
}

fn http_get(req: &mut Request) -> IronResult<Response> {
    let ref fileId = req.extensions
        .get::<Router>()
        .unwrap()
        .find("fileId")
        .unwrap();

    let document: rcFile = rcFile::get(Uuid::parse_str(fileId).unwrap()).unwrap();

    let json: String = json::encode(&document).unwrap();

    let res = Response::with((status::Ok, json));

    Ok(res)
}

fn http_post(req: &mut Request) -> IronResult<Response> {
    let map = req.get_ref::<Params>().unwrap();

    let js: Json = json::Json::from_str(&format!("{:?}", map)).unwrap();

    let doc: rcFile = json::decode(&js.to_string()).unwrap();

    Ok(Response::with((status::Ok, json::encode(&doc).unwrap())))
}

fn http_sync_get(req: &mut Request) -> IronResult<Response> {
    unimplemented!()
}
fn http_sync_post(req: &mut Request) -> IronResult<Response> {
    unimplemented!()
}
fn http_delete(req: &mut Request) -> IronResult<Response> {
    let ref fileId = req.extensions
        .get::<Router>()
        .unwrap()
        .find("fileId")
        .unwrap();

    let res = match rcFile::delete(Uuid::parse_str(fileId).unwrap()) {
        Ok(_) => Response::with((status::Ok)),
        Err(err) => Response::with((status::InternalServerError, err.description())),
    };

    Ok(res)
}

fn main() {
    let mut router = Router::new();
    router.get("/files", http_all_get, "get_files");
    router.get("/files/:fileId", http_get, "get_file");
    router.get("/file/sync", http_sync_get, "http_sync_get");
    router.post("/file/sync", http_sync_post, "http_sync_post");
    router.post("/file", http_post, "post_file");
    router.delete("/files/:fileId", http_delete, "delete_file");

    Iron::new(router).http(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 8080));

    println!("Server started on port 8080");
}

#[derive(RustcEncodable,RustcDecodable,Debug)]
struct rcFile {
    filename: String,
    fileId: Uuid,
    payload: String,
}

impl rcFile {
    fn new(filename: String, fileId: Uuid, payload: String) -> Self {
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
    fn post(fileId: Uuid, filename: String, payload: String) -> Result<rcFile, EncodingError> {
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
