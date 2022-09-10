// James Robertson 2022
// ACCRT Engineer Rust
// Main
//

use crossterm::{ cursor, event, terminal };

mod tui_blocks;
mod config;
use crate::tui_blocks::*;

const BUFFER_SIZE: usize = 8192;
const HEARTBEAT_DELTA_IN_MS: std::time::Duration = std::time::Duration::from_millis(2000);
const LISTEN_IP_ADDR_PORT: &str = "0.0.0.0:9001";

// TODO: Should polling rate be a part of telemetry parser?
//       this would move the thread sleep functions into
//       TelemetryParser the object
const POLLING_RATE_IN_MS: u64 = 16; // Roughly 60 Hz

struct NetworkInfo {
    socket:    std::net::UdpSocket,
    server_ip: String,
    _listen_ip: String,
    heartbeat: std::time::SystemTime
}

impl NetworkInfo {
    fn new(listen_ip: String, server_ip: String) -> NetworkInfo {
        NetworkInfo {
            socket: std::net::UdpSocket::bind(&listen_ip).unwrap(),
            server_ip,
            _listen_ip: listen_ip,
            heartbeat: std::time::SystemTime::now()
        }
    }
}

struct TelemetryParser {
    physics: serde_json::Value,
    graphics: serde_json::Value,
    statics: serde_json::Value, 
    blocks: Vec<Box<dyn TUIBlock>>,
    network: NetworkInfo
}

impl TelemetryParser {
    // TODO: Consider making this non looping
    fn main(&mut self) {
        // TODO: We must think this through with async
        let function_map: Vec<config::HotkeyFunction> = vec![
            config::HotkeyFunction::new("exit_terminal", exit_terminal)
        ];
        let hotkeys = config::build_hotkeys(function_map);
        // - to here

        let mut static_data_initialized: bool = false;

        loop {
            if TelemetryParser::is_event_available() {
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

            // TODO instead of this, we need to know when we are actually getting good data
            if self.physics["packetId"] != 0 {
                if !static_data_initialized {
                    self.init_vector_statics();
                    static_data_initialized = true;
                }

                for block in self.blocks.iter_mut() {
                    block.update(&self.physics, &self.graphics);
                    block.display();
                }
            }
            else {
                println!("{}{}", terminal::Clear(terminal::ClearType::All) ,cursor::MoveTo(0,0));
                println!("Connection established to {}, waiting for data...", self.network.server_ip);
                static_data_initialized = false;
            }

            self.send_heartbeat_to_server();
            sleep_for_polling_rate();
        }
    }

    fn new(listen_ip_addr: String, server_ip_addr: String) -> TelemetryParser {
        return TelemetryParser {
            physics: serde_json::Value::Null,
            graphics: serde_json::Value::Null,
            statics: serde_json::Value::Null,
            blocks: TelemetryParser::generate_blocks(),
            network: NetworkInfo::new(listen_ip_addr, server_ip_addr)
        }
    }

    fn generate_blocks() -> Vec<Box<dyn TUIBlock>> {
        let blocks: Vec<Box<dyn TUIBlock>> = vec![
            Box::new(tui_blocks::Tachometer::new(0,0)),
            Box::new(tui_blocks::TyreTemps::new(0,6)),
            Box::new(tui_blocks::LapTimes::new(24,0)),
            Box::new(tui_blocks::Thermometer::new(24,6)),
            Box::new(tui_blocks::BrakeTemps::new(0,12)),
            Box::new(tui_blocks::TyrePressures::new(24,12))
        ];

        return blocks;
    }

    fn preconnect_setup(&self) {
        println!("Sending request for data to {}", &self.network.server_ip);

        match self.network.socket.send_to("Give me the data!".as_bytes(), &self.network.server_ip) {
            Ok(_size) => { },
            Err(_e) => panic!("Send request for data failed!")
        };

        self.wait_for_initial_message();
    }

    fn update_telemetry_from_connection(&mut self) -> Result<(), serde_json::Error> {
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

    fn send_heartbeat_to_server(&mut self) {
        let current_time = std::time::SystemTime::now();
        
        if current_time.duration_since(self.network.heartbeat).unwrap() > HEARTBEAT_DELTA_IN_MS {
            self.network.socket.send_to("I'm alive!".as_bytes(), &self.network.server_ip).unwrap();
            self.network.heartbeat = current_time;
        }
    }

    fn init_vector_statics(&mut self) {
        for block in self.blocks.iter_mut() {
            block.init_statics(&self.statics);
        }
    }

    fn wait_for_initial_message(&self) {
        let mut buffer = [0; BUFFER_SIZE];
        println!("Waiting for connection...");
        self.network.socket.recv(&mut buffer).unwrap();
        println!("Connection successful!");
        // TODO we may have to update heartbeat, we may not
    }

    fn is_event_available() -> bool {
        event::poll(std::time::Duration::from_millis(0)).unwrap()
    }

}

fn main() {
    let server_ip_addr = match get_ip_from_args() {
        Some(val) => val,
        None => {
            println!("Failed to supply server IP as argument. Exiting...");
            std::process::exit(1);
        }
    };

    let mut telemetry_parser = TelemetryParser::new(String::from(LISTEN_IP_ADDR_PORT), server_ip_addr);

    telemetry_parser.preconnect_setup();

    terminal_setup();

    telemetry_parser.main();
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

fn _switch_blocks_view() {
    return
}

fn sleep_for(time: u64) {
    std::thread::sleep(std::time::Duration::from_millis(time));
}

fn sleep_for_polling_rate() {
    sleep_for(POLLING_RATE_IN_MS);
}

