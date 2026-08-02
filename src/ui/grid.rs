use crate::cell::CellState;
use crate::simulation::Simulation;
use crate::ui::theme::Theme;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::widgets::Widget;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ViewMode {
    #[default]
    Cells,
    Signals,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum VisualMode {
    #[default]
    Neon,
    Voxel,
    Crt,
}

pub struct GridView<'a> {
    pub sim: &'a Simulation,
    pub theme: &'a Theme,
    pub mode: VisualMode,
    pub view: ViewMode,
    pub asleep: bool,
    pub frame: u64,
}

impl<'a> Widget for GridView<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let (w, h) = self.sim.grid().size();
        let visible_w = area.width as usize;
        let visible_h = area.height as usize;
        if visible_w == 0 || visible_h == 0 {
            return;
        }

        for y in 0..visible_h.min(h) {
            for x in 0..visible_w.min(w) {
                let cell = self.sim.grid().get(x, y);
                let (ch, style) = match self.view {
                    ViewMode::Cells => self.render_cell(cell),
                    ViewMode::Signals => self.render_signal(self.sim.signals().get(x, y)),
                };
                if let Some(c) = buf.cell_mut((area.x + x as u16, area.y + y as u16)) {
                    c.set_symbol(&ch.to_string());
                    c.set_style(style);
                }
            }
        }

        if self.mode == VisualMode::Neon && !self.asleep && self.view == ViewMode::Cells {
            self.render_bloom(area, buf, w, h, visible_w, visible_h);
        }

        if self.mode == VisualMode::Voxel && !self.asleep && self.view == ViewMode::Cells {
            self.render_voxel_shadows(area, buf, w, h, visible_w, visible_h);
        }

        let (ax, ay) = self.sim.avatar().position();
        if ax < visible_w && ay < visible_h {
            let avatar_style = Style::default()
                .fg(self.theme.tertiary)
                .bg(self.theme.background)
                .add_modifier(ratatui::style::Modifier::BOLD);
            if let Some(c) = buf.cell_mut((area.x + ax as u16, area.y + ay as u16)) {
                c.set_symbol("@");
                c.set_style(avatar_style);
            }
        }
    }
}

impl<'a> GridView<'a> {
    fn render_cell(&self, cell: crate::cell::Cell) -> (char, Style) {
        if !cell.is_alive() {
            return (' ', Style::default().bg(self.theme.background));
        }
        if self.asleep {
            return (
                '·',
                Style::default()
                    .fg(self.theme.dim)
                    .bg(self.theme.background),
            );
        }

        let base = self.theme.genome_color(cell.genome.0);
        let (fg, bg) = match cell.state {
            CellState::Angry => (self.theme.warning, self.theme.background),
            CellState::Anxious => (self.theme.primary, self.theme.background),
            CellState::Sleepy => (self.theme.secondary, self.theme.background),
            CellState::Passion => (self.theme.tertiary, self.theme.background),
            CellState::Quietude => (Color::Rgb(180, 180, 255), self.theme.background),
            CellState::Calm => (base, self.theme.background),
        };

        let symbol = if self.mode == VisualMode::Voxel {
            self.theme.voxel_glyph(cell.energy)
        } else {
            self.theme.energy_glyph(cell.energy)
        };
        (
            symbol,
            Style::default()
                .fg(fg)
                .bg(bg)
                .add_modifier(ratatui::style::Modifier::BOLD),
        )
    }

    fn render_signal(&self, value: u8) -> (char, Style) {
        if value == 0 {
            return (' ', Style::default().bg(self.theme.background));
        }
        if self.asleep {
            return (
                '·',
                Style::default()
                    .fg(self.theme.dim)
                    .bg(self.theme.background),
            );
        }
        let color = self.theme.signal_color(value);
        let symbol = self.theme.energy_glyph(value);
        (symbol, Style::default().fg(color).bg(self.theme.background))
    }

    fn render_bloom(
        &self,
        area: Rect,
        buf: &mut Buffer,
        w: usize,
        h: usize,
        visible_w: usize,
        visible_h: usize,
    ) {
        for cy in 0..visible_h.min(h) {
            for cx in 0..visible_w.min(w) {
                let cell = self.sim.grid().get(cx, cy);
                if !cell.is_alive() {
                    continue;
                }
                let color = self.theme.genome_color(cell.genome.0);
                let neighbors = [
                    (cx.saturating_sub(1), cy),
                    (cx.saturating_add(1), cy.min(w - 1)),
                    (cx, cy.saturating_sub(1)),
                    (cx, cy.min(h - 1).saturating_add(1)),
                ];
                for (nx, ny) in neighbors {
                    if nx >= visible_w || ny >= visible_h {
                        continue;
                    }
                    let ncell = self.sim.grid().get(nx, ny);
                    if ncell.is_alive() {
                        continue;
                    }
                    let dx = if nx == cx { 0 } else { 1 };
                    let dy = if ny == cy { 0 } else { 1 };
                    let dist = dx + dy;
                    let glyph = self.theme.bloom_glyph(dist);
                    let style = Style::default()
                        .fg(dim(color, dist))
                        .bg(self.theme.background);
                    if let Some(c) = buf.cell_mut((area.x + nx as u16, area.y + ny as u16))
                        && c.symbol() == " "
                    {
                        c.set_symbol(&glyph.to_string());
                        c.set_style(style);
                    }
                }
            }
        }
    }

    fn render_voxel_shadows(
        &self,
        area: Rect,
        buf: &mut Buffer,
        w: usize,
        h: usize,
        visible_w: usize,
        visible_h: usize,
    ) {
        for cy in 0..visible_h.min(h) {
            for cx in 0..visible_w.min(w) {
                let cell = self.sim.grid().get(cx, cy);
                if !cell.is_alive() {
                    continue;
                }
                let below_y = cy.saturating_add(1);
                if below_y >= visible_h || below_y >= h {
                    continue;
                }
                let below = self.sim.grid().get(cx, below_y);
                let shadow_symbol = if below.is_alive() && below.energy < cell.energy {
                    '▖'
                } else if !below.is_alive() {
                    '▘'
                } else {
                    continue;
                };
                let color = dim(self.theme.genome_color(cell.genome.0), 1);
                if let Some(c) = buf.cell_mut((area.x + cx as u16, area.y + below_y as u16))
                    && !self.sim.grid().get(cx, below_y).is_alive()
                {
                    c.set_symbol(&shadow_symbol.to_string());
                    c.set_style(Style::default().fg(color).bg(self.theme.background));
                }
            }
        }
    }
}

fn dim(color: Color, distance: u8) -> Color {
    let (r, g, b) = match color {
        Color::Rgb(r, g, b) => (r, g, b),
        _ => return color,
    };
    let factor = match distance {
        0 => 55,
        1 => 35,
        _ => 20,
    };
    Color::Rgb(
        (r as u16 * factor / 100) as u8,
        (g as u16 * factor / 100) as u8,
        (b as u16 * factor / 100) as u8,
    )
}
