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

struct NetworkInfo {
    socket: std::net::UdpSocket,
    server_ip: String,
    listen_ip: String,
    heartbeat: std::time::SystemTime
}

struct TelemetryParser {
    physics: serde_json::Value,
    graphics: serde_json::Value,
    statics: serde_json::Value, 
    // Add blocks ?
    network: NetworkInfo
}

impl TelemetryParser {
    fn main(&mut self) {

        // CONSTRUCTION
        let server_ip_addr = match get_ip_from_args() {
            Some(val) => val,
            None => {
                println!("Failed to supply server IP as argument. Exiting...");
                std::process::exit(1);
            }
        };

        println!("Binding to socket with {}...", LISTEN_IP_ADDR_PORT);
        self.network.socket = std::net::UdpSocket::bind(LISTEN_IP_ADDR_PORT).unwrap();

        println!("Sending request for data to {}", &server_ip_addr);
        match self.network.socket.send_to("Give me the data!".as_bytes(), &server_ip_addr) {
            Ok(_size) => { },
            Err(_e) => panic!("Send request for data failed!")
        };

        self.wait_for_initial_message();
        // END CONSTRUCTION

        terminal_setup(); // From this point on we are in the alternate buffer

        let mut hotkeys: HashMap<event::Event, fn()> = HashMap::new();
        hotkeys.insert(self.build_key_event('q'), exit_terminal);

        let mut heartbeat = std::time::SystemTime::now();
        
        let mut blocks: Vec<Box<dyn TUIBlock>> = vec![
            Box::new(tui_blocks::Tachometer::new(0,0)),
            Box::new(tui_blocks::TyreTemps::new(0,6)),
            Box::new(tui_blocks::LapTimes::new(24,0)),
            Box::new(tui_blocks::Thermometer::new(24,6))];

        let mut static_data_initialized: bool = false;

        loop {
            if self.is_event_available() {
                match hotkeys.get(&event::read().unwrap()) {
                    Some(function) => function(),
                    None => { }
                }
            }

            match self.update_telemetry_from_connection() {
                Ok(()) => { },
                Err(_e) => { 
                    sleep_for_polling_rate();
                    continue;
                }
            };

            println!("{}", terminal::Clear(terminal::ClearType::All));

            if self.physics["packetId"] != 0 {
                if !static_data_initialized {
                    self.init_vector_statics(&mut blocks);
                    static_data_initialized = true;
                }

                for block in blocks.iter_mut() {
                    block.update(&self.physics, &self.graphics);
                    block.display();
                }
            }
            else {
                println!("{}{}", terminal::Clear(terminal::ClearType::All) ,cursor::MoveTo(0,0));
                println!("Connection established to {}, waiting for data...", server_ip_addr);
                static_data_initialized = false;
            }

            heartbeat = self.send_heartbeat_to_server(&server_ip_addr, heartbeat);
            sleep_for_polling_rate();
        }
    }

    fn update_telemetry_from_connection(&mut self) -> Result<(), serde_json::Error>{
        let mut buffer = [0; BUFFER_SIZE];
        let buf_len: usize = match self.network.socket.recv(&mut buffer) {
            Ok(buf_size) => buf_size,
            Err(e)       => panic!("{}", e)
        };

        let json_data: serde_json::Value = serde_json::from_slice(&buffer[0..buf_len])?;

        // TODO Still find a better way to do this
        self.physics = json_data["physics_data"].clone();
        self.graphics = json_data["graphics_data"].clone();
        self.statics = json_data["static_data"].clone();

        Ok(())
    }

    fn send_heartbeat_to_server(&mut self,
                                ip_addr: &String,
                                heartbeat: std::time::SystemTime) -> std::time::SystemTime {
        let current_time = std::time::SystemTime::now();
        let mut new_heartbeat = heartbeat;

        if current_time.duration_since(heartbeat).unwrap() > HEARTBEAT_DELTA_IN_MS {
            self.network.socket.send_to("I'm alive!".as_bytes(), ip_addr).unwrap();
            new_heartbeat = current_time;
        }

        return new_heartbeat;
    }

    fn init_vector_statics(&self, blocks: &mut Vec<Box<dyn TUIBlock>>) {
        for block in blocks.iter_mut() {
            block.init_statics(&self.statics);
        }
    }

    fn wait_for_initial_message(&mut self) {
        let mut buffer = [0; BUFFER_SIZE];
        println!("Waiting for connection...");
        self.network.socket.recv(&mut buffer).unwrap();
        println!("Connection successful!")
    }


    fn is_event_available(&self) -> bool {
        event::poll(std::time::Duration::from_millis(0)).unwrap()
    }

    fn build_key_event(&self, hotkey: char) -> event::Event {
        event::Event::Key(event::KeyEvent {
            code: event::KeyCode::Char(hotkey),
            modifiers: event::KeyModifiers::NONE
        })
    }
}

fn main() {
    println!("Hello, World!");
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


fn exit_terminal() {
    terminal_cleanup();
    std::process::exit(0);
}

fn sleep_for(time: u64) {
    std::thread::sleep(std::time::Duration::from_millis(time));
}

fn sleep_for_polling_rate() {
    sleep_for(POLLING_RATE_IN_MS);
}

