// James Robertson 2022
// TUI Blocks
//

use crossterm::cursor;

const RED_BLOCK: &str = "\x1b[91;1m▉\x1b[31;0m";
const WHITE_BLOCK: &str ="▉";
const RPM_BAR_LEN: usize = 0x10;

const COLOR_RESET: &str = "\x1b[31;0m";

const TYRE_COLOR_COLD: &str = "\x1b[96;1m";
const TYRE_COLOR_OPTIMAL: &str = "\x1b[92;1m";
const TYRE_COLOR_WARNING: &str = "\x1b[93;1m";
const TYRE_COLOR_TOO_HOT: &str = "\x1b[91;1m";

// TODO: These are rough estimates
const TYRE_NUM_COLD: f64 = 72.0;
const TYRE_NUM_OPTIMAL: f64 = 92.0;
const TYRE_NUM_WARNING: f64 = 100.0;


pub trait TUIBlock {
    fn update(&self, telemetry: &serde_json::Value);
    fn display(&mut self);
}

pub struct Bounds {
    start_x: u16,
    start_y: u16,
    _len_x: u32,
    _len_y: u32,
}

impl Bounds {
    pub fn new(start_x: u16, start_y: u16, _len_x: u32, _len_y: u32) -> Bounds {
        Bounds {start_x, start_y, _len_x, _len_y}
    }
}

pub struct Tachometer {
    coords: Bounds,
    rpm_cur: u32,
    rpm_max: u64, // This is public until the static init function is written in main.rs
    rpm_bar: [bool; RPM_BAR_LEN],
    gear_char: u8
}

impl Tachometer {
    pub fn new(x: u16, y: u16) -> Tachometer {
        return Tachometer {
            coords: Bounds::new(x, y, 0, 0),
            rpm_cur: 0,
            rpm_max: 0,
            rpm_bar: [false; RPM_BAR_LEN],
            gear_char: 0
        }
    }
    
    pub fn update(&mut self, rpm_cur: u32, mut gear_int: u8) {
        self.rpm_cur = rpm_cur;

        // Decrement by one to account for reverse starting at 0
        if gear_int >= 1 {
            gear_int -= 1;
        }

        self.gear_char = gear_int;

        let mut rpm_percentage = ((self.rpm_cur as f32 /
                                   self.rpm_max as f32)
                                  * RPM_BAR_LEN as f32).ceil() as usize;

        // May be a better way for this
        if rpm_percentage > self.rpm_bar.len() {
            rpm_percentage = self.rpm_bar.len() - 1;
        }

        for i in 0..rpm_percentage as usize {
            self.rpm_bar[i] = true;
        }
        for i in rpm_percentage..self.rpm_bar.len() {
            self.rpm_bar[i] = false;
        }
    }

    pub fn display(&self) {
        print!("{}Tachometer", 
               cursor::MoveTo(self.coords.start_x, self.coords.start_y));
        println!("{}RPM:  {} / {}",
                 cursor::MoveTo(self.coords.start_x + 2, self.coords.start_y + 1),
                 self.rpm_cur,
                 self.rpm_max);
        println!("{}Gear: {}", 
                 cursor::MoveTo(self.coords.start_x + 2, self.coords.start_y + 2),
                 self.gear_char);

        self.print_rpm_bar();
    }

    fn print_rpm_bar(&self) {
        let tachometer_end: &str = "┃";

        print!("{}{}", cursor::MoveTo(self.coords.start_x, self.coords.start_y + 4),
                       tachometer_end);
        
        if self.rpm_bar[RPM_BAR_LEN - 1] == true {
            for _i in 0..RPM_BAR_LEN - 1 {
                print!("{}", RED_BLOCK);
            }
        }
        else {
            for i in 0..(self.rpm_bar.len() - 1) {
                if self.rpm_bar[i] == true {
                    print!("{}", WHITE_BLOCK);
                }
                else {
                    print!(" ");
                }
            }
        }
        println!("{}", tachometer_end);
    }

    pub fn set_rpm_max(& mut self, rpm_max: &serde_json::Value) {
        self.rpm_max = match rpm_max.as_u64() {
            Some(num) => num,
            None      => 0 as u64
        }
    }
}

pub struct TyreTemps {
    coords: Bounds,
    tyres: [f64; 4] // Tyres going clockwise from front left (0) to rear left (3)
}

