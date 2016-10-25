#[macro_use]
extern crate rustc_serialize;
extern crate uuid;
extern crate bincode;

extern crate iron;
extern crate router;
extern crate params;

extern crate byteorder;

extern crate chrono;

use iron::status;
use router::Router;
use iron::prelude::*;
use params::Params;

use rustc_serialize::json::{self, Json};
use bincode::rustc_serialize::{encode_into, decode_from, EncodingError, DecodingError};
use bincode::SizeLimit;

use std::fs::{File, read_dir, remove_file};

use uuid::Uuid;

use std::error::Error;
use std::fs::OpenOptions;

use std::io::Error as IoError;

use std::net::{SocketAddrV4, Ipv4Addr};

use chrono::*;

fn http_all_get(req: &mut Request) -> IronResult<Response> {
    let files = RcFile::get_all();

    let json = json::encode(&files).unwrap();

    Ok(Response::with((status::Ok, json)))
}

fn http_get(req: &mut Request) -> IronResult<Response> {
    let ref file_id = req.extensions
        .get::<Router>()
        .unwrap()
        .find("file_id")
        .unwrap();

    let document: RcFile = RcFile::get(Uuid::parse_str(file_id).unwrap()).unwrap();

    let json: String = json::encode(&document).unwrap();

    let res = Response::with((status::Ok, json));

    Ok(res)
}

fn http_post(req: &mut Request) -> IronResult<Response> {
    let map = req.get_ref::<Params>().unwrap();

    let js: Json = json::Json::from_str(&format!("{:?}", map)).unwrap();

    let doc: RcFile = json::decode(&js.to_string()).unwrap();

    Ok(Response::with((status::Ok, json::encode(&doc).unwrap())))
}

fn http_sync_get(req: &mut Request) -> IronResult<Response> {
    unimplemented!()
}
fn http_sync_post(req: &mut Request) -> IronResult<Response> {
    unimplemented!()
}
fn http_delete(req: &mut Request) -> IronResult<Response> {
    let ref file_id = req.extensions
        .get::<Router>()
        .unwrap()
        .find("file_id")
        .unwrap();

    let res = match RcFile::delete(Uuid::parse_str(file_id).unwrap()) {
        Ok(_) => Response::with((status::Ok)),
        Err(err) => Response::with((status::InternalServerError, err.description())),
    };

    Ok(res)
}

fn main() {
    let mut router = Router::new();
    router.get("/files", http_all_get, "get_files");
    router.get("/files/:file_id", http_get, "get_file");
    router.get("/file/sync", http_sync_get, "http_sync_get");
    router.post("/file/sync", http_sync_post, "http_sync_post");
    router.post("/file", http_post, "post_file");
    router.delete("/files/:file_id", http_delete, "delete_file");

    Iron::new(router).http(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 8080));

    println!("Server started on port 8080");
}

#[derive(RustcEncodable,RustcDecodable,Debug)]
struct RcFile {
    filename: String,
    file_id: Uuid,
    payload: String,
    lastEdited: DateTime<UTC>,
}

impl RcFile {
    fn new(filename: String, file_id: Uuid, payload: String, lastEdited: DateTime<UTC>) -> Self {
        RcFile {
            filename: filename,
            file_id: file_id,
            payload: payload,
            lastEdited: lastEdited,
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

    fn get(file_id: Uuid) -> Result<RcFile, DecodingError> {
        let file: RcFile = try!(decode_from(&mut File::open(format!("./data/{}", file_id))
                                                .unwrap(),
                                            SizeLimit::Infinite));

        Ok(file)
    }

    // TODO error_handling for OpenOptions
    fn post(file_id: Uuid,
            filename: String,
            payload: String,
            lastEdited: DateTime<UTC>)
            -> Result<RcFile, EncodingError> {
        let mut f = OpenOptions::new()
            .write(true)
            .create(true)
            .open(format!("./data/{}", file_id))
            .unwrap();
        let rc = RcFile::new(filename, file_id, payload, lastEdited);

        try!(encode_into(&rc, &mut f, SizeLimit::Infinite));

        Ok(rc)
    }

    fn delete(file_id: Uuid) -> Result<(), IoError> {
        try!(remove_file(format!("./data/{}", file_id)));

        Ok(())
    }
}
