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

        display_data(telemetry);

        socket.send_to("I'm alive!".as_bytes(), IP_ADDR)?; // We COULD check this
        sleep_for(16); // Roughly 60 Hz
    }

    Ok(())
}

pub fn sleep_for(time: u64) -> () {
    std::thread::sleep(std::time::Duration::from_millis(time));
}

// TODO: Get the formatting working
fn display_data(telemetry: TelemetryData) -> () {
    println!("{}{}", termion::clear::All, cursor::Goto(1,1));
    println!("Throttle: {}",   telemetry.physics["gas"]);
    println!("Brake:    {}",   telemetry.physics["brake"]);
    println!("Fuel:     {0:.1} L", telemetry.physics["fuel"]);
    println!("");
    println!("{}Speed:    {data} kmh", cursor::Goto(24, 2), data=telemetry.physics["speedKmh"]);
    println!("{}Gear:     {data}",     cursor::Goto(24, 3), data=telemetry.physics["gear"]);
    println!("{}RPM:      {data}",     cursor::Goto(24, 4), data=telemetry.physics["rpms"]);
    println!("");
    println!("Tire Temps");
    println!("{}{}", cursor::Goto(14, 6), telemetry.physics["tyreTemp"][0]);
    println!("{}{}", cursor::Goto(24, 6), telemetry.physics["tyreTemp"][1]);
    println!("{}{}", cursor::Goto(14, 8), telemetry.physics["tyreTemp"][2]);
    println!("{}{}", cursor::Goto(24, 8), telemetry.physics["tyreTemp"][3]);
}

