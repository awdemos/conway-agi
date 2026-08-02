/// Player avatar that roams the grid and can interact with live cells.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Avatar {
    pub x: usize,
    pub y: usize,
    pub active: bool,
}

impl Avatar {
    pub fn new(x: usize, y: usize) -> Self {
        Self { x, y, active: true }
    }

    pub fn move_by(&mut self, dx: i32, dy: i32, width: usize, height: usize) {
        let nx = (self.x as i32 + dx).rem_euclid(width as i32) as usize;
        let ny = (self.y as i32 + dy).rem_euclid(height as i32) as usize;
        self.x = nx;
        self.y = ny;
    }

    pub fn position(&self) -> (usize, usize) {
        (self.x, self.y)
    }
}
