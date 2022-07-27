// James Robertson 2022
// ACCRT Engineer Rust
// Main
//

use crossterm::{ cursor, event, terminal };
use std::collections::HashMap;

mod tui_blocks;
use crate::tui_blocks::*;

const BUFFER_SIZE: usize = 8192;
const HEARTBEAT_DELTA_IN_MS: std::time::Duration = std::time::Duration::from_millis(2000);
const LISTEN_IP_ADDR_PORT: &str = "0.0.0.0:9001";
const POLLING_RATE_IN_MS: u64 = 16; // Roughly 60 Hz

// TODO: Maybe this shouldn't live here
struct TelemetryData {
    physics: serde_json::Value,
    graphics: serde_json::Value,
    statics: serde_json::Value
}

fn main()-> std::io::Result<()> {
    let server_ip_addr = match get_ip_from_args() {
        Some(val) => val,
        None => {
            println!("Failed to supply server IP as argument. Exiting...");
            std::process::exit(1);
        }
    };

    println!("Binding to socket with {}...", LISTEN_IP_ADDR_PORT);
    let socket = std::net::UdpSocket::bind(LISTEN_IP_ADDR_PORT)?;

    println!("Sending request for data to {}", &server_ip_addr);
    match socket.send_to("Give me the data!".as_bytes(), &server_ip_addr) {
        Ok(_size) => { },
        Err(_e) => panic!("Send request for data failed!")
    };

    wait_for_initial_message(&socket);

    terminal_setup(); // From this point on we are in the alternate buffer

    let mut hotkeys: HashMap<event::Event, fn()> = HashMap::new();
    hotkeys.insert(build_key_event('q'), exit_terminal);

    let mut heartbeat = std::time::SystemTime::now();
    
    let mut blocks: Vec<Box<dyn TUIBlock>> = vec![
        Box::new(tui_blocks::Tachometer::new(0,0)),
        Box::new(tui_blocks::TyreTemps::new(0,6)),
        Box::new(tui_blocks::LapTimes::new(24,0)),
        Box::new(tui_blocks::Thermometer::new(24,6))];

    // This check_var is to satisfy the compiler's dead code warning
    // until we can get a keystroke to kill the program
    let check_var = false;
    let mut static_data_initialized: bool = false;

    while !check_var {
        if is_event_available() {
            match hotkeys.get(&event::read().unwrap()) {
                Some(function) => function(),
                None => { }
            }
        }

        let telemetry = match get_telemetry_from_connection(&socket) {
            Some(val) => val,
            None => { 
                sleep_for_polling_rate();
                continue;
            }
        };

        println!("{}", terminal::Clear(terminal::ClearType::All));

        if telemetry.physics["packetId"] != 0 {
            if !static_data_initialized {
                init_vector_statics(&mut blocks, &telemetry.statics);
                static_data_initialized = true;
            }

            for block in blocks.iter_mut() {
                block.update(&telemetry.physics, &telemetry.graphics);
                block.display();
            }
        }
        else {
            println!("{}{}", terminal::Clear(terminal::ClearType::All) ,cursor::MoveTo(0,0));
            println!("Connection established to {}, waiting for data...", server_ip_addr);
            static_data_initialized = false;
        }

        heartbeat = send_heartbeat_to_server(&socket, &server_ip_addr, heartbeat);
        sleep_for_polling_rate();
    }

    Ok(())
}

fn get_ip_from_args() -> Option<String> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        return None;
    }
    else {
        return Some(String::from(&args[1]));
    }
}

fn terminal_setup() {
    crossterm::terminal::enable_raw_mode().unwrap();
    crossterm::execute!(std::io::stdout(), crossterm::terminal::EnterAlternateScreen).unwrap();
}

fn terminal_cleanup() {
    crossterm::execute!(std::io::stdout(), crossterm::terminal::LeaveAlternateScreen).unwrap();
    crossterm::terminal::disable_raw_mode().unwrap();
}

fn init_vector_statics(blocks: &mut Vec<Box<dyn TUIBlock>>, statics: &serde_json::Value) {
    for block in blocks.iter_mut() {
        block.init_statics(statics);
    }
}

fn wait_for_initial_message(socket: &std::net::UdpSocket) {
    let mut buffer = [0; BUFFER_SIZE];
    println!("Waiting for connection...");
    socket.recv(&mut buffer).unwrap();
    println!("Connection successful!")
}

fn send_heartbeat_to_server(socket: &std::net::UdpSocket,
                            ip_addr: &String,
                            heartbeat: std::time::SystemTime) -> std::time::SystemTime {
    let current_time = std::time::SystemTime::now();
    let mut new_heartbeat = heartbeat;

    if current_time.duration_since(heartbeat).unwrap() > HEARTBEAT_DELTA_IN_MS {
        socket.send_to("I'm alive!".as_bytes(), ip_addr).unwrap();
        new_heartbeat = current_time;
    }

    return new_heartbeat;
}

fn sleep_for(time: u64) {
    std::thread::sleep(std::time::Duration::from_millis(time));
}

fn sleep_for_polling_rate() {
    sleep_for(POLLING_RATE_IN_MS);
}

fn get_telemetry_from_connection(socket: &std::net::UdpSocket) -> Option<TelemetryData> {
    let mut buffer = [0; BUFFER_SIZE];
    let buf_len: usize = match socket.recv(&mut buffer) {
        Ok(buf_size) => buf_size,
        Err(e)       => panic!("{}", e)
    };

    let json_data: serde_json::Value = match serde_json::from_slice(&buffer[0..buf_len]) {
        Ok(json) => json,
        Err(_e)   => { return None; }
    };

    let telemetry = TelemetryData {
        physics:  json_data["physics_data"].clone(),
        graphics: json_data["graphics_data"].clone(),
        statics:  json_data["static_data"].clone()
    };

    return Some(telemetry);
}

fn is_event_available() -> bool {
    event::poll(std::time::Duration::from_millis(0)).unwrap()
}

fn build_key_event(hotkey: char) -> event::Event {
    event::Event::Key(event::KeyEvent {
        code: event::KeyCode::Char(hotkey),
        modifiers: event::KeyModifiers::NONE
    })
}

fn exit_terminal() {
    terminal_cleanup();
    std::process::exit(0);
}
