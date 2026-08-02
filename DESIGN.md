# conway-agi Visual Design System

A cyberpunk / Ruiner-inspired terminal UI for a human-in-the-loop artificial-life substrate.

## 0. Research log

- Ruiner color palette extracted from `https://www.color-hex.com/color-palette/1029396`: `#aa0000`, `#f60056`, `#770a7f`, `#490948`, `#2a00b9`.
- Ruiner UI screenshots: `https://interfaceingame.com/games/ruiner/` and `https://www.gameuidatabase.com/gameData.php?id=746`.
- Ruiner / cyberpunk fonts referenced in Steam discussion: PRESICAV, BoxedBook, VCR OSD MONO 2, Eurostile Bold Condensed / Soft Press Gothic.
- Ratatui cyberpunk/sci-fi reference projects: `ratatui-sci-fi` (Cyberpunk `#FF007F` / `#00F0FF`), `rem` (CRT phosphor profiles, scanlines, glitch lines), `metropolis` (cyberpunk skyline, neon signage), `ratatui-3d` (HalfBlock/Braille 3D renderer), `zoa` (3D ASCII/Braille styles).
- Terminal glyph references: Unicode Block Elements (`▀▄░▒▓█`), Box-drawing characters, Braille Patterns for high-resolution cell graphics.

## 1. Design intent

Make the existing Conway AGI terminal simulation feel like a polished cyberpunk HUD: high contrast, neon lineage colors, data-dense panels, faux-3D depth cues, and optional CRT/scanline atmosphere. Preserve all existing simulation behavior; only the presentation layer changes.

## 2. Visual modes

| Mode | Key | Description |
|------|-----|-------------|
| `Neon Bangkok` | `m` (cycle) | Default. Deep black background, neon genome colors, bloom halos, magenta HUD frames. |
| `Voxel City` | `v` | Faux-3D top-down view using vertical shading glyphs and cast shadows based on cell energy. |
| `CRT Terminal` | `t` | Toggleable atmosphere layer: phosphor cyan/green, scanlines, occasional glitch corruption, phosphor flicker. |

Pressing `m` cycles the primary renderer; pressing `t` toggles the CRT overlay on top of any renderer. Pressing `v` switches directly to Voxel view.

## 3. Color tokens

All colors are truecolor `ratatui::style::Color::Rgb`.

### 3.1 Neon Bangkok palette (default)

| Token | Hex | Usage |
|-------|-----|-------|
| `bg` | `#050505` | Background |
| `panel_bg` | `#0a0a10` | HUD panel fill |
| `primary` | `#f60056` | Highlights, active reward, selected mode |
| `secondary` | `#00f0ff` | Cyan accents, signal view |
| `tertiary` | `#39ff14` | Acid green, positive growth/budding hints |
| `quaternary` | `#770a7f` | Purple lineage group |
| `warning` | `#ff9e00` | Angry/high-violence cells, alert text |
| `dim` | `#3a3a50` | Inactive borders, scanlines |
| `text` | `#e0e0e0` | Status text |
| `muted` | `#7a7a9a` | Legend text, dim controls |

### 3.2 Genome lineage colors (16)

Map 16 genome tags to a continuous neon ramp anchored by the Ruiner palette:

```rust
const GENOME_PALETTE: [(u8, u8, u8); 16] = [
    (246, 0, 86),    // 0  hot pink
    (255, 158, 0),   // 1  orange
    (57, 255, 20),   // 2  acid green
    (0, 240, 255),   // 3  electric cyan
    (119, 10, 127),  // 4  purple
    (170, 0, 0),     // 5  deep red
    (73, 0, 184),    // 6  violet
    (255, 0, 128),   // 7  magenta
    (0, 255, 170),   // 8  teal
    (201, 0, 255),   // 9  electric purple
    (255, 255, 0),   // 10 acid yellow
    (0, 128, 255),   // 11 electric blue
    (255, 20, 147),  // 12 deep pink
    (0, 255, 128),   // 13 mint green
    (255, 69, 0),    // 14 red-orange
    (138, 43, 226),  // 15 blue-violet
];
```

### 3.3 CRT Terminal palette

| Token | Hex | Usage |
|-------|-----|-------|
| `crt_bg` | `#001100` | Near-black green phosphor background |
| `crt_fg` | `#00ff41` | Bright phosphor green |
| `crt_dim` | `#008f11` | Dim phosphor green |
| `crt_amber` | `#ffb000` | Amber mode accent |
| `crt_cyan` | `#00ffff` | Cyan mode accent |

