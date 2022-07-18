#[derive(Debug)]
pub struct Bounds {
    start_x: u32,
    start_y: u32,
    len_x: u32,
    len_y: u32,
}

impl Bounds {
    pub fn new(start_y: u32, start_x: u32, len_x: u32, len_y: u32) -> Bounds {
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
    pub fn update_tach(this: &mut Tachometer, rpm_cur: u32, mut gear_int: u8) {
        this.rpm_cur = rpm_cur;

        // Decrement by one to account for reverse starting at 0
        if gear_int >= 1 {
            gear_int -= 1;
        }

        this.gear_char = gear_int;
    }

    pub fn display_tach(this: &Tachometer) {
        print!("{:?}", this);
    }
}


#[derive(Debug)]
pub struct TyreTemps {
    pub coords: Bounds,
    pub tyres: [f32; 4] // Tyres going clockwise from front left (0) to rear left (3)
}
