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
    let mut my_tyres : TyreTemps = TyreTemps {
        coords: Bounds::new(0, 6, 0, 0),
        tyres:  [0f32, 0f32, 0f32, 0f32]
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
            display_blocks(&my_display, &my_tyres);
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

fn display_blocks(tacho: &Tachometer, tyres: &TyreTemps) -> () {
    Tachometer::display(tacho);
    TyreTemps::display(tyres);
}

// TODO: Get the formatting working
fn display_data(telemetry: TelemetryData) -> () {
    print!("{}{}", terminal::Clear(terminal::ClearType::All), cursor::MoveTo(0,0));

    println!("Throttle:  {}", telemetry.physics["gas"]);
    println!("Brake:     {}", telemetry.physics["brake"]);
    println!("Fuel:      {:.1} L", telemetry.physics["fuel"]);
    println!("Speed:     {} kmh", telemetry.physics["speedKmh"]);
    println!("");
    /*
    println!("{}Speed:    {data} kmh", cursor::MoveTo(24, 0), data=telemetry.physics["speedKmh"]);
    println!("{}Gear:     {data}",     cursor::MoveTo(24, 1), data=telemetry.physics["gear"]);
    println!("{}RPM:      {data}",     cursor::MoveTo(24, 2), data=telemetry.physics["rpms"]);
    println!("");
    */
    println!("Tire Temps");
    println!("{}{}", cursor::MoveTo(14, 5), telemetry.physics["tyreTemp"][0]);
    println!("{}{}", cursor::MoveTo(24, 5), telemetry.physics["tyreTemp"][1]);
    println!("{}{}", cursor::MoveTo(14, 7), telemetry.physics["tyreTemp"][2]);
    println!("{}{}", cursor::MoveTo(24, 7), telemetry.physics["tyreTemp"][3]);
    println!("{}Current Lap Time:  {}", cursor::MoveTo(0, 10), telemetry.graphics["currentTime"]);
}

// TODO: Create an initial setup function for static data

