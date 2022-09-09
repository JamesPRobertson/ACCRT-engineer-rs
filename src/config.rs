// James Robertson
// Config
//

use crossterm::{ cursor, event, terminal };

const CONFIG_FILE_PATH: &str = "src/cfg/options.yaml";
const CONFIG_FILE_MAX_BUFFER_SIZE: usize = 0x4000; // 64 KB

struct HotkeyFunction {
    function: fn(),
    name: String
}

pub fn build_hotkeys(functions: Vec<HotkeyFunction>) -> HashMap<event::Event, fn()> {
    let yaml = match load_yaml_file() {
        Ok(val) => val,
        Err(_) => { 
            // This will assume, for now, that the first function is exit terminal
            let mut default_hotkeys:HashMap<event::Event, fn()> = HashMap::new();
            default_hotkeys.insert(build_key_event('q'), functions[0].function);
            return default_hotkeys;
        }
    };
}

fn load_yaml_file() -> Result<serde_yaml::Value, Box<Error>> {
    let mut in_file = File::open(CONFIG_FILE_PATH)?;

    let mut buffer = [0; CONFIG_FILE_MAX_BUFFER_SIZE];
    let buf_len = in_file.read(&mut buffer)?;

    return serde_yaml::from_slice(&buffer[0..buf_len])?;
}

fn parse_function_from_yaml(function_name: &String) {
    
}

// TODO: Can potentially add support for key modifiers
fn build_key_event(hotkey: char) -> event::Event {
    event::Event::Key(event::KeyEvent {
        code: event::KeyCode::Char(hotkey),
        modifiers: event::KeyModifiers::NONE
    })
}

