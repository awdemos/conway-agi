use std::time::{Duration, Instant};

use crate::cell::Cell;
use crate::simulation::Simulation;
use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};
use ratatui::{DefaultTerminal, Frame};

const MIN_TICK_MS: u64 = 10;
const MAX_TICK_MS: u64 = 2000;

const GENOME_COLORS: [Color; 16] = [
    Color::Red,
    Color::Green,
    Color::Yellow,
    Color::Blue,
    Color::Magenta,
    Color::Cyan,
    Color::White,
    Color::LightRed,
    Color::LightGreen,
    Color::LightYellow,
    Color::LightBlue,
    Color::LightMagenta,
    Color::LightCyan,
    Color::Gray,
    Color::DarkGray,
    Color::Indexed(208),
];

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum ViewMode {
    #[default]
    Cells,
    Signals,
}

pub struct App {
    sim: Simulation,
    running: bool,
    tick_ms: u64,
    last_tick: Instant,
    message: String,
    view: ViewMode,
}

impl App {
    pub fn new(sim: Simulation) -> Self {
        Self {
            sim,
            running: true,
            tick_ms: 250,
            last_tick: Instant::now(),
            message: String::new(),
            view: ViewMode::Cells,
        }
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        enable_raw_mode()?;
        terminal.clear()?;

        let result = self.loop_(terminal);

        disable_raw_mode()?;
        result
    }

    fn loop_(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        loop {
            let now = Instant::now();
            if self.running && now.duration_since(self.last_tick).as_millis() as u64 >= self.tick_ms
            {
                self.sim.tick();
                self.last_tick = now;
            }

            terminal.draw(|frame| self.draw(frame))?;

            if event::poll(Duration::from_millis(10))?
                && let Event::Key(key) = event::read()?
            {
                if key.kind != KeyEventKind::Press {
                    continue;
                }
                match key.code {
                    KeyCode::Char('q') | KeyCode::Char('Q') => return Ok(()),
                    KeyCode::Char(' ') | KeyCode::Char('p') | KeyCode::Char('P') => {
                        self.running = !self.running;
                        self.message = if self.running {
                            "Running".to_string()
                        } else {
                            "Paused".to_string()
                        };
                    }
                    KeyCode::Char('r') | KeyCode::Char('R') => {
                        self.sim.reset(0.25);
                        self.message = "Reset".to_string();
                    }
                    KeyCode::Char('n') | KeyCode::Char('N') => {
                        self.sim.tick();
                        self.message = "Step".to_string();
                    }
                    KeyCode::Char('c') | KeyCode::Char('C') => {
                        self.view = match self.view {
                            ViewMode::Cells => ViewMode::Signals,
                            ViewMode::Signals => ViewMode::Cells,
                        };
                        self.message = format!("View: {:?}", self.view);
                    }
                    KeyCode::Char(d) if d.is_ascii_digit() => {
                        let signal_type = d.to_digit(10).unwrap_or(0) as u8;
                        self.sim.reward(signal_type);
                        self.message = format!("Rewarded signal {}", signal_type);
                    }
                    KeyCode::Char('+') | KeyCode::Char('=') => {
                        self.tick_ms = self.tick_ms.saturating_sub(50).max(MIN_TICK_MS);
                        self.message = format!("Speed: {} ms/tick", self.tick_ms);
                    }
                    KeyCode::Char('-') => {
                        self.tick_ms = self.tick_ms.saturating_add(50).min(MAX_TICK_MS);
                        self.message = format!("Speed: {} ms/tick", self.tick_ms);
                    }
                    _ => {}
                }
            }
        }
    }

