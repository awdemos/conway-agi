use crate::cell::Cell;
use crate::decoder::Decoder;
use crate::grid::Grid;
use crate::reward::RewardChannel;
use crate::rules::{StepResult, step};
use crate::signal::SignalGrid;
use rand::rngs::ThreadRng;

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
}

impl Simulation {
    pub fn new(width: usize, height: usize) -> Self {
        let current = Grid::new(width, height);
        let next = Grid::new(width, height);
        let signals = SignalGrid::new(width, height);
        let next_signals = SignalGrid::new(width, height);
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
        }
    }

    pub fn with_seed(mut self, density: f64) -> Self {
        self.current.randomize(density, &mut self.rng);
        self
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

    pub fn reset(&mut self, density: f64) {
        self.current.randomize(density, &mut self.rng);
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

    fn dominant_signal_type(&self) -> u8 {
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
