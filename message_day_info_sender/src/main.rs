#![feature(proc_macro)]
#![feature(custom_attribute)]
#[macro_use]
extern crate serde_derive;

extern crate serde;
extern crate serde_json;
extern crate hyper;
extern crate nanomsg;
extern crate time;

use nanomsg::{Socket, Protocol};

use std::thread;
use std::time::{Duration};
use std::io::{Read, Write};
use hyper::client::{Client, Request, Response};
use hyper::header::Connection;
use std::env;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Dagar {
    cachetid: String,
    version: String,
    uri: String,
    startdatum: String,
    slutdatum: String,
    dagar: Vec<DagarInfo>
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct DagarInfo {
    datum: String,
    veckodag: String,
    vecka: u32,
    namnsdag:Vec<String>,
    flaggdag: String
}


struct Forecast {

}
fn main() {
    //let url ="ipc:///tmp/pubsub.ipc";
    let url = "tcp://127.0.0.1:8021";
    let mut socket = Socket::new(Protocol::Pub).unwrap();
    let mut endpoint = socket.bind(url).unwrap();
    let mut count = 1u32;

    match socket.set_ipv4_only(true) {
        Ok(..) => {},
        Err(err) => panic!("Failed to change ipv4 only on the socket: {}", err)
    }
    let client = Client::new();
    let start = time::now();
    println!("{} {} {}", start.tm_year + 1900, start.tm_mon+1, start.tm_mday);
    let uri_of_the_day = format!("http://api.dryg.net/dagar/v2.1/{}/{}/{}", start.tm_year + 1900, start.tm_mon+1, start.tm_mday);
    println!("{}", uri_of_the_day);
    let mut res = client.get(uri_of_the_day.as_str()).send().unwrap();
    let mut buffer = String::new();
    res.read_to_string(&mut buffer).unwrap();
    println!("{:?}", buffer);
    let deserialized_data: Dagar = serde_json::from_str(&mut buffer).unwrap();
    println!("{:?}", deserialized_data);
    println!("Server is ready.");

    loop {
        let msg = format!("{} #{}", "weather",  count);
        match socket.write_all(msg.as_bytes()) {
            Ok(..) => println!("Published '{}'.", msg),
            Err(err) => {
                println!("Server failed to publish '{}'.", err);
                break
            }
        }
        thread::sleep(Duration::from_millis(400));
        count += 1;
    }

    endpoint.shutdown();
}
