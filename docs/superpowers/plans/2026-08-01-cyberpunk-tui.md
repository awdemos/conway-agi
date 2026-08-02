# Cyberpunk/Ruiner TUI Visual Overhaul — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Transform the existing ratatui TUI into a selectable cyberpunk/Ruiner-themed HUD with Neon Bangkok (default), Voxel City, and CRT Terminal visual modes, while keeping all simulation behavior intact.

**Architecture:** Extract all visual decisions from `src/tui.rs` into a new `src/ui/` module tree (`theme`, `grid`, `hud`, `legend`, `effects`). `src/tui.rs` becomes a thin app shell that switches themes/views and routes input. Use ratatui `Color::Rgb`, Unicode block/Braille/box-drawing glyphs, and crossterm mouse events.

**Tech Stack:** Rust, ratatui 0.30.2, crossterm 0.29.0.

---

### Task 1: Create theme module

**Files:**
- Create: `src/ui/theme.rs`
- Modify: `src/ui/mod.rs`

- [ ] **Step 1: Define `Theme` struct and palette constants**

```rust
pub struct Theme {
    pub background: Color,
    pub panel_bg: Color,
    pub primary: Color,
    pub secondary: Color,
    pub tertiary: Color,
    pub text: Color,
    pub muted: Color,
    pub dim: Color,
    pub warning: Color,
    pub genome_colors: [Color; 16],
    pub signal_colors: [Color; 5],
    pub energy_glyphs: [char; 6],
    pub voxel_glyphs: [char; 8],
    pub bloom_glyphs: [char; 3],
    pub border_glyphs: BorderGlyphs,
}
```

- [ ] **Step 2: Implement `Theme::neon()`, `Theme::voxel()`, `Theme::crt()` constructors using `Color::Rgb`**

- [ ] **Step 3: Add helper methods**

```rust
impl Theme {
    pub fn genome_color(&self, index: u8) -> Color;
    pub fn signal_color(&self, value: u8) -> Color;
    pub fn energy_glyph(&self, energy: u8) -> char;
    pub fn voxel_glyph(&self, energy: u8) -> char;
    pub fn bloom_glyph(&self, distance: u8) -> char;
}
```

- [ ] **Step 4: Export from `src/ui/mod.rs`**

- [ ] **Step 5: Verify with `cargo check`**

Run: `cargo check`
Expected: no errors.

- [ ] **Step 6: Commit**

```bash
git add src/ui/theme.rs src/ui/mod.rs
git commit -m "feat(ui): add cyberpunk theme module with neon/voxel/crt palettes"
```

---

### Task 2: Create grid renderer

**Files:**
- Create: `src/ui/grid.rs`
- Modify: `src/ui/mod.rs`

- [ ] **Step 1: Define `GridView` widget and props**

```rust
pub struct GridView<'a> {
    pub sim: &'a Simulation,
    pub theme: &'a Theme,
    pub view: ViewMode,
    pub asleep: bool,
    pub effect_frame: u64,
}
```

- [ ] **Step 2: Implement `Widget` for `GridView`**

Render `cells` or `signals` depending on `view`. Use theme helpers for color/glyph lookup.

- [ ] **Step 3: Add bloom halo pass for Neon mode**

For each alive cell, write `░▒▓` into neighboring dead cells with a dimmer version of the same genome color.

- [ ] **Step 4: Add Voxel mode vertical shading + shadows**

Map energy to `▁..█`; if a cell has lower energy than a neighbor above it, cast a shadow glyph `▖`/`▘`.

- [ ] **Step 5: Verify with `cargo check`**

Run: `cargo check`
Expected: no errors.

- [ ] **Step 6: Commit**

```bash
git add src/ui/grid.rs src/ui/mod.rs
git commit -m "feat(ui): add grid renderer with neon bloom and voxel shading"
```

---

### Task 3: Create HUD and legend renderers

**Files:**
- Create: `src/ui/hud.rs`, `src/ui/legend.rs`
- Modify: `src/ui/mod.rs`

- [ ] **Step 1: Implement `HudPanel` widget**

Render four status rows: gen/pop/energy/mood, births/deaths/signal, message, controls. Use theme colors and double-line borders in Neon mode, phosphor text in CRT mode.

- [ ] **Step 2: Add Braille population sparkline**

Maintain a rolling `Vec<usize>` of last 40 population values in `App`; pass to `HudPanel`.

- [ ] **Step 3: Implement `Legend` widget**

Floating overlay with genome color swatches and signal ramp. Dim when `asleep`.

