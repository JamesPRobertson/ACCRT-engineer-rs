// James Robertson 2022
// TUI Blocks
//

use crossterm::cursor;

const RED_BLOCK: &str = "\x1b[91;1m▉\x1b[31;0m";
const WHITE_BLOCK: &str ="▉";
const RPM_BAR_LEN: usize = 0x10;

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
    pub coords: Bounds,
    pub rpm_cur: u32,
    pub rpm_max: u32,
    pub rpm_bar: [bool; RPM_BAR_LEN],
    pub gear_char: u8
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
}

pub struct TyreTemps {
    pub coords: Bounds,
    pub tyres: [f32; 4] // Tyres going clockwise from front left (0) to rear left (3)
}

impl TyreTemps {
    pub fn new(x: u16, y: u16) -> TyreTemps {
        return TyreTemps {
            coords: Bounds::new(x, y, 0, 0),
            tyres: [0 as f32; 4]
        }
    }

    pub fn display(&self) {
        print!("{}Tyres:",
               cursor::MoveTo(self.coords.start_x, self.coords.start_y));
        println!("{}{:.0}", 
               cursor::MoveTo(self.coords.start_x + 2, self.coords.start_y + 1),
                self.tyres[0]);
        println!("{}{:.0}", 
               cursor::MoveTo(self.coords.start_x + 8, self.coords.start_y + 1),
                self.tyres[1]);
        println!("{}{:.0}", 
               cursor::MoveTo(self.coords.start_x + 8, self.coords.start_y + 3),
                self.tyres[2]);
        println!("{}{:.0}", 
               cursor::MoveTo(self.coords.start_x + 2, self.coords.start_y + 3),
                self.tyres[3]);
    }

    // TODO: do this more smartly
    pub fn update(&mut self, temps: &Vec<serde_json::Value>) {
        //this.tyres = temps;
        
        self.tyres[0] = temps[0].as_f64().unwrap() as f32;
        self.tyres[1] = temps[1].as_f64().unwrap() as f32;
        self.tyres[2] = temps[2].as_f64().unwrap() as f32;
        self.tyres[3] = temps[3].as_f64().unwrap() as f32;
    }
}

pub struct LapTimes {
    pub coords: Bounds,
    pub time_cur: String,
    pub time_last: String,
    pub time_best: String
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
    pub coords: Bounds,
    pub temp_track: f64,
    pub temp_air: f64
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
