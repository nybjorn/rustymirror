#![feature(proc_macro)]
#![feature(custom_attribute)]
#[macro_use]
extern crate serde_derive;

extern crate serde;
extern crate serde_json;
extern crate hyper;
extern crate nanomsg;
extern crate time;
extern crate rocksdb;

use nanomsg::{Socket, Protocol};

use std::thread;
use std::time::{Duration};
use std::io::{Read, Write};
use hyper::client::{Client};
use rocksdb::{DB, Direction, IteratorMode};
use std::str;


#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Dagar {
    cachetid: String,
    version: String,
    uri: String,
    startdatum: String,
    slutdatum: String,
    dagar: Vec<DagarInfo>
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Default)]
struct DagarInfo {
    datum: String,
    veckodag: String,
    vecka: u32,
    namnsdag:Vec<String>,
    flaggdag: String
}


fn fetch_today_and_store_month(db: &DB, date: String) -> DagarInfo {
    let client = Client::new();
    let start = time::now();
    let uri_of_the_day = format!("http://api.dryg.net/dagar/v2.1/{}/{}", start.tm_year + 1900, start.tm_mon+1);
    println!("{}", uri_of_the_day);
    let mut res = client.get(uri_of_the_day.as_str()).send().unwrap();
    let mut buffer = String::new();
    res.read_to_string(&mut buffer).unwrap();
    println!("{:?}", buffer);
    let deserialized_data: Dagar = serde_json::from_str(&mut buffer).unwrap();
    let mut today: DagarInfo = Default::default();
    for day in deserialized_data.dagar {
        db.put(day.datum.as_bytes(), serde_json::to_string(&day).unwrap().as_bytes());
        if day.datum == date {
            today = day;
        }
    }
    let iter = db.iterator(IteratorMode::From(date.as_bytes(), Direction::Reverse)); // From a key in Direction::{forward,reverse}
    for (key, _) in iter {
        db.delete(key.as_ref());
    }
    return today;
}

fn main() {
    let db = DB::open_default("rocksdb_storage").unwrap();

    //let url ="ipc:///tmp/pubsub.ipc";
    let url = "tcp://127.0.0.1:8021";
    let mut socket = Socket::new(Protocol::Pub).unwrap();
    let mut endpoint = socket.connect(url).unwrap();

    match socket.set_ipv4_only(true) {
        Ok(..) => {},
        Err(err) => panic!("Failed to change ipv4 only on the socket: {}", err)
    }

    let start = time::now();
    let date =format!("{}-{}-{}", start.tm_year + 1900, start.tm_mon+1, start.tm_mday);

    let mut today: DagarInfo = Default::default();
    match db.get(date.as_bytes()) {
        Ok(Some(value)) => today = serde_json::from_str(value.to_utf8().unwrap()).unwrap(),
        Ok(None) => {
                        today = fetch_today_and_store_month(&db, date);
        },
        Err(e) => println!("operational problem encountered: {}", e),
    }


    /*
    let uri_of_the_day = format!("http://api.dryg.net/dagar/v2.1/{}/{}/{}", start.tm_year + 1900, start.tm_mon+1, start.tm_mday);
    println!("{}", uri_of_the_day);
    let mut res = client.get(uri_of_the_day.as_str()).send().unwrap();
    let mut buffer = String::new();
    res.read_to_string(&mut buffer).unwrap();
    println!("{:?}", buffer);
    let deserialized_data: Dagar = serde_json::from_str(&mut buffer).unwrap();
    println!("{:?}", deserialized_data);
    println!("Server is ready.");
    */
    let namnsdagsbarn: String = today.namnsdag.join(", ");
    let msg = format!("{} {}|Vecka {}|{}",
                      today.datum, today.veckodag,
                      today.vecka,
                      namnsdagsbarn);
    

    loop {
        match socket.write_all(msg.as_bytes()) {
            Ok(..) => println!("Published '{}'.", msg),
            Err(err) => {
                println!("Server failed to publish '{}'.", err);
                break
            }
        }
        thread::sleep(Duration::from_millis(4000));
    }

    endpoint.shutdown();
}
