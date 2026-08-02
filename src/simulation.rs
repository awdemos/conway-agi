use crate::avatar::Avatar;
use crate::cell::Cell;
use crate::decoder::Decoder;
use crate::grid::Grid;
use crate::reward::RewardChannel;
use crate::rules::{StepResult, step};
use crate::signal::SignalGrid;
use rand::RngExt;
use rand::rngs::ThreadRng;
use std::collections::HashMap;

pub struct Simulation {
    current: Grid,
    next: Grid,
    signals: SignalGrid,
    next_signals: SignalGrid,
    rewards: RewardChannel,
    decoder: Decoder,
    generation: u64,
    rng: ThreadRng,
    last_result: StepResult,
    avatar: Avatar,
    transcripts: HashMap<(usize, usize), Vec<(String, String)>>,
}

impl Simulation {
    pub fn new(width: usize, height: usize) -> Self {
        let current = Grid::new(width, height);
        let next = Grid::new(width, height);
        let signals = SignalGrid::new(width, height);
        let next_signals = SignalGrid::new(width, height);
        let avatar = Avatar::new(width / 2, height / 2);
        Self {
            current,
            next,
            signals,
            next_signals,
            rewards: RewardChannel::default(),
            decoder: Decoder::default(),
            generation: 0,
            rng: rand::rng(),
            last_result: StepResult {
                births: 0,
                deaths: 0,
                emissions: 0,
                rewarded: 0,
            },
            avatar,
            transcripts: HashMap::new(),
        }
    }

    pub fn with_seed(mut self, density: f64) -> Self {
        self.current.randomize(density, &mut self.rng);
        self.name_live_cells();
        self
    }

    pub fn avatar(&self) -> &Avatar {
        &self.avatar
    }

    pub fn move_avatar(&mut self, dx: i32, dy: i32) {
        self.avatar
            .move_by(dx, dy, self.current.width(), self.current.height());
    }

    pub fn query_cell(&self, x: usize, y: usize) -> Option<Cell> {
        let (w, h) = self.current.size();
        if x >= w || y >= h {
            return None;
        }
        let cell = self.current.get(x, y);
        if cell.is_alive() { Some(cell) } else { None }
    }

    pub fn cell_transcript(&self, x: usize, y: usize) -> &[(String, String)] {
        self.transcripts.get(&(x, y)).map_or(&[], |v| v.as_slice())
    }

    pub fn record_exchange(&mut self, x: usize, y: usize, player: String, cell: String) {
        self.transcripts
            .entry((x, y))
            .or_default()
            .push((player, cell));
    }

    fn name_live_cells(&mut self) {
        let mut positions = Vec::new();
        for (x, y, cell) in self.current.iter() {
            if cell.is_alive() && cell.name.iter().all(|b| *b == 0) {
                positions.push((x, y));
            }
        }
        let mut names = Vec::with_capacity(positions.len());
        for _ in 0..positions.len() {
            names.push(self.random_name());
        }
        for ((x, y), name) in positions.into_iter().zip(names) {
            if let Some(cell) = self.current.get_mut(x, y) {
                cell.name = name;
            }
        }
    }

    fn random_name(&mut self) -> [u8; 4] {
        let consonants = b"bcdfghjklmnpqrstvwxz";
        let vowels = b"aeiouy";
        let mut name = [0u8; 4];
        for (i, slot) in name.iter_mut().enumerate() {
            *slot = if i % 2 == 0 {
                consonants[self.rng.random_range(0..consonants.len())]
            } else {
                vowels[self.rng.random_range(0..vowels.len())]
            };
        }
        name
    }

    pub fn grid(&self) -> &Grid {
        &self.current
    }

    pub fn signals(&self) -> &SignalGrid {
        &self.signals
    }

    pub fn generation(&self) -> u64 {
        self.generation
    }

    pub fn tick(&mut self) {
        let reward_signal = self.rewards.take();
        let result = step(
            &self.current,
            &mut self.next,
            &self.signals,
            &mut self.next_signals,
            reward_signal,
            &mut self.rng,
        );
        std::mem::swap(&mut self.current, &mut self.next);
        std::mem::swap(&mut self.signals, &mut self.next_signals);
        self.generation += 1;
        self.last_result = result;

        if let Some(sig) = reward_signal {
            self.decoder.push_signal(sig);
        } else if self.generation.is_multiple_of(10) {
            let dominant = self.dominant_signal_type();
            self.decoder.push_signal(dominant);
        }
    }

    pub fn reward(&mut self, signal_type: u8) {
        self.rewards.trigger(signal_type);
    }

    pub fn message(&self) -> String {
        self.decoder.message()
    }

