// James Robertson 2022
// TUI Blocks
//

use crossterm::cursor;

const RED_BLOCK: &str = "\x1b[91;1m▉\x1b[31;0m";
const WHITE_BLOCK: &str ="▉";
const RPM_BAR_LEN: usize = 10;

pub struct Bounds {
    start_x: u16,
    start_y: u16,
    len_x: u32,
    len_y: u32,
}

impl Bounds {
    pub fn new(start_x: u16, start_y: u16, len_x: u32, len_y: u32) -> Bounds {
        Bounds {start_x, start_y, len_x, len_y}
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
    pub fn update(&mut self, rpm_cur: u32, mut gear_int: u8) {
        self.rpm_cur = rpm_cur;

        // Decrement by one to account for reverse starting at 0
        if gear_int >= 1 {
            gear_int -= 1;
        }

        self.gear_char = gear_int;

        // This line may be inefficient
        let mut rpm_percentage = ((self.rpm_cur as f32 / self.rpm_max as f32) * 10.0).ceil() as usize;

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
        print!("{}┃", cursor::MoveTo(self.coords.start_x, self.coords.start_y + 4));
        
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
        println!("┃");
    }
}

//#[derive(Debug)]
pub struct TyreTemps {
    pub coords: Bounds,
    pub tyres: [f32; 4] // Tyres going clockwise from front left (0) to rear left (3)
}

impl TyreTemps {
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

// TODO: Switch to using strings
pub struct LapTimes {
    pub coords: Bounds,
    pub time_cur: u64,
    pub time_last: u64,
    pub time_best: u64
}

impl LapTimes {
    pub fn update(&mut self, time_cur: u64, time_last: u64, time_best: u64) {
        self.time_cur = time_cur;

        if self.time_last != time_last {
            self.time_last = time_last;
        }
        
        if self.time_best != time_best {
            self.time_best = time_best;
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
