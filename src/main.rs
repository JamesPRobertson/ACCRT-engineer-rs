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

const TEMP_IP_ADDR: &str = "99.129.97.238:9000";

// TODO: Maybe this shouldn't live here
struct TelemetryData {
    physics: serde_json::Value,
    graphics: serde_json::Value,
    statics: serde_json::Value
}

fn main()-> std::io::Result<()> {
    /*
    let server_ip_addr = match get_ip_from_args() {
        Some(val) => val,
        None => {
            println!("Failed to supply IP, exiting");
            std::process::exit(1);
        }
    };
    */

    //let blocks: Vec<Box<dyn tui_blocks::TUIBlock>> = init_vector();
    let mut blocks = vec![Box::new(tui_blocks::Tachometer::new(0,0))];

    println!("Beginning server...");
    let socket = std::net::UdpSocket::bind(LISTEN_IP_ADDR_PORT)?;

    println!("Sending request for data to {}", TEMP_IP_ADDR);
    socket.send_to("Give me the data!".as_bytes(), TEMP_IP_ADDR)?;

    let mut heartbeat = std::time::SystemTime::now();
    
    /*
    let mut block_tach  = Tachometer::new(0, 0);
    let mut block_tyres = TyreTemps::new(0, 6);
    let mut block_times = LapTimes::new(24, 0);
    let mut block_thermometer = Thermometer::new(24, 6);
    */

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
            for i in 0..blocks.len() {
                let block = match blocks.get_mut(i) {
                    Some(val) => val,
                    None => { panic!("Out of bounds!") }
                };

                block.update(&telemetry.physics, &telemetry.graphics);
                block.display();
            }
        }
        else {
            print!("{}{}", terminal::Clear(terminal::ClearType::All), cursor::MoveTo(0,0));
            println!("Connection established to {}, waiting for data...", TEMP_IP_ADDR);
        }

        heartbeat = send_heartbeat_to_server(&socket, heartbeat);
        sleep_for(16); // Roughly 60 Hz
    }

    Ok(())
}

fn get_ip_from_args() -> Option<String> {
    // TODO: Rewrite as function that returns the IP
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        return None;
    }
    else {
        return Some(String::from(&args[1]));
    }
}

fn init_vector() -> Vec<Box<dyn tui_blocks::TUIBlock>> {
    return vec![Box::new(tui_blocks::Tachometer::new(0,0))]
}

fn send_heartbeat_to_server(socket: &std::net::UdpSocket,
                            heartbeat: std::time::SystemTime) -> std::time::SystemTime {
    let current_time = std::time::SystemTime::now();
    let mut new_heartbeat = heartbeat;

    if current_time.duration_since(heartbeat).unwrap() > HEARTBEAT_DELTA_IN_MS {
        socket.send_to("I'm alive!".as_bytes(), TEMP_IP_ADDR).unwrap();
        new_heartbeat = current_time;
    }

    return new_heartbeat;
}

/// Wrapper function for Thread Sleep call for simplicity
fn sleep_for(time: u64) {
    std::thread::sleep(std::time::Duration::from_millis(time));
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

