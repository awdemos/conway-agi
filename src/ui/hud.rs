use crate::cell::CellState;
use crate::simulation::Simulation;
use crate::ui::grid::VisualMode;
use crate::ui::theme::Theme;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Widget, Wrap};

pub struct HudPanel<'a> {
    pub sim: &'a Simulation,
    pub theme: &'a Theme,
    pub mode: VisualMode,
    pub message: &'a str,
    pub tick_ms: u64,
    pub asleep: bool,
    pub population_history: &'a [usize],
    pub input_mode: bool,
    pub input_buffer: &'a str,
}

impl<'a> Widget for HudPanel<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let stats = self.sim.step_stats();
        let (sig_avg, sig_max) = self.sim.signal_stats();
        let (dominant_state, state_count) = self.sim.dominant_state();
        let state_label = format!("{:?} {}", dominant_state, state_count);
        let (reward_type, reward_pct) = self.sim.reward_progress();
        let reward_text = match reward_type {
            Some(t) => format!("Rewarding {t}: {:.0}%", reward_pct * 100.0),
            None => "No reward".to_string(),
        };

        let sparkline = braille_sparkline(self.population_history, 40);
        let mode_label = match self.mode {
            VisualMode::Neon => "NEON",
            VisualMode::Voxel => "VOXEL",
            VisualMode::Crt => "CRT",
        };

        let row1 = format!(
            "Gen: {} | Pop: {} | Energy: {:.1}% | Genomes: {} | Rewarded: {}",
            self.sim.generation(),
            self.sim.population(),
            self.sim.average_energy(),
            self.sim.genome_diversity(),
            stats.rewarded,
        );
        let row2 = format!(
            "Mode: {} | Mood: {} | B:{} D:{} E:{} | Signal: {:.1}/{} | {}ms",
            mode_label,
            state_label,
            stats.births,
            stats.deaths,
            stats.emissions,
            sig_avg,
            sig_max,
            self.tick_ms,
        );
        let row3 = format!("{} | {}", reward_text, self.message);
        let row4 = if self.input_mode {
            format!("Chat: {}_", self.input_buffer)
        } else if self.asleep {
            "Sleeping...  Move/press to wake | wasd avatar | x query | click cell | : chat"
                .to_string()
        } else {
            "q quit | p pause | r reset | n step | c view | 0-9 reward | +/- speed | wasd avatar | x query | click cell | : chat".to_string()
        };

        let lines = vec![
            Line::from(vec![
                Span::styled(row1, Style::default().fg(self.theme.text)),
                Span::raw("  "),
                Span::styled(sparkline, Style::default().fg(self.theme.secondary)),
            ]),
            Line::from(Span::styled(row2, Style::default().fg(self.theme.muted))),
            Line::from(Span::styled(row3, Style::default().fg(self.theme.primary))),
            Line::from(Span::styled(
                row4,
                if self.asleep {
                    Style::default().fg(self.theme.dim)
                } else {
                    Style::default()
                        .fg(self.theme.muted)
                        .add_modifier(Modifier::DIM)
                },
            )),
        ];

        let border_color = if reward_type.is_some() {
            self.theme.primary
        } else {
            self.theme.dim
        };
        let title_color = self.theme.text;
        let title = if self.asleep {
            "Status [sleep]"
        } else {
            "Status"
        };

        Paragraph::new(lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(border_color))
                    .title(Span::styled(
                        title,
                        Style::default()
                            .fg(title_color)
                            .add_modifier(Modifier::BOLD),
                    )),
            )
            .alignment(ratatui::layout::Alignment::Left)
            .wrap(Wrap { trim: true })
            .render(area, buf);
    }
}

fn braille_sparkline(values: &[usize], width: usize) -> String {
    if values.is_empty() || width == 0 {
        return String::new();
    }
    let max = *values.iter().max().unwrap_or(&1);
    let max = max.max(1);
    let step = values.len() as f64 / width as f64;
    let mut out = String::with_capacity(width);
    for i in 0..width {
        let idx = ((i as f64 * step).round() as usize).min(values.len() - 1);
        let value = values[idx];
        let normalized = (value as f64 / max as f64).clamp(0.0, 1.0);
        let braille = match normalized {
            0.0..0.125 => '⠀',
            0.125..0.25 => '⡀',
            0.25..0.375 => '⡠',
            0.375..0.5 => '⡤',
            0.5..0.625 => '⡦',
            0.625..0.75 => '⡧',
            0.75..0.875 => '⡷',
            _ => '⡿',
        };
        out.push(braille);
    }
    out
}

pub fn state_color(state: CellState, theme: &Theme) -> Color {
    match state {
        CellState::Calm => theme.text,
        CellState::Anxious => theme.primary,
        CellState::Angry => theme.warning,
        CellState::Sleepy => theme.secondary,
        CellState::Passion => theme.tertiary,
        CellState::Quietude => Color::Rgb(180, 180, 255),
    }
}
