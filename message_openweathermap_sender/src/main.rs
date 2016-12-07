#![feature(proc_macro)]
#![feature(custom_attribute)]
#[macro_use] extern crate serde_derive;

extern crate serde;
extern crate serde_json;
extern crate nanomsg;
extern crate hyper;
extern crate time;

use nanomsg::{Socket, Protocol};
use std::thread;
use std::time::Duration;
use std::io::{Read, Write};
use hyper::client::{Client, Request, Response};
use hyper::header::Connection;
use std::env;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Forecast {
    city: City,
    cod: String,
    message: f32,
    cnt: u32,
    list: Vec<List>
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct City {
    id: u32,
    name: String,
    coord: Coordinate,
    country: String
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Coordinate {
    lon: f32,
    lat: f32,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct List {
    dt: u32,
    main: ListMain,
    weather: Vec<Weather>,
    clouds: Clouds,
    wind: Wind,
    dt_txt: String
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct ListMain {
    temp: f32,
    temp_min: f32,
    temp_max: f32,
    pressure: f32,
    sea_level: f32,
    grnd_level: f32,
    humidity: f32,
    temp_kf: f32
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Weather {
    id: u32,
    main: String,
    description: String,
    icon: String
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Clouds {
    all: u32
}


#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Wind {
    speed: f32,
    deg: f32,
}

fn fetch_open_weather(client: &Client, api_path: &String) -> Forecast {
    let mut res = client.get(api_path.as_str()).send().unwrap();
    let mut buffer = String::new();
    res.read_to_string(&mut buffer).unwrap();
    println!("{:?}", buffer);
    return serde_json::from_str(&mut buffer).unwrap();
}

fn main() {
    if env::args().count() != 4 {
        println!("Usage: {} <apikey> <city> <lang>",
                 env::args().nth(0).unwrap());
        std::process::exit(1);
    }

    let api_key = env::args().nth(1).unwrap();
    let city = env::args().nth(2).unwrap();
    let lang = env::args().nth(3).unwrap();
    let api_path = format!("http://api.openweathermap.org/data/2.5/forecast?id={}&appid={}&units=metric&lang={}&cnt=3", city, api_key, lang);
    println!("{}", api_path);

    //let url ="ipc:///tmp/pubsub.ipc";
    let url = "tcp://127.0.0.1:8022";
    let mut socket = Socket::new(Protocol::Pub).unwrap();
    let mut endpoint = socket.connect(url).unwrap();

    let client = Client::new();


    println!("Server is ready.");

    loop {
        let mut all_msg = String::new();
        all_msg = format!("Väder");

        let deserialized_data: Forecast = fetch_open_weather(&client, &api_path);
        for list in deserialized_data.list {
            let msg = format!("{}|{}˚|{}",
                              list.dt_txt,
                              list.main.temp,
                              list.weather[0].description);
            all_msg = format!("{}|{}", all_msg, msg);
        }

        match socket.write_all(all_msg.as_bytes()) {
            Ok(..) => println!("Published '{}'.", all_msg),
            Err(err) => {
                println!("Server failed to publish '{}'.", err);
                break
            }
        }
        thread::sleep(Duration::from_millis(11 * 60 *10000));
    }

    endpoint.shutdown();
}
