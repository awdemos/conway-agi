use crate::cell::{Brain, Cell, CellState, Genome, RANDOMNESS};
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
            let (neighbors, parent_genomes, parent_brains, parent_attachments) =
                current.neighbors_with_attachments(x, y);

            let mut new_cell = if cell.is_alive() {
                let next = apply_alive(cell, neighbors);
                if !next.alive {
                    deaths += 1;
                }
                next
            } else {
                try_birth(
                    neighbors,
                    &parent_genomes,
                    &parent_brains,
                    &parent_attachments,
                    rng,
                )
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
                    &new_cell,
                    rng,
                );
                new_cell.last_action = action;

                update_state(&mut new_cell, neighbors, reward, current, x, y);

                if action == 0
                    && u16::from(new_cell.energy)
                        > u16::from(Cell::BUD_COST) + u16::from(local_signal)
                    && (1..=3).contains(&neighbors)
                    && let Some((bx, by)) = find_empty_neighbor(current, x, y, rng)
                {
                    let mut bud = Cell::alive(new_cell.genome);
                    bud.energy = Cell::BUD_COST;
                    bud.brain = new_cell.brain;
                    bud.state = new_cell.state;
                    bud.name = new_cell.name;
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
                    new_cell.soothe();
                }

                if reward != 0 {
                    rewarded += 1;
                    new_cell.brain.reinforce(action, reward);
                    if reward > 0 {
                        new_cell.soothe();
                    } else {
                        new_cell.trauma();
                    }
                }
                new_cell.update_attachment(action, reward, neighbors);
                new_cell.last_reward = reward.clamp(-MAX_REWARD, MAX_REWARD);
                new_cell.random_drift();
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

#[allow(clippy::too_many_arguments)]
fn choose_action<R: Rng>(
    brain: &Brain,
    energy: u8,
    neighbors: usize,
    signal: u8,
    last_reward: i8,
    current_reward: i8,
    cell: &Cell,
    rng: &mut R,
) -> usize {
    let reward_input = if current_reward != 0 {
        current_reward
    } else {
        last_reward
    };
    let base = brain.score(energy, neighbors, signal, reward_input);
    let state_bonus: i16 = match cell.state {
        CellState::Anxious => -8,
        CellState::Sleepy => -16,
        CellState::Quietude => -4,
        CellState::Passion => 12,
        _ => 0,
    };
    let attachment_bias =
        i16::from(cell.attachment.to_budding) - i16::from(cell.attachment.to_resting);
    let scores = [
        base + state_bonus
            + attachment_bias
            + i16::from(rng.random_range(-RANDOMNESS..=RANDOMNESS)),
        base / 2
            + i16::from(cell.attachment.to_emitting)
            + i16::from(rng.random_range(-RANDOMNESS..=RANDOMNESS)),
        i16::from(rng.random_range(-RANDOMNESS..=RANDOMNESS))
            + i16::from(cell.attachment.to_resting),
    ];
    let mut best = 0;
    for (i, score) in scores.iter().enumerate().skip(1) {
        if *score > scores[best] {
            best = i;
        }
    }
    best
}

fn update_state(cell: &mut Cell, neighbors: usize, reward: i8, grid: &Grid, x: usize, y: usize) {
    let xi = x as isize;
    let yi = y as isize;
    let mut nearby_stress = 0u8;
    let mut nearby_violence = 0u8;
    let mut nearby_peace = 0u8;
    for dy in -1..=1 {
        for dx in -1..=1 {
            if dx == 0 && dy == 0 {
                continue;
            }
            let n = grid.cell_at(xi + dx, yi + dy);
            if n.is_alive() {
                nearby_stress = nearby_stress.saturating_add(n.agitation.max(n.violence) as u8);
                nearby_violence = nearby_violence.saturating_add(n.violence as u8);
                nearby_peace = nearby_peace.saturating_add(n.peace as u8);
            }
        }
    }

    if reward < 0 || neighbors > 3 || nearby_stress >= 40 || cell.energy < 50 {
        cell.trauma();
        if cell.agitation > 60 || (cell.energy < 50 && cell.age > 30) {
            cell.state = CellState::Anxious;
        }
    }

    if reward > 0 || nearby_peace >= 40 {
        cell.soothe();
        if cell.peace > cell.agitation && cell.peace > cell.violence {
            cell.state = CellState::Calm;
        }
    }

    if cell.violence > 60 && nearby_violence >= 30 {
        cell.state = CellState::Angry;
        cell.harm();
    }

    if cell.energy > 180 && cell.peace > 40 && cell.state != CellState::Anxious {
        cell.state = CellState::Sleepy;
    } else if cell.state == CellState::Sleepy && (cell.energy < 120 || cell.agitation > 30) {
        cell.state = CellState::Calm;
    }

    if reward > 0 && cell.energy > 150 && cell.attachment.to_budding > 20 {
        cell.state = CellState::Passion;
    } else if cell.state == CellState::Passion && (cell.energy < 100 || cell.agitation > 40) {
        cell.state = CellState::Calm;
    }

    if cell.peace > 50
        && cell.attachment.to_resting > 10
        && cell.attachment.to_solitude > cell.attachment.to_crowds
        && cell.state != CellState::Anxious
        && cell.state != CellState::Angry
    {
        cell.state = CellState::Quietude;
    } else if cell.state == CellState::Quietude && (cell.agitation > 30 || cell.violence > 30) {
        cell.state = CellState::Calm;
    }
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
    parent_attachments: &[crate::cell::Attachment],
    rng: &mut R,
) -> Cell {
    if neighbors == 3 {
        let mut cell = Cell::alive(inherit_genome(parents, rng));
        cell.brain = Brain::inherit(parent_brains, rng);
        cell.attachment = inherit_attachment(parent_attachments, rng);
        cell
    } else {
        Cell::dead()
    }
}

fn inherit_attachment<R: Rng>(
    parents: &[crate::cell::Attachment],
    rng: &mut R,
) -> crate::cell::Attachment {
    if parents.is_empty() {
        return crate::cell::Attachment::default();
    }
    let mut child = crate::cell::Attachment::default();
    for parent in parents {
        child.to_budding = child.to_budding.saturating_add(parent.to_budding / 3);
        child.to_emitting = child.to_emitting.saturating_add(parent.to_emitting / 3);
        child.to_resting = child.to_resting.saturating_add(parent.to_resting / 3);
        child.to_crowds = child.to_crowds.saturating_add(parent.to_crowds / 3);
        child.to_solitude = child.to_solitude.saturating_add(parent.to_solitude / 3);
    }
    if rng.random_ratio(1, 3) {
        child.to_budding = child.to_budding.saturating_add(rng.random_range(-3..=3));
        child.to_emitting = child.to_emitting.saturating_add(rng.random_range(-3..=3));
        child.to_resting = child.to_resting.saturating_add(rng.random_range(-3..=3));
        child.to_crowds = child.to_crowds.saturating_add(rng.random_range(-3..=3));
        child.to_solitude = child.to_solitude.saturating_add(rng.random_range(-3..=3));
    }
    child
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
    } else {
        next.trauma();
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
        let cell = try_birth(3, &[Genome(1), Genome(1), Genome(2)], &[], &[], &mut rng);
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
