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

    let mut my_display: Tachometer = Tachometer {
        coords: Bounds::new(0, 0, 0, 0),
        rpm_cur: 0,
        rpm_max: 0,
        rpm_bar: [false; 10],
        gear_char: 0
    };

    // Convert these to constructor functions?
    let mut my_tyres: TyreTemps = TyreTemps {
        coords: Bounds::new(0, 6, 0, 0),
        tyres:  [0f32, 0f32, 0f32, 0f32]
    };

    let mut my_times: LapTimes = LapTimes {
        coords: Bounds::new(24, 0, 0, 0),
        time_cur: 0 as u64,
        time_last: 0 as u64,
        time_best: 0 as u64
    };

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
            graphics: json_data["graphics_data"].clone(),
            statics: json_data["static_data"].clone()
        };

        if telemetry.physics["packetId"] != 0 {
            my_display.rpm_max = telemetry.statics["maxRpm"].as_u64().unwrap() as u32;

            print!("{}", terminal::Clear(terminal::ClearType::All));
            my_tyres.update(&telemetry.physics["tyreTemp"].as_array().unwrap());
            
            my_display.update(
                *&telemetry.physics["rpms"].as_u64().unwrap() as u32,
                *&telemetry.physics["gear"].as_u64().unwrap() as u8);

            my_times.update(telemetry.graphics["iCurrentTime"].as_u64().unwrap(),
                            telemetry.graphics["iLastTime"].as_u64().unwrap(), 
                            telemetry.graphics["iBestTime"].as_u64().unwrap());

            display_blocks(&my_display, &my_tyres, &my_times);
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

fn display_blocks(tacho: &Tachometer, tyres: &TyreTemps, times: &LapTimes) -> () {
    tacho.display();
    tyres.display();
    times.display();
}

// TODO: Create an initial setup function for statics data

