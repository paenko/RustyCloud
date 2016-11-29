#[macro_use]
extern crate rustc_serialize;
extern crate uuid;
extern crate bincode;
extern crate iron;
extern crate router;
extern crate params;
extern crate chrono;
extern crate base64;

use iron::status;
use router::Router;
use iron::prelude::*;
use params::{Params, Value};

use rustc_serialize::json::{self, Json};
use bincode::rustc_serialize::{encode_into, decode_from, EncodingError, DecodingError};
use bincode::SizeLimit;

use std::fs::{File, read_dir, remove_file, Metadata, metadata};
use std::time::{SystemTime, UNIX_EPOCH};

use uuid::Uuid;

use std::error::Error;
use std::fs::OpenOptions;

use std::io::Error as IoError;

use std::net::{SocketAddrV4, Ipv4Addr};

use chrono::*;
use base64::{encode, decode};

use std::rc::Rc;

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

    let mut f = OpenOptions::new()
        .write(true)
        .create(true)
        .open(format!("./data/{}", doc.file_id))
        .unwrap();

    encode_into(&doc, &mut f, SizeLimit::Infinite);

    Ok(Response::with((status::Ok, json::encode(&doc).unwrap())))
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

// Update on server
fn http_push(req: &mut Request) -> IronResult<Response> {
    let map = req.get_ref::<Params>().unwrap();

    let js: Json = json::Json::from_str(&format!("{:?}", map)).unwrap();

    let doc: Vec<(Uuid, DateTime<UTC>)> = json::decode(&js.to_string()).unwrap();

    let mut requesting: Vec<Uuid> = Vec::new();

    let files = RcFile::get_all();

    for (id, time) in doc {
        if (false) {
            requesting.push(id);
        }
    }

    Ok(Response::with((status::Ok, json::encode(&requesting).unwrap())))
}

// Update on client
fn http_pull(req: &mut Request) -> IronResult<Response> {
    let files = RcFile::get_all();
    let mut result: Vec<(Uuid, DateTime<UTC>)> = Vec::new();

    for f in files {
        let attr = metadata(f.filename).unwrap();
        let time = attr.modified().unwrap();

        result.push((f.file_id, system_time_to_date_time(time)));
    }

    Ok(Response::with((status::Ok, json::encode(&result).unwrap())))
}

fn system_time_to_date_time(t: SystemTime) -> DateTime<UTC> {
    let (sec, nsec) = match t.duration_since(UNIX_EPOCH) {
        Ok(dur) => (dur.as_secs() as i64, dur.subsec_nanos()),
        Err(e) => {
            // unlikely but should be handled
            let dur = e.duration();
            let (sec, nsec) = (dur.as_secs() as i64, dur.subsec_nanos());
            if nsec == 0 {
                (-sec, 0)
            } else {
                (-sec - 1, 1_000_000_000 - nsec)
            }
        }
    };
    UTC.timestamp(sec, nsec)
}

fn http_sync_file(req: &mut Request) -> IronResult<Response> {
    let map = req.get_ref::<Params>().unwrap();

    match map.find(&["payload"]) {
        Some(&Value::String(ref payload)) => {
            match map.find(&["fileid"]) {
                Some(&Value::String(ref file_id)) => {
                    let mut file = RcFile::get(Uuid::parse_str(file_id).unwrap()).unwrap();
                    file.payload = payload.to_string();

                    let mut file_handler = File::open(&file.filename).unwrap();

                    encode_into(&file, &mut file_handler, SizeLimit::Infinite);

                    Ok(Response::with(status::Ok))
                } 
                _ => Ok(Response::with(iron::status::NotFound)),
            }
        }
        _ => Ok(Response::with(iron::status::NotFound)),
    }
}

fn main() {
    let mut router = Router::new();
    router.get("/files", http_all_get, "get_files");
    router.get("/files/:file_id", http_get, "get_file");
    router.post("/file/push", http_push, "http_push");
    router.post("/file/pull", http_pull, "http_pull");
    router.post("/file/sync", http_sync_file, "http_sync_file");
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

    fn get_all() -> Vec<RcFile> {
        let mut files: Vec<RcFile> = Vec::new();
        let paths = read_dir("./data").unwrap();

        for path in paths {
            let file_name: String = path.unwrap().path().to_str().unwrap().to_string();

            let mut filehandler = File::open(file_name).unwrap();

            let file = decode_from(&mut filehandler, SizeLimit::Infinite).unwrap();

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
