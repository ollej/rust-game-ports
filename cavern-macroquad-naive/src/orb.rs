pub struct Orb {}

impl Orb {
    #[allow(dead_code)]
    pub fn update(&mut self) {
        eprintln!("WRITEME: Orb#update");
    }

    #[allow(dead_code)]
    pub fn draw(&self) {
        eprintln!("WRITEME: Orb#draw");
    }
}