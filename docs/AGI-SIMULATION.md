# From Conway's Game of Life to AGI: A Design Roadmap

This document explains how the `conway-agi` cellular automaton can be progressively extended into a substrate for artificial general intelligence research.

## 1. Why Game of Life is a useful substrate

Conway's Game of Life is Turing-complete, emergent, and parallel. Complex structures (gliders, guns, puffers, breeders) arise from local rules. That makes it an ideal testbed for studying how global intelligence might arise from local, simple agents.

The current implementation adds two key ingredients missing from pure Conway:

1. **Energy**: creates selection pressure. Patterns that are efficient and self-sustaining outlive random noise.
2. **Genomes**: creates heredity. Stable patterns can propagate slight variations, enabling evolution.

Together these ingredients form the minimum viable substrate for an **artificial life world**.

## 2. Current model

Each cell has:

- `alive: bool`
- `energy: u8`
- `genome: u8`

Rules per tick:

1. Apply Conway survival/birth rules.
2. Born cells inherit the majority genome of their three parents, with small mutation probability.
3. Surviving cells gain energy from an ideal neighborhood density and lose energy to crowding and metabolism.
4. Cells with enough energy can **bud** a copy into an adjacent empty cell, making reproduction active rather than passive.
5. Cells emit diffusable **chemical signals** based on their genome. Local signal intensity feeds back into the budding threshold, giving cells a crude environmental sense.
6. Cells with zero energy die regardless of Conway rules.

This produces active colonies that compete for space, emit signals, and adapt their reproduction to local conditions.

## 3. Roadmap to AGI-like behavior

The path from here to AGI-like behavior is a ladder of increasingly complex emergent capabilities. Each step adds one new mechanism and validates that new behavior appears before moving on.

### Step 1 — Self-replication ✅

Cells now actively bud into empty neighboring cells when they have surplus energy. Reproduction is no longer limited to the strict 3-neighbor Conway birth rule. This lets lineages expand, contract, and colonize territory.

### Step 2 — Sensing and signal propagation ✅

A second grid channel carries diffusable chemical signals. Live cells emit signals; signals diffuse and decay each tick. Cells read the local signal level and use it as part of their budding decision. This converts static patterns into simple sensors/actuators.

### Step 3 — Human-in-the-loop reinforcement learning ✅

Each cell now has a tiny brain: four weights that map local observations (energy, neighbor density, signal strength, recent reward) to a choice among three actions:

- **Bud** into an empty neighbor
- **Emit** a chemical signal
- **Rest** and regenerate energy

Humans press keys `0-9` to reward the corresponding signal type. Cells emitting the rewarded signal receive positive reinforcement; cells emitting a different signal receive negative reinforcement. The cell's brain weights are updated immediately. Offspring inherit mutated copies of their parents' brains, so rewarded behaviors spread through the population.

A signal decoder maps the dominant signal type to a symbol and accumulates a rolling "message" that the human can read in the status bar. This closes the loop: human reward → cellular learning → collective message.

This is not yet AGI, but it is the first genuine **human-cell communication channel** and a working reinforcement-learning substrate.

### Step 3 — Internal state machines

Give each cell a small register file (e.g., 4-8 bytes) and a simple instruction set (move, compare, jump). Cells execute a tiny program every tick. The genome now encodes a program. Selection favors programs that gather energy, avoid crowding, and reproduce.

### Step 4 — Multicellular agents

Allow adjacent cells with the same genome to share energy and communicate through local channels. The colony becomes the unit of selection, not the cell. This enables specialization: boundary cells, nutrient-gathering cells, reproductive cells.

### Step 5 — Memory and learning

Colonies with internal state can store short-term memory in their register files. Reward signals (energy surplus) reinforce states that led to reward. Simple reinforcement learning emerges from selection pressure.

### Step 6 — World model and prediction

Colonies that model their local environment (e.g., anticipate where food signals will appear) outperform reactive ones. This requires a compact neural network or associative memory encoded in the genome.

### Step 7 — Generalization

Once colonies can learn from reward, expose them to a variety of generated micro-environments. Select for genomes whose learning algorithm generalizes across environments. This is the transition from narrow to general intelligence.

## 4. Implementation guidelines

- Keep every rule local. Global coordination should be emergent, not hard-coded.
- Run many parallel simulations and select winners by survival time + reproduction rate.
- Log lineage trees, genome diversity, energy flows, and colony structures.
- Use visualization to understand what is evolving. The unexpected structures are the signal.

## 5. Relation to AGI

AGI is not a single algorithm; it is the capacity to solve novel problems in varied environments. A cellular substrate like this one is not AGI itself, but it is a way to grow AGI from the bottom up:

- Evolution provides the search algorithm.
- Energy constraints provide the objective.
- Genomes provide the hypothesis space.
- The grid provides the world.

The goal is not to predict exactly what will evolve, but to build conditions under which increasingly general problem-solvers are naturally selected.
