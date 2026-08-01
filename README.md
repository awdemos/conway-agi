# conway-agi

A terminal-based Conway's Game of Life implemented in Rust, extended with simple artificial-life primitives that make it a starting point for an **AGI simulation** substrate.

## Run

```bash
cargo run --release
```

## Controls

| Key | Action |
|-----|--------|
| `q` | Quit |
| `p` / `Space` | Pause / resume |
| `r` | Reset with random seed |
| `n` | Step one generation while paused |
| `c` | Toggle cell / signal view |
| `0-9` | Reward the corresponding signal type |
| `+` / `-` | Increase / decrease speed |

## What is special about this Game of Life?

- **Conway rules remain intact**: cells with 2-3 neighbors survive; dead cells with exactly 3 live neighbors are born.
- **Energy metabolism**: every live cell pays a metabolic cost each tick. Crowded cells pay more. Well-supported cells gain energy through "photosynthesis". Cells that hit zero energy die of starvation.
- **Genome inheritance**: when a cell is born, it inherits a genome tag from its three parents. If parents disagree, the majority wins; ties are resolved randomly and mutations can occur. This creates lineages and selection pressure.
- **Active reproduction**: cells can bud into empty neighbors when they have surplus energy, independent of the strict Conway birth rule.
- **Chemical signals**: a parallel signal grid diffuses and decays. Cells emit signals based on their genome and read local signal levels.
- **Per-cell brain + human RL**: each cell has four weights that map energy, neighbors, signal, and reward to one of three actions (bud, emit, rest). Humans press `0-9` to reward signal types. Brains update in real time and are inherited with mutation.
- **Human-readable message decoder**: the dominant signal type each tick is mapped to a symbol, producing a rolling "message" in the status bar.

These additions turn the simulation from pure cellular automata into a minimal **human-in-the-loop evolutionary intelligence substrate**.

## Architecture

- `src/cell.rs` — Cell state, genome, brain weights.
- `src/grid.rs` — Toroidal grid and neighbor queries.
- `src/rules.rs` — Conway step, budding, signaling, brain action selection, reinforcement.
- `src/signal.rs` — Diffusable chemical signal grid.
- `src/reward.rs` — Human reward channel.
- `src/decoder.rs` — Signal-to-symbol message decoder.
- `src/simulation.rs` — Buffers, tick loop, aggregate statistics, decoder update.
- `src/tui.rs` — ratatui terminal interface.

## Roadmap to a real AGI simulation

See [`docs/AGI-SIMULATION.md`](docs/AGI-SIMULATION.md) for a step-by-step design for evolving this substrate toward artificial general intelligence.