impl TyreTemps {
    pub fn new(x: u16, y: u16) -> TyreTemps {
        return TyreTemps {
            coords: Bounds::new(x, y, 0, 0),
            tyres: [0 as f64; 4]
        }
    }

    pub fn display(&self) {
        print!("{}Tyres:",
               cursor::MoveTo(self.coords.start_x, self.coords.start_y));

        // TODO: Can we iterate over the tyres somehow?
        self.print_tyre_with_offset(2, 1, 0);
        self.print_tyre_with_offset(8, 1, 1);
        self.print_tyre_with_offset(8, 3, 2);
        self.print_tyre_with_offset(2, 3, 3);
    }

    fn print_tyre_with_offset(&self, x_offset: u16, y_offset: u16, tyre_index: usize) {
        let text_color: &str;
        if self.tyres[tyre_index] < TYRE_NUM_COLD {
            text_color = TYRE_COLOR_COLD;
        }
        else if self.tyres[tyre_index] < TYRE_NUM_OPTIMAL {
            text_color = TYRE_COLOR_OPTIMAL;
        }
        else if self.tyres[tyre_index] < TYRE_NUM_WARNING {
            text_color = TYRE_COLOR_WARNING;
        }
        else {
            text_color = TYRE_COLOR_TOO_HOT;
        }

        println!("{}{}{:.0}{}", 
                 cursor::MoveTo(self.coords.start_x + x_offset,
                                self.coords.start_y + y_offset),
                 text_color,
                 self.tyres[tyre_index],
                 COLOR_RESET);
    }

    // TODO: Can still do this better
    pub fn update(&mut self, temps: &Vec<serde_json::Value>) {
        for i in 0..self.tyres.len() {
            self.tyres[i] = temps[i].as_f64().unwrap();
        }
    }
}

pub struct LapTimes {
    coords: Bounds,
    time_cur: String,
    time_last: String,
    time_best: String
}

impl LapTimes {
    pub fn new(x: u16, y: u16) -> LapTimes {
        return LapTimes {
            coords: Bounds::new(x, y, 0, 0),
            time_cur: String::new(),
            time_last: String::new(),
            time_best: String::new()
        }
    }

    pub fn update(&mut self, time_cur: Option<&str>, time_last: Option<&str>, time_best: Option<&str>) {
        match time_cur {
            Some(s) => self.time_cur = s.to_string(),
            None => ()
        }

        match time_last {
            Some(s)=> {
                if self.time_last != s.to_string() {
                    self.time_last = s.to_string();
                }
            }
            None => ()
        }

        match time_best {
            Some(s)=> {
                if self.time_best != s.to_string() {
                    self.time_best = s.to_string();
                }
            }
            None => ()
        }
    }

    pub fn display(&self) {
        println!("{}Lap Times", cursor::MoveTo(self.coords.start_x, self.coords.start_y));
        println!("{}Current Lap: {}",
                 cursor::MoveTo(self.coords.start_x + 2, self.coords.start_y + 1),
                 self.time_cur);
        println!("{}Last Lap:    {}",
                 cursor::MoveTo(self.coords.start_x + 2, self.coords.start_y + 2),
                 self.time_last);
        println!("{}Best Lap:    {}",
                 cursor::MoveTo(self.coords.start_x + 2, self.coords.start_y + 3),
                 self.time_best);
    }
}

pub struct Thermometer {
    coords: Bounds,
    temp_track: f64,
    temp_air: f64
}

impl Thermometer {
    pub fn new(x: u16, y: u16) -> Thermometer {
        return Thermometer {
            coords: Bounds::new(x, y, 0, 0),
            temp_track: 0 as f64,
            temp_air: 0 as f64
        }
    }

    pub fn update(&mut self, track: f64, air: f64) {
        self.temp_track = track;
        self.temp_air = air;
    }

    pub fn display(&self) {
        println!("{}Thermometer", cursor::MoveTo(self.coords.start_x, self.coords.start_y));
        println!("{}Track Temp: {:.1}",
                 cursor::MoveTo(self.coords.start_x + 2, self.coords.start_y + 1),
                 self.temp_track);
        println!("{}Air Temp:   {:.1}",
                 cursor::MoveTo(self.coords.start_x + 2, self.coords.start_y + 2),
                 self.temp_air);
    }
}
