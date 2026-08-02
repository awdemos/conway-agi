use rand::{Rng, RngExt};

pub const BRAIN_WEIGHTS: usize = 4;

/// Small per-cell randomness applied to action selection.
pub const RANDOMNESS: i8 = 12;

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

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CellState {
    #[default]
    Calm,
    Anxious,
    Angry,
    Sleepy,
    Passion,
    Quietude,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Attachment {
    pub to_budding: i8,
    pub to_emitting: i8,
    pub to_resting: i8,
    pub to_crowds: i8,
    pub to_solitude: i8,
}

impl Attachment {
    pub const fn new() -> Self {
        Self {
            to_budding: 0,
            to_emitting: 0,
            to_resting: 0,
            to_crowds: 0,
            to_solitude: 0,
        }
    }
}

impl Default for Attachment {
    fn default() -> Self {
        Attachment::new()
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
    pub state: CellState,
    pub age: u16,
    pub agitation: i8,
    pub violence: i8,
    pub peace: i8,
    pub memory: u8,
    pub attachment: Attachment,
    pub name: [u8; 4],
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
            state: CellState::Calm,
            age: 0,
            agitation: 0,
            violence: 0,
            peace: 0,
            memory: 0,
            attachment: Attachment::new(),
            name: [0; 4],
        }
    }

    pub fn alive_named(genome: Genome, name: [u8; 4]) -> Self {
        Self {
            alive: true,
            energy: Self::MAX_ENERGY,
            genome,
            brain: Brain::neutral(),
            last_reward: 0,
            last_action: 0,
            state: CellState::Calm,
            age: 0,
            agitation: 0,
            violence: 0,
            peace: 0,
            memory: 0,
            attachment: Attachment::new(),
            name,
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
            state: CellState::Calm,
            age: 0,
            agitation: 0,
            violence: 0,
            peace: 0,
            memory: 0,
            attachment: Attachment::new(),
            name: [0; 4],
        }
    }

    pub fn name_string(&self) -> String {
        self.name
            .iter()
            .map(|b| char::from_u32(u32::from(*b)).unwrap_or('?'))
            .filter(|c| !c.is_control())
            .collect::<String>()
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
        if self.alive {
            self.age = self.age.saturating_add(1);
        }
        self.alive
    }

    pub fn remember(&mut self, event: u8) {
        self.memory = (self.memory << 2) | (event & 0x03);
    }

    pub fn trauma(&mut self) {
        self.remember(0b01);
        self.agitation = self.agitation.saturating_add(20);
    }

    pub fn harm(&mut self) {
        self.remember(0b10);
        self.violence = self.violence.saturating_add(16);
    }

    pub fn soothe(&mut self) {
        self.remember(0b11);
        self.peace = self.peace.saturating_add(16);
    }

    pub fn random_drift(&mut self) {
        if self.violence > 0 {
            self.violence -= 1;
        }
        if self.agitation > 0 {
            self.agitation -= 1;
        }
        if self.peace > 0 {
            self.peace -= 1;
        }
    }

    pub fn update_attachment(&mut self, action: usize, reward: i8, neighbors: usize) {
        let delta = if reward > 0 {
            4
        } else if reward < 0 {
            -4
        } else {
            1
        };
        match action {
            0 => self.attachment.to_budding = self.attachment.to_budding.saturating_add(delta),
            1 => self.attachment.to_emitting = self.attachment.to_emitting.saturating_add(delta),
            2 => self.attachment.to_resting = self.attachment.to_resting.saturating_add(delta),
            _ => {}
        }
        if neighbors >= 3 {
            self.attachment.to_crowds = self.attachment.to_crowds.saturating_add(delta);
        } else {
            self.attachment.to_solitude = self.attachment.to_solitude.saturating_add(delta);
        }
    }
}
