// James Robertson 2022
// TUI Blocks
//

use crossterm::cursor;

#[derive(Debug)]
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

#[derive(Debug)]
pub struct Tachometer {
    pub coords: Bounds,
    pub rpm_cur: u32,
    pub rpm_max: u32,
    pub gear_char: u8
    //tach_display: &str
}

impl Tachometer {
    pub fn update(&mut self, rpm_cur: u32, mut gear_int: u8) {
        self.rpm_cur = rpm_cur;

        // Decrement by one to account for reverse starting at 0
        if gear_int >= 1 {
            gear_int -= 1;
        }

        self.gear_char = gear_int;
    }

    pub fn display(&self) {
        print!("{}Tachometer", 
               cursor::MoveTo(self.coords.start_x, self.coords.start_y));
        println!("{}RPM:  {} / {}",
                 cursor::MoveTo(self.coords.start_x, self.coords.start_y + 1),
                 self.rpm_cur,
                 self.rpm_max);
        println!("{}Gear: {}", 
                 cursor::MoveTo(self.coords.start_x, self.coords.start_y + 2),
                 self.gear_char);

    }
}

#[derive(Debug)]
pub struct TyreTemps {
    pub coords: Bounds,
    pub tyres: [f32; 4] // Tyres going clockwise from front left (0) to rear left (3)
}

impl TyreTemps {
    pub fn display(&self) {
        print!("{}Tyres:",
               cursor::MoveTo(self.coords.start_x, self.coords.start_y));
        println!("{}{}", 
               cursor::MoveTo(self.coords.start_x, self.coords.start_y + 1),
                self.tyres[0]);
        println!("{}{}", 
               cursor::MoveTo(self.coords.start_x + 10, self.coords.start_y + 1),
                self.tyres[1]);
        println!("{}{}", 
               cursor::MoveTo(self.coords.start_x + 10, self.coords.start_y + 3),
                self.tyres[2]);
        println!("{}{}", 
               cursor::MoveTo(self.coords.start_x, self.coords.start_y + 3),
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
