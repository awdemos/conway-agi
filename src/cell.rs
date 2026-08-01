use rand::{Rng, RngExt};

pub const BRAIN_WEIGHTS: usize = 4;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Genome(pub u8);

impl Genome {
    pub const WILD: Genome = Genome(0);

    pub fn mutate<R: Rng>(self, rng: &mut R) -> Self {
        if rng.random_ratio(1, 8) {
            let bit = rng.random_range(0..4);
            Genome(self.0 ^ (1 << bit))
        } else {
            self
        }
    }

    pub fn signal_type(&self) -> u8 {
        self.0 & 0x0f
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Brain {
    pub weights: [i8; BRAIN_WEIGHTS],
}

impl Default for Brain {
    fn default() -> Self {
        Brain::neutral()
    }
}

impl Brain {
    pub const fn neutral() -> Self {
        Self {
            weights: [0; BRAIN_WEIGHTS],
        }
    }

    pub fn score(&self, energy: u8, neighbors: usize, signal: u8, recent_reward: i8) -> i16 {
        let energy_term = i16::from(self.weights[0]) * i16::from(energy) / 32;
        let neighbor_term = i16::from(self.weights[1]) * i16::from(neighbors as u8) * 4;
        let signal_term = i16::from(self.weights[2]) * i16::from(signal) / 16;
        let reward_term = i16::from(self.weights[3]) * i16::from(recent_reward) * 8;
        energy_term + neighbor_term + signal_term + reward_term
    }

    pub fn reinforce(&mut self, action_index: usize, reward_delta: i8) {
        let idx = action_index % self.weights.len();
        let update = reward_delta.clamp(-32, 32);
        self.weights[idx] = self.weights[idx].saturating_add(update).clamp(-127, 127);
    }

    pub fn inherit<R: Rng>(parents: &[Brain], rng: &mut R) -> Self {
        if parents.is_empty() {
            return Brain::neutral();
        }
        let mut child = Brain::neutral();
        for i in 0..BRAIN_WEIGHTS {
            let parent = &parents[rng.random_range(0..parents.len())];
            child.weights[i] = parent.weights[i];
            if rng.random_ratio(1, 4) {
                child.weights[i] += rng.random_range(-4..=4);
            }
        }
        child
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Cell {
    pub alive: bool,
    pub energy: u8,
    pub genome: Genome,
    pub brain: Brain,
    pub last_reward: i8,
    pub last_action: usize,
}

impl Default for Cell {
    fn default() -> Self {
        Cell::dead()
    }
}

impl Cell {
    pub const MAX_ENERGY: u8 = 255;
    pub const EMIT_COST: u8 = 12;
    pub const BUD_COST: u8 = 48;

    pub const fn dead() -> Self {
        Self {
            alive: false,
            energy: 0,
            genome: Genome::WILD,
            brain: Brain::neutral(),
            last_reward: 0,
            last_action: 0,
        }
    }

    pub const fn alive(genome: Genome) -> Self {
        Self {
            alive: true,
            energy: Self::MAX_ENERGY,
            genome,
            brain: Brain::neutral(),
            last_reward: 0,
            last_action: 0,
        }
    }

    pub const fn is_alive(self) -> bool {
        self.alive
    }

    pub fn absorb(&mut self, amount: u8) {
        self.energy = self.energy.saturating_add(amount);
    }

    pub fn metabolize(&mut self, cost: u8) -> bool {
        self.energy = self.energy.saturating_sub(cost);
        self.alive = self.alive && self.energy > 0;
        self.alive
    }
}
