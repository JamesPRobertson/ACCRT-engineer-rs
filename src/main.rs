use std::net::UdpSocket;
use serde_json;
//use termion::{cursor, color};
use termion::cursor;

const BUFFER_SIZE:usize = 8192;
const IP_ADDR:&str      = "99.129.97.238:9000";

struct TelemetryData {
    physics: serde_json::Value,
    _graphics: serde_json::Value,
    _statics: serde_json::Value
}

fn main()-> std::io::Result<()> {
    println!("Beginning server...");
    let socket = UdpSocket::bind("0.0.0.0:9001")?;
    socket.send_to("Give me the data!".as_bytes(), IP_ADDR)?;

    let check_var = false;

    while !check_var {
        // Double check what this line below does
        let mut buffer = [0; BUFFER_SIZE];
        let buf_len: usize = socket.recv(&mut buffer)?;

        // TODO: This actually needs to be handled gracefully
        //       as sometimes UDP can get mangled.
        let json_data: serde_json::Value = match serde_json::from_slice(&buffer[0..buf_len]) {
            Ok(json) => json,
            Err(e)   => panic!("{}", e),
        };

        // There has to be a better way
        let telemetry = TelemetryData {
            physics: json_data["physics_data"].clone(),
            _graphics: json_data["graphics_data"].clone(),
            _statics: json_data["static_data"].clone()
        };

        /*
        let physics_data: &serde_json::Value = json["physics_data"];
        let _graphics_data: &serde_json::Value = telemetry["graphics_data"];
        let _static_data: &serde_json::Value = telemetry["static_data"];
        */

        display_data(telemetry);
        sleep_for(250);
    }

    Ok(())
}

pub fn sleep_for(time: u64) -> () {
    std::thread::sleep(std::time::Duration::from_millis(time));
}

fn display_data(telemetry: TelemetryData) -> () {
    println!("{}{}", termion::clear::All, cursor::Goto(0,0));
    println!("Throttle: {}",   telemetry.physics["gas"]);
    println!("Brake:    {}",   telemetry.physics["brake"]);
    println!("Fuel:     {} L", telemetry.physics["fuel"]);
    println!("\n");
    println!("Speed:    {} kmh", telemetry.physics["speedKmh"]);
    println!("Gear:     {}",     telemetry.physics["gear"]);
    println!("RPM:      {}",     telemetry.physics["rpms"]);
    println!("\n");
    println!("Tires     {}", telemetry.physics["tyreTemp"]);
}