## 4. Typography / glyphs

| Effect | Glyph set | Purpose |
|--------|-----------|---------|
| Energy density | `·∙░▒▓█` | Cell energy in Neon mode |
| Faux-3D height | `▁▂▃▄▅▆▇█` | Voxel City energy height |
| 3D edges / shadows | `▖▗▘▙▚▛▜▝▞▟` | Voxel cast shadows and corner shading |
| HUD frames | `┏┓┗┛┃━┣┫┳┻╋` / `╔╗╚╝║═╠╣╦╩╬` | Panel borders (heavy + double) |
| Bloom halo | `░▒▓` around alive cells | Simulated glow in Neon mode |
| High-res detail | Braille `⠁`..`⣿` | Sparklines, small radars in status panel |
| Glitch | `▚▞╳̷̛̼` | Random corruption in CRT mode |

## 5. Components

### 5.1 Theme module (`src/ui/theme.rs`)

- `Theme` struct with color tokens, glyph ramps, and border styles.
- `Theme::neon()`, `Theme::voxel()`, `Theme::crt()` constructors.
- Helper: `Theme::genome_color(index)`, `Theme::signal_color(value)`, `Theme::energy_glyph(energy)`, `Theme::voxel_glyph(energy)`, `Theme::bloom_glyph(distance)`.

### 5.2 Grid renderer (`src/ui/grid.rs`)

- `GridView` widget implementing `ratatui::widgets::Widget`.
- Renders cells or signals depending on the active `ViewMode`.
- Neon mode: alive cell = colored energy glyph + optional bloom halo on neighbors.
- Voxel mode: alive cell = vertical height glyph chosen by energy; cast shadow on lower-energy neighbor cells.

### 5.3 HUD renderer (`src/ui/hud.rs`)

- `HudPanel` widget for the status area.
- Four rows: generation/population/energy/mood, births/deaths/signal, message, controls.
- Mini sparkline for recent population or energy history (Braille).
- Neon mode: double-line magenta/cyan borders, bold title, underlined reward type.
- CRT mode: phosphor text, occasional scanline separator rows.

### 5.4 Legend overlay (`src/ui/legend.rs`)

- Floating 26-column panel over the grid.
- Neon mode: genome color swatches with black-on-neon labels; signal ramp with colored block glyphs.
- Voxel mode: height ramp legend.
- CRT mode: phosphor legend.

### 5.5 Effects overlay (`src/ui/effects.rs`)

- `Effects` widget drawn as the top layer.
- CRT: horizontal scanline rows at alternating `tick`-driven offsets; rare glitch row/column swaps; vignette via dark border cells.
- Bloom: pre-computed halo pass around high-energy cells (Neon mode only).

### 5.6 App shell (`src/tui.rs`)

- Thinned down: event loop, mode switching, theme switching, mouse poke, chat input.
- Calls `GridView`, `HudPanel`, `Legend`, `Effects` in sequence each frame.

## 6. Motion and effects rules

- Bloom and scanline offsets update every tick; no faster than the simulation tick rate.
- Glitch corruption has a low probability per frame (e.g., 2%) in CRT mode and lasts 1-3 ticks.
- Mode transitions are instant; no fade (terminal cannot alpha-blend).
- Reward pulse: when a human reward is active, the HUD border color shifts to `primary` for the reward duration.

## 7. Controls additions

| Key | Action |
|-----|--------|
| `m` | Cycle visual theme (Neon → Voxel → CRT base) |
| `v` | Switch to Voxel City view |
| `t` | Toggle CRT atmosphere overlay |
| `:` or `/` | Open chat prompt |
| `z` | Force sleep mode |
| Mouse left click | Poke cell |

## 8. Accessibility / terminal constraints

- Truecolor (24-bit) terminal required for full effect. 256-color terminals get a desaturated fallback using `Color::Indexed`.
- Some terminals render geometric glyphs poorly; Voxel mode may degrade to the energy-density ramp on terminals with limited Unicode support.
- Keep status text readable at all times; never let effects obscure the message or controls.

## 9. Accepted debt

- Faux-3D is an optical illusion via glyph choice; it is not geometric 3D.
- Glow is simulated by colored background/foreground pairs, not true alpha blur.
- CRT scanlines reduce perceived sharpness; this is intentional and togglable.
