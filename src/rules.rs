use crate::cell::{Brain, Cell, Genome};
use crate::grid::Grid;
use crate::signal::SignalGrid;
use rand::{Rng, RngExt};

const BASE_METABOLISM: u8 = 1;
const CROWDING_PENALTY: u8 = 3;
const PHOTOSYNTHESIS: u8 = 8;
const SIGNAL_DECAY: u8 = 6;
const SIGNAL_EMISSION: u8 = 32;
const MAX_REWARD: i8 = 40;

pub struct StepResult {
    pub births: usize,
    pub deaths: usize,
    pub emissions: usize,
    pub rewarded: usize,
}

pub fn step<R: RngExt>(
    current: &Grid,
    next: &mut Grid,
    signals: &SignalGrid,
    next_signals: &mut SignalGrid,
    reward_signal: Option<u8>,
    rng: &mut R,
) -> StepResult {
    let (w, h) = current.size();
    assert_eq!((w, h), next.size(), "grids must match in size");
    assert_eq!(
        (w, h),
        next_signals.size(),
        "signal grids must match in size"
    );

    signals.diffuse(next_signals, SIGNAL_DECAY);

    let mut births = 0;
    let mut deaths = 0;
    let mut emissions = 0;
    let mut rewarded = 0;

    for y in 0..h {
        for x in 0..w {
            let cell = current.get(x, y);
            let (neighbors, parent_genomes, parent_brains) = current.neighbors(x, y);

            let mut new_cell = if cell.is_alive() {
                let next = apply_alive(cell, neighbors);
                if !next.alive {
                    deaths += 1;
                }
                next
            } else {
                try_birth(neighbors, &parent_genomes, &parent_brains, rng)
            };

            if new_cell.is_alive() {
                let local_signal = next_signals.get(x, y);
                let reward = reward_for(new_cell.genome.signal_type(), reward_signal);

                let action = choose_action(
                    &new_cell.brain,
                    new_cell.energy,
                    neighbors,
                    local_signal,
                    new_cell.last_reward,
                    reward,
                    rng,
                );
                new_cell.last_action = action;

                if action == 0
                    && u16::from(new_cell.energy)
                        > u16::from(Cell::BUD_COST) + u16::from(local_signal)
                    && (1..=3).contains(&neighbors)
                    && let Some((bx, by)) = find_empty_neighbor(current, x, y, rng)
                {
                    let mut bud = Cell::alive(new_cell.genome);
                    bud.energy = Cell::BUD_COST;
                    bud.brain = new_cell.brain;
                    next.set(bx, by, bud);
                    new_cell.energy -= Cell::BUD_COST;
                    births += 1;
                }

                if action == 1
                    && u16::from(new_cell.energy) > u16::from(Cell::EMIT_COST) * 4
                    && local_signal < 80
                {
                    let amount = SIGNAL_EMISSION.min(u8::MAX - next_signals.get(x, y));
                    next_signals.add(x, y, amount);
                    new_cell.energy -= Cell::EMIT_COST;
                    emissions += 1;
                }

                if action == 2 {
                    new_cell.energy = new_cell.energy.saturating_add(PHOTOSYNTHESIS);
                }

                if reward != 0 {
                    rewarded += 1;
                    new_cell.brain.reinforce(action, reward);
                }
                new_cell.last_reward = reward.clamp(-MAX_REWARD, MAX_REWARD);
            }

            next.set(x, y, new_cell);
        }
    }

    StepResult {
        births,
        deaths,
        emissions,
        rewarded,
    }
}

fn reward_for(cell_signal: u8, reward_signal: Option<u8>) -> i8 {
    match reward_signal {
        Some(target) if cell_signal == target % 16 => 40,
        Some(_) => -10,
        None => 0,
    }
}

fn choose_action<R: Rng>(
    brain: &Brain,
    energy: u8,
    neighbors: usize,
    signal: u8,
    last_reward: i8,
    current_reward: i8,
    rng: &mut R,
) -> usize {
    let reward_input = if current_reward != 0 {
        current_reward
    } else {
        last_reward
    };
    let base = brain.score(energy, neighbors, signal, reward_input);
    let scores = [
        base + rng.random_range(-4..=4),
        base / 2 + rng.random_range(-4..=4),
        rng.random_range(-4..=4),
    ];
    let mut best = 0;
    for (i, score) in scores.iter().enumerate().skip(1) {
        if *score > scores[best] {
            best = i;
        }
    }
    best
}

