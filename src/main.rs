// James Robertson 2022
// ACCRT Engineer Rust
// Main.rs
//

use crossterm::{ cursor, terminal };

mod tui_blocks;
use crate::tui_blocks::*;

const BUFFER_SIZE: usize = 8192;
const HEARTBEAT_DELTA_IN_MS: std::time::Duration = std::time::Duration::from_millis(2000);
const LISTEN_IP_ADDR_PORT: &str = "0.0.0.0:9001";

struct TelemetryData {
    physics: serde_json::Value,
    graphics: serde_json::Value,
    statics: serde_json::Value
}

fn main()-> std::io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        println!("No IP Address supplied as an argument. Exiting...");
        std::process::exit(1);
    }

    let server_ip_addr = &args[1];

    println!("Beginning server...");
    let socket = std::net::UdpSocket::bind(LISTEN_IP_ADDR_PORT)?;

    println!("Sending request for data to {}", server_ip_addr);
    socket.send_to("Give me the data!".as_bytes(), server_ip_addr)?;

    let mut heartbeat = std::time::SystemTime::now();

    let mut block_tach  = Tachometer::new(0, 0);
    let mut block_tyres = TyreTemps::new(0, 6);
    let mut block_times = LapTimes::new(24, 0);
    let mut block_thermometer = Thermometer::new(24, 6);

    // This check_var is to satisfy the compiler's dead code warning
    // until we can get a keystroke to kill the program
    let check_var = false;

    while !check_var {
        let json_data = get_json_from_connection(&socket);

        // There has to be a better way
        let telemetry = TelemetryData {
            physics: json_data["physics_data"].clone(),
            graphics: json_data["graphics_data"].clone(),
            statics: json_data["static_data"].clone()
        };

        if telemetry.physics["packetId"] != 0 {
            block_tach.set_rpm_max(&telemetry.statics["maxRpm"]);

            print!("{}", terminal::Clear(terminal::ClearType::All));
            block_tyres.update(&telemetry.physics["tyreTemp"].as_array().unwrap());
            
            block_tach.update(*&telemetry.physics["rpms"].as_u64().unwrap() as u32,
                              *&telemetry.physics["gear"].as_u64().unwrap() as u8);

            block_times.update(telemetry.graphics["currentTime"].as_str(),
                               telemetry.graphics["lastTime"].as_str(), 
                               telemetry.graphics["bestTime"].as_str());

            block_thermometer.update(telemetry.physics["roadTemp"].as_f64().unwrap(),
                                     telemetry.physics["airTemp"].as_f64().unwrap());
            

            display_blocks(&block_tach, &block_tyres, &block_times, &block_thermometer);
        }
        else {
            print!("{}{}", terminal::Clear(terminal::ClearType::All), cursor::MoveTo(0,0));
            println!("Connection established to {}, waiting for data...", server_ip_addr);
        }

        heartbeat = send_heartbeat_to_server(&socket, heartbeat);
        sleep_for(16); // Roughly 60 Hz
    }

    Ok(())
}

fn send_heartbeat_to_server(socket: &std::net::UdpSocket,
                            heartbeat: std::time::SystemTime) -> std::time::SystemTime {
    let current_time = std::time::SystemTime::now();
    let mut new_heartbeat = heartbeat;

    if current_time.duration_since(heartbeat).unwrap() > HEARTBEAT_DELTA_IN_MS {
        socket.send_to("I'm alive!".as_bytes(), IP_ADDR).unwrap();
        new_heartbeat = current_time;
    }

    return new_heartbeat;
}

/// Wrapper function for Thread Sleep call for simplicity
fn sleep_for(time: u64) {
    std::thread::sleep(std::time::Duration::from_millis(time));
}

fn display_blocks(tacho: &Tachometer, tyres: &TyreTemps,
                  times: &LapTimes,   therm: &Thermometer) {
    tacho.display();
    tyres.display();
    times.display();
    therm.display();
}

fn get_json_from_connection(socket: &std::net::UdpSocket) -> serde_json::Value {
    // For the moment, this function will panic if it encounters an error.
    // This will be fixed when a better method for dealing with errors
    // is learned.
    let mut buffer = [0; BUFFER_SIZE];
    let buf_len: usize = match socket.recv(&mut buffer) {
        Ok(buf_size) => buf_size,
        Err(e)       => panic!("{}", e)
    };

    // TODO: This actually needs to be handled gracefully as sometimes UDP can get mangled.
    let json_data: serde_json::Value = match serde_json::from_slice(&buffer[0..buf_len]) {
        Ok(json) => json,
        Err(e)   => panic!("{}", e)
    };

    return json_data;
}

// TODO: Create an initial setup function for statics data