    fn draw(&self, frame: &mut Frame) {
        let area = frame.area();

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(5), Constraint::Length(5)])
            .split(area);

        let grid_area = chunks[0];
        let status_area = chunks[1];

        let title = match self.view {
            ViewMode::Cells => "Conway's Game of Life → AGI Simulation (Cells)",
            ViewMode::Signals => "Conway's Game of Life → AGI Simulation (Signals)",
        };
        let grid_block = Block::default().title(title).borders(Borders::ALL);
        let inner = grid_block.inner(grid_area);
        frame.render_widget(grid_block, grid_area);

        let (w, h) = self.sim.grid().size();
        let cell_w = inner.width as usize;
        let cell_h = inner.height as usize;

        let mut lines = Vec::with_capacity(cell_h.min(h));
        for y in 0..cell_h.min(h) {
            let mut spans = Vec::with_capacity(cell_w.min(w) * 2);
            for x in 0..cell_w.min(w) {
                let (ch, style) = match self.view {
                    ViewMode::Cells => render_cell(self.sim.grid().get(x, y)),
                    ViewMode::Signals => render_signal(self.sim.signals().get(x, y)),
                };
                spans.push(Span::styled(ch.to_string(), style));
            }
            lines.push(Line::from(spans));
        }
        let grid_widget = Paragraph::new(lines).alignment(Alignment::Left);
        frame.render_widget(grid_widget, inner);

        self.draw_status(frame, status_area);

        if grid_area.width >= 30 && grid_area.height >= 12 {
            self.draw_legend(frame, grid_area);
        }
    }

    fn draw_status(&self, frame: &mut Frame, area: Rect) {
        let stats = self.sim.step_stats();
        let (sig_avg, sig_max) = self.sim.signal_stats();

        let (reward_type, reward_pct) = self.sim.reward_progress();
        let reward_text = match reward_type {
            Some(t) => format!("Rewarding {t}: {:.0}%", reward_pct * 100.0),
            None => "No reward".to_string(),
        };
        let row1 = format!(
            "Gen: {} | Pop: {} | Energy: {:.1}% | Genomes: {} | Rewarded: {} | {} | {}",
            self.sim.generation(),
            self.sim.population(),
            self.sim.average_energy(),
            self.sim.genome_diversity(),
            stats.rewarded,
            reward_text,
            self.message,
        );
        let row2 = format!(
            "Births: {} | Deaths: {} | Emissions: {} | Signal: {:.1} / {} | Speed: {} ms/tick",
            stats.births, stats.deaths, stats.emissions, sig_avg, sig_max, self.tick_ms,
        );
        let row3 = format!("Message: {}", self.sim.message());
        let row4 =
            "Controls: q quit | p pause | r reset | n step | c view | 0-9 reward | +/- speed";

        let lines = vec![
            Line::from(Span::raw(row1)),
            Line::from(Span::raw(row2)),
            Line::from(Span::raw(row3)),
            Line::from(Span::styled(
                row4,
                Style::default().fg(Color::Gray).add_modifier(Modifier::DIM),
            )),
        ];

        let status = Paragraph::new(lines)
            .block(Block::default().borders(Borders::ALL).title("Status"))
            .alignment(Alignment::Left)
            .wrap(Wrap { trim: true });
        frame.render_widget(status, area);
    }

    fn draw_legend(&self, frame: &mut Frame, grid_area: Rect) {
        let legend_w = 26;
        let legend_h = match self.view {
            ViewMode::Cells => 10,
            ViewMode::Signals => 8,
        };
        let x = grid_area.x + grid_area.width.saturating_sub(legend_w + 1);
        let y = grid_area.y + 1;
        let legend_area = Rect {
            x,
            y,
            width: legend_w,
            height: legend_h,
        };

        frame.render_widget(Clear, legend_area);

        let mut lines = Vec::new();
        lines.push(Line::from(Span::styled(
            " Legend ",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )));

        match self.view {
            ViewMode::Cells => {
                lines.push(Line::from(Span::styled(
                    "Symbol = alive cell",
                    Style::default().fg(Color::Gray),
                )));
                lines.push(Line::from(Span::raw("Color = genome lineage")));
                lines.push(genome_legend_line(0..4));
                lines.push(genome_legend_line(4..8));
                lines.push(genome_legend_line(8..12));
                lines.push(genome_legend_line(12..16));
                lines.push(Line::from(Span::raw("Brightness = energy")));
                lines.push(Line::from(Span::styled(
                    "Bright 100%  Dim ≤30%",
                    Style::default().fg(Color::Gray),
                )));
            }
            ViewMode::Signals => {
                lines.push(Line::from(Span::styled(
                    "Signal strength",
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD),
                )));
                lines.push(signal_legend_row(0..=51, "weak ", Color::DarkGray));
                lines.push(signal_legend_row(52..=103, "low  ", Color::LightYellow));
                lines.push(signal_legend_row(104..=155, "mid  ", Color::Blue));
                lines.push(signal_legend_row(156..=207, "high ", Color::Cyan));
                lines.push(signal_legend_row(208..=255, "max  ", Color::LightCyan));
            }
        }

        let legend = Paragraph::new(lines).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray)),
        );
        frame.render_widget(legend, legend_area);
    }
}