fn find_empty_neighbor<R: Rng>(
    grid: &Grid,
    x: usize,
    y: usize,
    rng: &mut R,
) -> Option<(usize, usize)> {
    let xi = x as isize;
    let yi = y as isize;
    let w = grid.width() as isize;
    let h = grid.height() as isize;
    let mut empties = [(0usize, 0usize); 8];
    let mut count = 0;
    for dy in -1..=1 {
        for dx in -1..=1 {
            if dx == 0 && dy == 0 {
                continue;
            }
            let nx = (xi + dx).rem_euclid(w) as usize;
            let ny = (yi + dy).rem_euclid(h) as usize;
            if !grid.get(nx, ny).is_alive() {
                empties[count] = (nx, ny);
                count += 1;
            }
        }
    }
    if count == 0 {
        return None;
    }
    Some(empties[rng.random_range(0..count)])
}

fn try_birth<R: Rng>(
    neighbors: usize,
    parents: &[Genome],
    parent_brains: &[Brain],
    rng: &mut R,
) -> Cell {
    if neighbors == 3 {
        let mut cell = Cell::alive(inherit_genome(parents, rng));
        cell.brain = Brain::inherit(parent_brains, rng);
        cell
    } else {
        Cell::dead()
    }
}

fn apply_alive(cell: Cell, neighbors: usize) -> Cell {
    let survives = neighbors == 2 || neighbors == 3;
    let mut next = cell;
    next.alive = survives;

    if survives {
        let cost = if neighbors <= 3 {
            BASE_METABOLISM
        } else {
            BASE_METABOLISM + CROWDING_PENALTY
        };
        let gain = if neighbors == 2 || neighbors == 3 {
            PHOTOSYNTHESIS
        } else {
            0
        };
        next.absorb(gain);
        let still_alive = next.metabolize(cost);
        next.alive = still_alive;
    }

    next
}

fn inherit_genome<R: Rng>(parents: &[Genome], rng: &mut R) -> Genome {
    if parents.is_empty() {
        return Genome::WILD;
    }

    let mut counts = [0u8; 256];
    for g in parents {
        counts[g.0 as usize] = counts[g.0 as usize].saturating_add(1);
    }

    let max = counts.iter().copied().max().unwrap_or(0);
    let winners: Vec<u8> = counts
        .iter()
        .enumerate()
        .filter(|(_, c)| **c == max)
        .map(|(i, _)| i as u8)
        .collect();

    let chosen = winners[rng.random_range(0..winners.len())];
    Genome(chosen).mutate(rng)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    #[test]
    fn birth_requires_three_neighbors() {
        let mut rng = ChaCha8Rng::seed_from_u64(1);
        let cell = try_birth(3, &[Genome(1), Genome(1), Genome(2)], &[], &mut rng);
        assert!(cell.is_alive());
    }

    #[test]
    fn underpopulation_kills() {
        let cell = Cell::alive(Genome(5));
        let next = apply_alive(cell, 1);
        assert!(!next.is_alive());
    }

    #[test]
    fn overpopulation_kills() {
        let cell = Cell::alive(Genome(5));
        let next = apply_alive(cell, 4);
        assert!(!next.is_alive());
    }

    #[test]
    fn stable_block_survives_one_step() {
        let mut current = Grid::new(4, 4);
        let mut next = Grid::new(4, 4);
        let signals = SignalGrid::new(4, 4);
        let mut next_signals = SignalGrid::new(4, 4);
        for y in 1..=2 {
            for x in 1..=2 {
                current.set(x, y, Cell::alive(Genome::WILD));
            }
        }
        let mut rng = ChaCha8Rng::seed_from_u64(7);
        step(
            &current,
            &mut next,
            &signals,
            &mut next_signals,
            None,
            &mut rng,
        );
        for y in 1..=2 {
            for x in 1..=2 {
                assert!(next.get(x, y).is_alive(), "block cell ({x},{y}) died");
            }
        }
    }

    #[test]
    fn budding_fills_empty_neighbor() {
        let mut current = Grid::new(6, 6);
        let mut next = Grid::new(6, 6);
        let signals = SignalGrid::new(6, 6);
        let mut next_signals = SignalGrid::new(6, 6);
        for y in 2..=3 {
            for x in 2..=3 {
                let mut cell = Cell::alive(Genome(7));
                cell.energy = 200;
                cell.brain.weights = [127, 0, 0, 0];
                current.set(x, y, cell);
            }
        }
        let mut rng = ChaCha8Rng::seed_from_u64(9);
        let result = step(
            &current,
            &mut next,
            &signals,
            &mut next_signals,
            None,
            &mut rng,
        );
        assert!(result.births > 0, "colony with high energy should bud");
    }

    #[test]
    fn reward_targets_matching_signal() {
        assert_eq!(reward_for(3, Some(3)), 40);
        assert_eq!(reward_for(3, Some(5)), -10);
        assert_eq!(reward_for(3, None), 0);
    }
}
