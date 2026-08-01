/// A chemical signal channel that diffuses and decays across the grid.
/// Each cell position stores the local signal intensity.
#[derive(Clone, Debug)]
pub struct SignalGrid {
    width: usize,
    height: usize,
    values: Vec<u8>,
}

impl SignalGrid {
    pub fn new(width: usize, height: usize) -> Self {
        assert!(
            width > 0 && height > 0,
            "signal grid dimensions must be positive"
        );
        Self {
            width,
            height,
            values: vec![0; width * height],
        }
    }

    pub fn size(&self) -> (usize, usize) {
        (self.width, self.height)
    }

    fn index(&self, x: usize, y: usize) -> usize {
        y * self.width + x
    }

    pub fn get(&self, x: usize, y: usize) -> u8 {
        self.values[self.index(x, y)]
    }

    pub fn set(&mut self, x: usize, y: usize, value: u8) {
        let idx = self.index(x, y);
        self.values[idx] = value;
    }

    pub fn add(&mut self, x: usize, y: usize, amount: u8) {
        let idx = self.index(x, y);
        self.values[idx] = self.values[idx].saturating_add(amount);
    }

    pub fn signal_at(&self, x: isize, y: isize) -> u8 {
        let x = x.rem_euclid(self.width as isize) as usize;
        let y = y.rem_euclid(self.height as isize) as usize;
        self.get(x, y)
    }

    /// Average signal of the 8 neighbors (used for diffusion).
    pub fn neighbor_average(&self, x: usize, y: usize) -> u8 {
        let xi = x as isize;
        let yi = y as isize;
        let mut sum: u16 = 0;
        let mut count: u16 = 0;
        for dy in -1..=1 {
            for dx in -1..=1 {
                if dx == 0 && dy == 0 {
                    continue;
                }
                sum += u16::from(self.signal_at(xi + dx, yi + dy));
                count += 1;
            }
        }
        (sum / count) as u8
    }

    /// Diffuse and decay signals into the next buffer.
    pub fn diffuse(&self, next: &mut SignalGrid, decay: u8) {
        let (w, h) = self.size();
        assert_eq!((w, h), next.size(), "signal grids must match in size");

        for y in 0..h {
            for x in 0..w {
                let neighbor = self.neighbor_average(x, y);
                let local = self.get(x, y);
                let blended = (u16::from(local) + u16::from(neighbor)) / 2;
                let decayed = blended.saturating_sub(u16::from(decay)).min(255) as u8;
                next.set(x, y, decayed);
            }
        }
    }

    pub fn clear(&mut self) {
        self.values.fill(0);
    }

    pub fn average(&self) -> f64 {
        let total: u64 = self.values.iter().map(|v| u64::from(*v)).sum();
        total as f64 / self.values.len() as f64
    }

    pub fn max(&self) -> u8 {
        self.values.iter().copied().max().unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn diffusion_spreads_local_value() {
        let mut current = SignalGrid::new(5, 5);
        let mut next = SignalGrid::new(5, 5);
        current.set(2, 2, 100);
        current.diffuse(&mut next, 0);
        assert!(next.get(2, 2) < 100, "center should lose concentration");
        assert!(next.get(1, 2) > 0, "neighbor should gain concentration");
    }

    #[test]
    fn decay_reduces_values() {
        let mut current = SignalGrid::new(3, 3);
        let mut next = SignalGrid::new(3, 3);
        current.set(1, 1, 50);
        current.diffuse(&mut next, 10);
        assert!(next.get(1, 1) < 50, "value should decay");
    }
}