fn render_cell(cell: Cell) -> (char, Style) {
    if !cell.is_alive() {
        return (' ', Style::default().bg(Color::Reset));
    }
    let base = GENOME_COLORS[cell.genome.0 as usize % GENOME_COLORS.len()];
    let fg = brighten(base, cell.energy);
    let symbol = energy_symbol(cell.energy);
    (symbol, Style::default().fg(fg))
}

fn render_signal(value: u8) -> (char, Style) {
    if value == 0 {
        return (' ', Style::default());
    }
    let (symbol, base) = match value {
        208..=255 => ('█', Color::LightCyan),
        156..=207 => ('▓', Color::Cyan),
        104..=155 => ('▒', Color::Blue),
        52..=103 => ('░', Color::LightYellow),
        _ => ('░', Color::DarkGray),
    };
    (symbol, Style::default().fg(base))
}

fn genome_legend_line(range: std::ops::Range<usize>) -> Line<'static> {
    let spans: Vec<Span<'static>> = range
        .map(|i| {
            let color = GENOME_COLORS[i];
            Span::styled(
                format!(" {:02X} ", i),
                Style::default().fg(Color::Black).bg(color),
            )
        })
        .collect();
    Line::from(spans)
}

fn signal_legend_row(
    range: std::ops::RangeInclusive<u8>,
    label: &'static str,
    color: Color,
) -> Line<'static> {
    let start = *range.start();
    let end = *range.end();
    let range_text = if start == end {
        format!("{start:3}")
    } else {
        format!("{start:3}-{end:3}")
    };
    Line::from(vec![
        Span::styled(label, Style::default().fg(color)),
        Span::styled(" █ ", Style::default().fg(color)),
        Span::styled(range_text, Style::default().fg(Color::Gray)),
    ])
}

fn energy_symbol(energy: u8) -> char {
    match energy {
        230..=255 => '█',
        180..=229 => '▓',
        130..=179 => '▒',
        80..=129 => '░',
        30..=79 => '·',
        _ => '∙',
    }
}

fn brighten(base: Color, _energy: u8) -> Color {
    match base {
        Color::Black => Color::DarkGray,
        Color::DarkGray => Color::Gray,
        Color::Gray => Color::White,
        Color::White => Color::White,
        Color::Red => Color::LightRed,
        Color::LightRed => Color::LightRed,
        Color::Green => Color::LightGreen,
        Color::LightGreen => Color::LightGreen,
        Color::Yellow => Color::LightYellow,
        Color::LightYellow => Color::LightYellow,
        Color::Blue => Color::LightBlue,
        Color::LightBlue => Color::LightBlue,
        Color::Magenta => Color::LightMagenta,
        Color::LightMagenta => Color::LightMagenta,
        Color::Cyan => Color::LightCyan,
        Color::LightCyan => Color::LightCyan,
        other => other,
    }
}

pub fn run_app(sim: Simulation) -> Result<()> {
    let mut terminal = ratatui::init();
    let result = App::new(sim).run(&mut terminal);
    ratatui::restore();
    result
}
