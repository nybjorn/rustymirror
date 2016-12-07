
extern crate hyper;
extern crate nanomsg;
extern crate time;
extern crate rss;

use nanomsg::{Socket, Protocol};

use std::thread;
use std::time::{Duration};
use std::io::{Read, Write};
use hyper::client::{Client, Request, Response};
use hyper::header::Connection;
use std::env;
use std::io::BufReader;
use rss::Channel;


fn main() {
    //let url ="ipc:///tmp/pubsub.ipc";
    let url = "tcp://127.0.0.1:8022";
    let mut socket = Socket::new(Protocol::Pub).unwrap();
    let mut endpoint = socket.connect(url).unwrap();
    let mut count = 1u32;

    match socket.set_ipv4_only(true) {
        Ok(..) => {},
        Err(err) => panic!("Failed to change ipv4 only on the socket: {}", err)
    }
    let client = Client::new();
    let mut response = match client.get("http://www.yr.no/place/Sweden/Stockholm/Stockholm/forecast.rss").send() {
        Ok(response) => response,
        Err(e) => panic!("Could not fetch Logentries data: {}", e)
    };

    let reader = BufReader::new(response);
    let channel = Channel::read_from(reader).unwrap();


    println!("{:#?}", channel);

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
