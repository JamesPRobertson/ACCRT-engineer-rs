// James Robertson
// Config
//

use std::collections::HashMap;
use crossterm::event;
use std::error::Error;
use std::fs::File;
use std::io::Read;
use std::str::FromStr;

const CONFIG_FILE_PATH: &str = "src/cfg/options.yaml";
const CONFIG_FILE_MAX_BUFFER_SIZE: usize = 0x4000; // 64 KB

#[derive(Debug)]
pub struct HotkeyFunction {
    function: fn(),
    name: String
}

impl HotkeyFunction {
    pub fn new(name_str: &str, function: fn()) -> HotkeyFunction {
        let name: String = String::from(name_str);

        HotkeyFunction {
            name,
            function
        }
    }
}

pub fn build_hotkeys(functions: Vec<HotkeyFunction>) -> HashMap<event::Event, fn()> {
    let mut hotkeys: HashMap<event::Event, fn()> = HashMap::new();

    let yaml = match load_yaml_file() {
        Ok(val) => val,
        Err(_) => { 
            // This will assume, for now, that the first function is exit terminal
            hotkeys.insert(build_key_event('q'), functions[0].function);
            return hotkeys;
        }
    };

    // TODO: potential failure cases
    for entry in functions {
        let hotkey_char = match convert_yaml_str_to_char(&yaml["hotkeys"][entry.name]) {
            Some(val) => val,
            None => { continue; } // TODO: we should log an error here
        };
        hotkeys.insert(build_key_event(hotkey_char), entry.function);
    }

    return hotkeys;
}

fn convert_yaml_str_to_char(value: &serde_yaml::Value) -> Option<char> {
    let _hotkey_char: char = match value.as_str() {
        Some(val) => match char::from_str(val) {
            Ok(it) => { return Some(it) },
            Err(_) => { return None }
        },
        None => { return None }
    };
}

fn load_yaml_file() -> Result<serde_yaml::Value, Box<dyn Error>> {
    let mut in_file = File::open(CONFIG_FILE_PATH)?;

    let mut buffer = [0; CONFIG_FILE_MAX_BUFFER_SIZE];
    let buf_len = in_file.read(&mut buffer)?;

    // lol idk if this line works :^)
    let yaml: serde_yaml::Value =serde_yaml::from_slice(&buffer[0..buf_len])?;

    Ok(yaml)
}

// TODO: Can potentially add support for key modifiers
fn build_key_event(hotkey: char) -> event::Event {
    event::Event::Key(event::KeyEvent {
        code: event::KeyCode::Char(hotkey),
        modifiers: event::KeyModifiers::NONE
    })
}

