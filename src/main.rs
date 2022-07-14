use std::net::UdpSocket;
use serde_json;

const BUFFER_SIZE:usize = 8192;
const IP_ADDR:&str      = "99.129.97.238:9000";

fn main()-> std::io::Result<()> {
    println!("Beginning server...");
    let socket = UdpSocket::bind("0.0.0.0:9001")?;
    socket.send_to("Give me the data!".as_bytes(), IP_ADDR)?;

    // Double check what this line below does
    let mut buffer = [0; BUFFER_SIZE];
    let buf_len: usize = socket.recv(&mut buffer)?;

    // TODO: This actually needs to be handled gracefully
    //       as sometimes UDP can get mangled.
    let telemetry: serde_json::Value = match serde_json::from_slice(&buffer[0..buf_len]) {
        Ok(json) => json,
        Err(e)   => panic!("{}", e),
    };

    let physics_data: &serde_json::Value = &telemetry["physics_data"];
    let _graphics_data: &serde_json::Value = &telemetry["graphics_data"];
    let _static_data: &serde_json::Value = &telemetry["static_data"];

    println!("Throttle: {}",   physics_data["gas"]);
    println!("Brake:    {}",   physics_data["brake"]);
    println!("Fuel:     {} L", physics_data["fuel"]);
    println!("\n");
    println!("Speed:    {} kmh", physics_data["speedKmh"]);
    println!("Gear:     {}",     physics_data["gear"]);
    println!("RPM:      {}",     physics_data["rpms"]);
    println!("\n");
    println!("Tires     {}", physics_data["tyreTemp"]);

    Ok(())
}