- [ ] **Step 4: Verify with `cargo check`**

Run: `cargo check`
Expected: no errors.

- [ ] **Step 5: Commit**

```bash
git add src/ui/hud.rs src/ui/legend.rs src/ui/mod.rs
git commit -m "feat(ui): add cyberpunk HUD panel and legend overlay"
```

---

### Task 4: Create effects overlay

**Files:**
- Create: `src/ui/effects.rs`
- Modify: `src/ui/mod.rs`

- [ ] **Step 1: Implement `Effects` widget**

Top-layer overlay drawn after grid. Contains:
- CRT scanlines: alternating dim horizontal rows.
- Glitch corruption: random row/column swaps with low probability.
- Vignette: dark border cells.

- [ ] **Step 2: Tie effect intensity to `effect_frame` and active theme**

Only render CRT overlay when CRT atmosphere is enabled.

- [ ] **Step 3: Verify with `cargo check`**

Run: `cargo check`
Expected: no errors.

- [ ] **Step 4: Commit**

```bash
git add src/ui/effects.rs src/ui/mod.rs
git commit -m "feat(ui): add CRT scanline, glitch, and vignette effects overlay"
```

---

### Task 5: Refactor `src/tui.rs` into app shell

**Files:**
- Modify: `src/tui.rs`
- Modify: `src/lib.rs`
- Create: `src/ui/mod.rs`

- [ ] **Step 1: Add `src/ui/mod.rs` re-exporting all UI modules**

```rust
pub mod effects;
pub mod grid;
pub mod hud;
pub mod legend;
pub mod theme;
```

- [ ] **Step 2: Add `src/ui` to `src/lib.rs`**

- [ ] **Step 3: Replace drawing code in `App::draw` with new widgets**

Use `GridView`, `HudPanel`, `Legend`, `Effects`. Remove old inline rendering helpers.

- [ ] **Step 4: Add theme/view mode state to `App`**

```rust
pub enum VisualMode { Neon, Voxel, Crt }
```

Add keys `m`, `v`, `t` to switch modes.

- [ ] **Step 5: Add population history buffer to `App`**

Push `sim.population()` each tick into a `VecDeque<usize>` capped at 80.

- [ ] **Step 6: Verify with `cargo clippy -- -D warnings && cargo test`**

Expected: clean.

- [ ] **Step 7: Commit**

```bash
git add src/tui.rs src/lib.rs src/ui/mod.rs
git commit -m "refactor(tui): switch to theme-driven ui widgets and add visual mode keys"
```

---

### Task 6: Update README controls and architecture

**Files:**
- Modify: `README.md`

- [ ] **Step 1: Add new controls (`m`, `v`, `t`) to the controls table**

- [ ] **Step 2: Update architecture section to list `src/ui/` modules**

- [ ] **Step 3: Verify with `cargo fmt -- --check`**

- [ ] **Step 4: Commit**

```bash
git add README.md
git commit -m "docs: document visual modes and new ui module layout"
```

---

### Task 7: Full verification

- [ ] **Step 1: Run formatting check**

Run: `cargo fmt -- --check`
Expected: no diff.

- [ ] **Step 2: Run clippy**

Run: `cargo clippy -- -D warnings`
Expected: clean.

- [ ] **Step 3: Run tests**

Run: `cargo test`
Expected: all tests pass.

- [ ] **Step 4: Run release build**

Run: `cargo build --release`
Expected: success.

- [ ] **Step 5: Manual TUI smoke test**

Run: `timeout 5 cargo run --release` in a tmux pty. Capture a screenshot with the web-terminal visual QA script if available.
Expected: app starts, grid visible, no panic.

- [ ] **Step 6: Commit if not already committed**

---

### Task 8: Visual QA dual-oracle review

- [ ] **Step 1: Capture TUI screenshots for each visual mode**

Use the project's web-terminal visual QA pattern for Neon, Voxel, CRT, and sleep states.

- [ ] **Step 2: Run visual-qa script**

```bash
node "$SKILL_DIR/scripts/visual-qa.mjs" tui-check <capture.txt> --cols <N>
```

- [ ] **Step 3: Dispatch Pass A / Pass B reviewer subagents**

See `visual-qa` skill prompts. Pass A checks design-system and functional integrity; Pass B checks visual fidelity and layout.

- [ ] **Step 4: Fix any blockers and re-capture**

- [ ] **Step 5: Final commit**

```bash
git commit -m "qa: visual verification passed for cyberpunk TUI overhaul"
```
