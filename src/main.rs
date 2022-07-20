// James Robertson 2022
// ACCRT Engineer Rust
// Main.rs
//

use crossterm::{ cursor, terminal };

mod tui_blocks;
use crate::tui_blocks::*;

const BUFFER_SIZE: usize = 8192;
const IP_ADDR: &str      = "99.129.97.238:9000";

struct TelemetryData {
    physics: serde_json::Value,
    graphics: serde_json::Value,
    statics: serde_json::Value
}

fn main()-> std::io::Result<()> {
    println!("Beginning server...");
    let socket = std::net::UdpSocket::bind("0.0.0.0:9001")?;

    println!("Sending request for data to {}", IP_ADDR);
    socket.send_to("Give me the data!".as_bytes(), IP_ADDR)?;

    let mut block_tach  = Tachometer::new(0, 0);
    let mut block_tyres = TyreTemps::new(0, 6);
    let mut block_times = LapTimes::new(24, 0);
    let mut block_thermometer = Thermometer::new(24, 6);

    let check_var = false;

    while !check_var {
        // Double check what this line below does
        let mut buffer = [0; BUFFER_SIZE];
        let buf_len: usize = socket.recv(&mut buffer)?;

        // TODO: This actually needs to be handled gracefully as sometimes UDP can get mangled.
        let json_data: serde_json::Value = match serde_json::from_slice(&buffer[0..buf_len]) {
            Ok(json) => json,
            Err(e)   => panic!("{}", e),
        };

        // There has to be a better way
        let telemetry = TelemetryData {
            physics: json_data["physics_data"].clone(),
            graphics: json_data["graphics_data"].clone(),
            statics: json_data["static_data"].clone()
        };

        if telemetry.physics["packetId"] != 0 {
            block_tach.rpm_max = telemetry.statics["maxRpm"].as_u64().unwrap() as u32;

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
            println!("Connection established to {}, waiting for data...", IP_ADDR);
        }

        socket.send_to("I'm alive!".as_bytes(), IP_ADDR)?; // We COULD check this
        sleep_for(16); // Roughly 60 Hz
    }

    Ok(())
}

pub fn sleep_for(time: u64) -> () {
    std::thread::sleep(std::time::Duration::from_millis(time));
}

fn display_blocks(tacho: &Tachometer,
                  tyres: &TyreTemps,
                  times: &LapTimes,
                  therm: &Thermometer) {
    tacho.display();
    tyres.display();
    times.display();
    therm.display();
}

// TODO: Create an initial setup function for statics data

