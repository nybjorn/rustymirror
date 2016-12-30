#![feature(proc_macro)]
#![feature(custom_attribute)]
#[macro_use]
extern crate serde_derive;

extern crate serde;
extern crate serde_json;
extern crate nanomsg;
extern crate hyper;



extern crate time;
extern crate clap;

use clap::{Arg, App, SubCommand};

use nanomsg::{Socket, Protocol};

use std::thread;
use std::time::Duration;
use std::io::{Read, Write};
use hyper::client::{Client, Request, Response};
use hyper::header::Connection;
use std::env;


#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct RealTimeDepartures {
    #[serde(rename = "StatusCode")]
    status_code: u32,
    #[serde(rename = "Message")]
    message: Option<String>,
    #[serde(rename = "ExecutionTime")]
    execution_time: u32,
    #[serde(rename = "ResponseData")]
    response_data: ResponseData
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct ResponseData {
    #[serde(rename = "LatestUpdate")]
    latest_update: String,
    #[serde(rename = "DataAge")]
    data_age: u32,
    #[serde(rename = "Metros")]
    metros: Vec<Journey>,
    #[serde(rename = "Buses")]
    buses: Vec<Journey>,
    #[serde(rename = "Trains")]
    trains: Vec<Journey>,
    #[serde(rename = "Trams")]
    trams: Vec<Journey>,
    #[serde(rename = "Ships")]
    ships: Vec<Journey>,
    #[serde(rename = "StopPointDeviations")]
    stop_point_deviations: Vec<StopDeviation>
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct StopDeviation {
    #[serde(rename = "StopInfo")]
    stop_info: StopInfo,
    #[serde(rename = "Deviation")]
    deviation: Deviation
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct StopInfo {
    #[serde(rename = "GroupOfLine")]
    group_of_line: String,
    #[serde(rename = "StopAreaName")]
    stop_area_name: String,
    #[serde(rename = "StopAreaNumber")]
    stop_area_number: u32,
    #[serde(rename = "TransportMode")]
    transport_mode: String
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Journey {
    #[serde(rename = "SiteId")]
    site_id: u32,
    #[serde(rename = "TransportMode")]
    transport_mode: String,
    #[serde(rename = "StopAreaName")]
    stop_area_name: String,
    #[serde(rename = "StopAreaNumber")]
    stop_area_number: u32,
    #[serde(rename = "StopPointNumber")]
    stop_point_number: u32,
    #[serde(rename = "LineNumber")]
    line_number: String,
    #[serde(rename = "Destination")]
    destination: String,
    #[serde(rename = "TimeTabledDateTime")]
    time_tabled_data_time: String,
    #[serde(rename = "ExpectedDateTime")]
    expected_date_time: String,
    #[serde(rename = "DisplayTime")]
    display_time: String,
    #[serde(rename = "Deviations")]
    deviations: Vec<Deviation>,
    #[serde(rename = "JourneyDirection")]
    journey_direction: u32
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Deviation {
    #[serde(rename = "Consequence")]
    consequence: String,
    #[serde(rename = "ImportanceLevel")]
    important_level: u32,
    #[serde(rename = "Text")]
    text: String
}

fn fetchRealTimeDepartures(client: &Client, api_path: &String, station: String) -> RealTimeDepartures {
    let station_path = format!("{}&siteid={}", api_path, station);
    println!("{}", station_path);
    let mut res = client.get(station_path.as_str()).send().unwrap();
    let mut buffer = String::new();
    res.read_to_string(&mut buffer).unwrap();
    println!("{:?}", buffer);
    return serde_json::from_str(&mut buffer).unwrap();
}

fn main() {

    let matches = App::new("message_traffic_sender")
        .arg(Arg::with_name("api_key")
                 .help("Api key from trafiklab") // Displayed when showing help info
                 .requires("stops")            // Says, "If the user uses "input", they MUST
                 .required(true)                // By default this argument MUST be present
        )
        .arg(Arg::with_name("stops")
            .help(", separated list of stations")
        )
        .get_matches();
    //let url ="ipc:///Stringp/pubsub.ipc";
//    if env::args().count() != 3 {
//        println!("Usage: {} <apikey> <stop[,s]",
//                 env::args().nth(0).unwrap());
//        std::process::exit(1);
//    }

  //  let station_pendeln = "1525";
  //  let station_buss = "1508";

    let api_key = matches.value_of("api_key").unwrap();
    let api_path = format!("https://api.sl.se/api2/realtimedepartures.json?key={}&timewindow=10", api_key);
    println!("{}", api_path);

    let stops = matches.value_of("stops").unwrap();

    let url = "tcp://127.0.0.1:5555";
    let mut socket = Socket::new(Protocol::Pub).unwrap();
    let mut endpoint = socket.connect(url).unwrap();


    match socket.set_ipv4_only(true) {
        Ok(..) => {},
        Err(err) => panic!("Failed to change ipv4 only on the socket: {}", err)
    }

    let client = Client::new();

    println!("Server is ready.");

    loop {
        let mut all_msg = String::new();
        all_msg = format!("Avgångar");
        for stop in stops.split(",") {
            println!("{}", stop);
            let deserialized_data: RealTimeDepartures = fetchRealTimeDepartures(&client, &api_path, String::from(stop));
            println!("{:?}", deserialized_data);
            for journey in deserialized_data.response_data.buses {
                let msg = format!("{} {}|{}|{}",
                                  journey.transport_mode,
                                  journey.line_number,
                                  journey.destination,
                                  journey.display_time );
                all_msg = format!("{}|{}", all_msg, msg);
            }
            for journey in deserialized_data.response_data.trams {
                let msg = format!("{} {}|{}|{}",
                                  journey.transport_mode,
                                  journey.line_number,
                                  journey.destination,
                                  journey.display_time );
                all_msg = format!("{}|{}", all_msg, msg);
            }
        }

        match socket.write_all(all_msg.as_bytes()) {
            Ok(..) => println!("Published '{}'.", all_msg),
            Err(err) => {
                println!("Server failed to publish '{}'.", err);
                break
            }
        }
        thread::sleep(Duration::from_millis(30000));
    }

    endpoint.shutdown();
}
