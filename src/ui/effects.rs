use crate::ui::theme::Theme;
use rand::RngExt;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::widgets::Widget;

pub struct Effects {
    pub theme: Theme,
    pub frame: u64,
    pub enabled: bool,
    pub rng_seed: u64,
}

impl Widget for Effects {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if !self.enabled {
            return;
        }

        self.render_scanlines(area, buf);
        self.render_vignette(area, buf);
        self.render_glitch(area, buf);
    }
}

impl Effects {
    fn render_scanlines(&self, area: Rect, buf: &mut Buffer) {
        let scan_offset = (self.frame % 4) as u16;
        for dy in (scan_offset..area.height).step_by(4) {
            let y = area.y + dy;
            for dx in 0..area.width {
                let x = area.x + dx;
                if let Some(cell) = buf.cell_mut((x, y)) {
                    let style = cell.style();
                    let mut fg = style.fg.unwrap_or(self.theme.text);
                    fg = dim_color(fg, 30);
                    cell.set_style(
                        Style::default()
                            .fg(fg)
                            .bg(style.bg.unwrap_or(self.theme.background)),
                    );
                }
            }
        }
    }

    fn render_vignette(&self, area: Rect, buf: &mut Buffer) {
        let border = 1u16;
        for dy in 0..area.height {
            for dx in 0..area.width {
                let edge_dist = dy
                    .min(area.height - 1 - dy)
                    .min(dx.min(area.width - 1 - dx));
                if edge_dist < border {
                    let x = area.x + dx;
                    let y = area.y + dy;
                    if let Some(cell) = buf.cell_mut((x, y)) {
                        let style = cell.style();
                        let fg = dim_color(style.fg.unwrap_or(self.theme.text), 25);
                        let bg = dim_color(style.bg.unwrap_or(self.theme.background), 25);
                        cell.set_style(Style::default().fg(fg).bg(bg));
                    }
                }
            }
        }
    }

    fn render_glitch(&self, area: Rect, buf: &mut Buffer) {
        let frame_seed = self.rng_seed.wrapping_add(self.frame);
        let mut rng = rand::rngs::StdRng::seed_from_u64(frame_seed);
        if !rng.random_ratio(1, 25) {
            return;
        }
        let rows = rng.random_range(1..=3);
        for _ in 0..rows {
            let y = area.y + rng.random_range(0..area.height);
            let shift = rng.random_range(1..=3i16);
            let row_width = area.width;
            if shift > 0 {
                for dx in (0..row_width).rev() {
                    let src_x = (dx as i16 - shift).max(0) as u16;
                    let dst_x = area.x + dx;
                    let src_x_abs = area.x + src_x;
                    let src_cell = &buf[(src_x_abs, y)];
                    let src_style = src_cell.style();
                    let src_symbol = src_cell.symbol().to_string();
                    if let Some(cell) = buf.cell_mut((dst_x, y)) {
                        cell.set_symbol(&src_symbol);
                        cell.set_style(corrupt_style(src_style, &self.theme));
                    }
                }
            } else {
                for dx in 0..row_width {
                    let src_x = (dx as i16 - shift).min(row_width as i16 - 1) as u16;
                    let dst_x = area.x + dx;
                    let src_x_abs = area.x + src_x;
                    let src_cell = &buf[(src_x_abs, y)];
                    let src_style = src_cell.style();
                    let src_symbol = src_cell.symbol().to_string();
                    if let Some(cell) = buf.cell_mut((dst_x, y)) {
                        cell.set_symbol(&src_symbol);
                        cell.set_style(corrupt_style(src_style, &self.theme));
                    }
                }
            }
        }
    }
}

fn corrupt_style(style: Style, theme: &Theme) -> Style {
    let fg = style.fg.unwrap_or(theme.text);
    let bg = style.bg.unwrap_or(theme.background);
    Style::default().fg(invert_or_noise(fg, theme)).bg(bg)
}

fn invert_or_noise(color: Color, theme: &Theme) -> Color {
    match color {
        Color::Rgb(r, g, b) => {
            if (r as u16 + g as u16 + b as u16) > 384 {
                theme.background
            } else {
                theme.primary
            }
        }
        _ => theme.primary,
    }
}

fn dim_color(color: Color, percent: u16) -> Color {
    match color {
        Color::Rgb(r, g, b) => Color::Rgb(
            (r as u16 * percent / 100) as u8,
            (g as u16 * percent / 100) as u8,
            (b as u16 * percent / 100) as u8,
        ),
        _ => color,
    }
}

use rand::SeedableRng;
