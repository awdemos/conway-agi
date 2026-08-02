use crate::cell::{Brain, Cell, Genome};
use rand::RngExt;

/// A fixed-size, toroidal grid of cells.
#[derive(Clone, Debug)]
pub struct Grid {
    width: usize,
    height: usize,
    cells: Vec<Cell>,
}

impl Grid {
    pub fn new(width: usize, height: usize) -> Self {
        assert!(width > 0 && height > 0, "grid dimensions must be positive");
        Self {
            width,
            height,
            cells: vec![Cell::dead(); width * height],
        }
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn size(&self) -> (usize, usize) {
        (self.width, self.height)
    }

    fn index(&self, x: usize, y: usize) -> usize {
        y * self.width + x
    }

    pub fn get(&self, x: usize, y: usize) -> Cell {
        self.cells[self.index(x, y)]
    }

    pub fn set(&mut self, x: usize, y: usize, cell: Cell) {
        let idx = self.index(x, y);
        self.cells[idx] = cell;
    }

    pub fn get_mut(&mut self, x: usize, y: usize) -> Option<&mut Cell> {
        let idx = self.index(x, y);
        self.cells.get_mut(idx)
    }

    pub fn cell_at(&self, x: isize, y: isize) -> Cell {
        let x = x.rem_euclid(self.width as isize) as usize;
        let y = y.rem_euclid(self.height as isize) as usize;
        self.get(x, y)
    }

    pub fn neighbors(&self, x: usize, y: usize) -> (usize, Vec<Genome>, Vec<Brain>) {
        let xi = x as isize;
        let yi = y as isize;
        let mut count = 0;
        let mut genomes = Vec::with_capacity(8);
        let mut brains = Vec::with_capacity(8);
        for dy in -1..=1 {
            for dx in -1..=1 {
                if dx == 0 && dy == 0 {
                    continue;
                }
                let n = self.cell_at(xi + dx, yi + dy);
                if n.is_alive() {
                    count += 1;
                    genomes.push(n.genome);
                    brains.push(n.brain);
                }
            }
        }
        (count, genomes, brains)
    }

    pub fn neighbors_with_attachments(
        &self,
        x: usize,
        y: usize,
    ) -> (usize, Vec<Genome>, Vec<Brain>, Vec<crate::cell::Attachment>) {
        let xi = x as isize;
        let yi = y as isize;
        let mut count = 0;
        let mut genomes = Vec::with_capacity(8);
        let mut brains = Vec::with_capacity(8);
        let mut attachments = Vec::with_capacity(8);
        for dy in -1..=1 {
            for dx in -1..=1 {
                if dx == 0 && dy == 0 {
                    continue;
                }
                let n = self.cell_at(xi + dx, yi + dy);
                if n.is_alive() {
                    count += 1;
                    genomes.push(n.genome);
                    brains.push(n.brain);
                    attachments.push(n.attachment);
                }
            }
        }
        (count, genomes, brains, attachments)
    }

    pub fn live_cells(&self) -> usize {
        self.cells.iter().filter(|c| c.is_alive()).count()
    }

    pub fn iter(&self) -> impl Iterator<Item = (usize, usize, Cell)> + use<'_> {
        self.cells.iter().enumerate().map(|(idx, cell)| {
            let x = idx % self.width;
            let y = idx / self.width;
            (x, y, *cell)
        })
    }

    pub fn randomize<R: rand::Rng>(&mut self, density: f64, rng: &mut R) {
        for y in 0..self.height {
            for x in 0..self.width {
                if rng.random_bool(density) {
                    self.set(x, y, Cell::alive(Genome::WILD));
                } else {
                    self.set(x, y, Cell::dead());
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    #[test]
    fn toroidal_wrapping() {
        let mut grid = Grid::new(5, 5);
        grid.set(0, 0, Cell::alive(Genome(1)));
        let (count, genomes, _) = grid.neighbors(4, 4);
        assert_eq!(count, 1);
        assert_eq!(genomes, vec![Genome(1)]);
    }

    #[test]
    fn neighbor_count_block() {
        let mut grid = Grid::new(4, 4);
        // 2x2 block at (1,1).
        for y in 1..=2 {
            for x in 1..=2 {
                grid.set(x, y, Cell::alive(Genome::WILD));
            }
        }
        assert_eq!(grid.neighbors(1, 1).0, 3);
        assert_eq!(grid.neighbors(0, 0).0, 1);
    }

    #[test]
    fn neighbor_brains_collected() {
        let mut grid = Grid::new(4, 4);
        let mut cell = Cell::alive(Genome::WILD);
        cell.brain.weights = [1, 2, 3, 4];
        grid.set(1, 1, cell);
        let (_, _, brains) = grid.neighbors(0, 0);
        assert_eq!(brains.len(), 1);
        assert_eq!(brains[0].weights, [1, 2, 3, 4]);
    }

    #[test]
    fn randomize_density() {
        let mut grid = Grid::new(20, 20);
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        grid.randomize(0.5, &mut rng);
        let live = grid.live_cells();
        assert!(live > 50 && live < 250, "unexpected live count: {live}");
    }
}