    pub fn poke(&mut self, x: usize, y: usize) {
        let w = self.current.width();
        let h = self.current.height();
        if x >= w || y >= h {
            return;
        }
        let mut cell = self.current.get(x, y);
        if cell.is_alive() {
            cell.absorb(80);
            cell.soothe();
            cell.agitation = 0;
            cell.violence = 0;
            self.signals.add(x, y, 64);
        } else {
            cell = Cell::alive(crate::cell::Genome::WILD);
            cell.energy = 120;
            self.signals.add(x, y, 96);
        }
        self.current.set(x, y, cell);
    }

    pub fn dominant_state(&self) -> (crate::cell::CellState, usize) {
        let mut counts = [0usize; 6];
        for (_, _, cell) in self.current.iter() {
            if cell.is_alive() {
                counts[cell.state as usize] += 1;
            }
        }
        let mut best = crate::cell::CellState::Calm;
        let mut best_count = counts[best as usize];
        for (idx, count) in counts.iter().enumerate() {
            if *count > best_count {
                best_count = *count;
                best = match idx {
                    0 => crate::cell::CellState::Calm,
                    1 => crate::cell::CellState::Anxious,
                    2 => crate::cell::CellState::Angry,
                    3 => crate::cell::CellState::Sleepy,
                    4 => crate::cell::CellState::Passion,
                    5 => crate::cell::CellState::Quietude,
                    _ => crate::cell::CellState::Calm,
                };
            }
        }
        (best, best_count)
    }

    pub fn attachment_summary(&self) -> crate::cell::Attachment {
        let mut total = crate::cell::Attachment::new();
        let mut count = 0usize;
        for (_, _, cell) in self.current.iter() {
            if cell.is_alive() {
                total.to_budding = total.to_budding.saturating_add(cell.attachment.to_budding);
                total.to_emitting = total
                    .to_emitting
                    .saturating_add(cell.attachment.to_emitting);
                total.to_resting = total.to_resting.saturating_add(cell.attachment.to_resting);
                total.to_crowds = total.to_crowds.saturating_add(cell.attachment.to_crowds);
                total.to_solitude = total
                    .to_solitude
                    .saturating_add(cell.attachment.to_solitude);
                count += 1;
            }
        }
        if count > 0 {
            total.to_budding /= count as i8;
            total.to_emitting /= count as i8;
            total.to_resting /= count as i8;
            total.to_crowds /= count as i8;
            total.to_solitude /= count as i8;
        }
        total
    }

    pub fn reset(&mut self, density: f64) {
        self.current.randomize(density, &mut self.rng);
        self.name_live_cells();
        self.signals.clear();
        self.next_signals.clear();
        self.rewards = RewardChannel::default();
        self.decoder.clear();
        self.generation = 0;
        self.last_result = StepResult {
            births: 0,
            deaths: 0,
            emissions: 0,
            rewarded: 0,
        };
        self.avatar = Avatar::new(self.current.width() / 2, self.current.height() / 2);
        self.transcripts.clear();
    }

    pub fn population(&self) -> usize {
        self.current.live_cells()
    }

    pub fn average_energy(&self) -> f64 {
        let mut total = 0u64;
        let mut count = 0usize;
        for (_, _, cell) in self.current.iter() {
            if cell.is_alive() {
                total += u64::from(cell.energy);
                count += 1;
            }
        }
        if count == 0 {
            return 0.0;
        }
        (total as f64 / count as f64) / f64::from(Cell::MAX_ENERGY) * 100.0
    }

    pub fn genome_diversity(&self) -> usize {
        let mut seen = [false; 256];
        let mut distinct = 0;
        for (_, _, cell) in self.current.iter() {
            if cell.is_alive() {
                let idx = cell.genome.0 as usize;
                if !seen[idx] {
                    seen[idx] = true;
                    distinct += 1;
                }
            }
        }
        distinct
    }

    pub fn signal_stats(&self) -> (f64, u8) {
        (self.signals.average(), self.signals.max())
    }

    pub fn step_stats(&self) -> &StepResult {
        &self.last_result
    }

    pub fn reward_progress(&self) -> (Option<u8>, f64) {
        self.rewards.progress()
    }

    pub fn dominant_signal_type(&self) -> u8 {
        let mut counts = [0usize; 16];
        for (_, _, cell) in self.current.iter() {
            if cell.is_alive() {
                counts[cell.genome.signal_type() as usize] += 1;
            }
        }
        counts
            .iter()
            .enumerate()
            .max_by_key(|(_, c)| *c)
            .map(|(i, _)| i as u8)
            .unwrap_or(0)
    }
}
