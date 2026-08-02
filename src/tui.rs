use std::collections::VecDeque;
use std::time::{Duration, Instant};

use crate::chat::{Chat, ChatResult};
use crate::simulation::Simulation;
use crate::ui::effects::Effects;
use crate::ui::grid::{GridView, ViewMode, VisualMode};
use crate::ui::hud::HudPanel;
use crate::ui::legend::Legend;
use crate::ui::theme::Theme;
use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind, MouseButton, MouseEventKind};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Widget, Wrap};
use ratatui::{DefaultTerminal, Frame};

const MIN_TICK_MS: u64 = 10;
const MAX_TICK_MS: u64 = 2000;
const SLEEP_AFTER_MS: u64 = 120_000;
const HISTORY_CAP: usize = 80;

pub struct App {
    sim: Simulation,
    running: bool,
    tick_ms: u64,
    last_tick: Instant,
    last_input: Instant,
    message: String,
    view: ViewMode,
    visual_mode: VisualMode,
    crt_enabled: bool,
    legend_visible: bool,
    input_mode: InputMode,
    input_buffer: String,
    chat: Chat,
    chat_pending: Option<ChatResult>,
    population_history: VecDeque<usize>,
    query_popup: Option<(usize, usize)>,
    avatar_chat_target: Option<(usize, usize)>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum InputMode {
    #[default]
    Normal,
    Chat,
}

impl App {
    pub fn new(sim: Simulation) -> Self {
        #[cfg(feature = "llm")]
        let chat = if let Some(responder) = crate::chat::llm::OpenAiResponder::from_env() {
            Chat::with_responder(Box::new(responder))
        } else {
            Chat::new()
        };
        #[cfg(not(feature = "llm"))]
        let chat = Chat::new();

        Self {
            sim,
            running: true,
            tick_ms: 250,
            last_tick: Instant::now(),
            last_input: Instant::now(),
            message: String::new(),
            view: ViewMode::Cells,
            visual_mode: VisualMode::Neon,
            crt_enabled: false,
            legend_visible: true,
            input_mode: InputMode::Normal,
            input_buffer: String::new(),
            chat,
            chat_pending: None,
            population_history: VecDeque::with_capacity(HISTORY_CAP),
            query_popup: None,
            avatar_chat_target: None,
        }
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        enable_raw_mode()?;
        let _ = crossterm::execute!(std::io::stdout(), crossterm::event::EnableMouseCapture);
        terminal.clear()?;

        let result = self.loop_(terminal);

        let _ = crossterm::execute!(std::io::stdout(), crossterm::event::DisableMouseCapture);
        disable_raw_mode()?;
        result
    }

    fn loop_(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        loop {
            let now = Instant::now();
            let asleep = now.duration_since(self.last_input).as_millis() as u64 >= SLEEP_AFTER_MS;
            let effective_tick = if asleep { 2000 } else { self.tick_ms };

            if self.running
                && now.duration_since(self.last_tick).as_millis() as u64 >= effective_tick
            {
                self.sim.tick();
                self.apply_pending_chat();
                self.record_population();
                self.last_tick = now;
            }

            terminal.draw(|frame| self.draw(frame, asleep))?;

            if !event::poll(Duration::from_millis(10))? {
                continue;
            }
            let event = event::read()?;
            self.last_input = Instant::now();

            match self.input_mode {
                InputMode::Normal => self.handle_normal_event(event)?,
                InputMode::Chat => self.handle_chat_event(event)?,
            }
        }
    }

    fn record_population(&mut self) {
        if self.population_history.len() >= HISTORY_CAP {
            self.population_history.pop_front();
        }
        self.population_history.push_back(self.sim.population());
    }

    fn theme(&self) -> Theme {
        match self.visual_mode {
            VisualMode::Neon => Theme::neon(),
            VisualMode::Voxel => Theme::voxel(),
            VisualMode::Crt => Theme::crt(),
        }
    }

    fn cycle_visual_mode(&mut self) {
        self.visual_mode = match self.visual_mode {
            VisualMode::Neon => VisualMode::Voxel,
            VisualMode::Voxel => VisualMode::Crt,
            VisualMode::Crt => VisualMode::Neon,
        };
        self.message = format!("Mode: {:?}", self.visual_mode);
    }

    fn handle_normal_event(&mut self, event: Event) -> Result<()> {
        match event {
            Event::Key(key) if key.kind == KeyEventKind::Press => match key.code {
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
                    self.sim.reset(0.18);
                    self.population_history.clear();
                    self.message = "Reset".to_string();
                }
                KeyCode::Char('n') | KeyCode::Char('N') => {
                    self.sim.tick();
                    self.apply_pending_chat();
                    self.record_population();
                    self.message = "Step".to_string();
                }
                KeyCode::Char('c') | KeyCode::Char('C') => {
                    self.view = match self.view {
                        ViewMode::Cells => ViewMode::Signals,
                        ViewMode::Signals => ViewMode::Cells,
                    };
                    self.message = format!("View: {:?}", self.view);
                }
                KeyCode::Char('m') | KeyCode::Char('M') => self.cycle_visual_mode(),
                KeyCode::Char('v') | KeyCode::Char('V') => {
                    self.visual_mode = VisualMode::Voxel;
                    self.message = "Mode: Voxel".to_string();
                }
                KeyCode::Char('t') | KeyCode::Char('T') => {
                    self.crt_enabled = !self.crt_enabled;
                    self.message = format!("CRT: {}", if self.crt_enabled { "on" } else { "off" });
                }
                KeyCode::Char('l') | KeyCode::Char('L') => {
                    self.legend_visible = !self.legend_visible;
                    self.message =
                        format!("Legend: {}", if self.legend_visible { "on" } else { "off" });
                }
                KeyCode::Char('z') | KeyCode::Char('Z') => {
                    self.message = "Sleep mode toggled".to_string();
                    self.last_input =
                        Instant::now() - Duration::from_millis(SLEEP_AFTER_MS.saturating_add(1));
                }
                KeyCode::Char('w') => self.sim.move_avatar(0, -1),
                KeyCode::Char('a') => self.sim.move_avatar(-1, 0),
                KeyCode::Char('s') => self.sim.move_avatar(0, 1),
                KeyCode::Char('d') => self.sim.move_avatar(1, 0),
                KeyCode::Char('x') | KeyCode::Char('X') => {
                    self.query_popup = Some(self.sim.avatar().position());
                }
                KeyCode::Esc => {
                    self.query_popup = None;
                    self.avatar_chat_target = None;
                }
                KeyCode::Char(':') | KeyCode::Char('/') => {
                    self.input_mode = InputMode::Chat;
                    self.input_buffer.clear();
                    let target = self.sim.avatar().position();
                    if self.sim.query_cell(target.0, target.1).is_some() {
                        self.avatar_chat_target = Some(target);
                        self.message = "Chat with cell: ".to_string();
                    } else {
                        self.avatar_chat_target = None;
                        self.message = "Chat to colony: ".to_string();
                    }
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
            },
            Event::Mouse(mouse) => {
                if let MouseEventKind::Down(MouseButton::Left) = mouse.kind {
                    let x = mouse.column as usize;
                    let y = mouse.row as usize;
                    let (gx, gy) = grid_offset_from_mouse(x, y);
                    if self.sim.query_cell(gx, gy).is_some() {
                        self.query_popup = Some((gx, gy));
                    } else {
                        self.sim.poke(gx, gy);
                        self.message = format!("Poked ({}, {})", gx, gy);
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_chat_event(&mut self, event: Event) -> Result<()> {
        match event {
            Event::Key(key) if key.kind == KeyEventKind::Press => match key.code {
                KeyCode::Enter => {
                    let result = self.chat.parse(&self.input_buffer);
                    self.apply_chat(&result);
                    self.chat_pending = Some(result);
                    self.input_mode = InputMode::Normal;
                    self.input_buffer.clear();
                }
                KeyCode::Esc => {
                    self.input_mode = InputMode::Normal;
                    self.input_buffer.clear();
                    self.message = "Chat cancelled".to_string();
                }
                KeyCode::Char(c) => {
                    self.input_buffer.push(c);
                    self.message = format!("Chat: {}", self.input_buffer);
                }
                KeyCode::Backspace => {
                    self.input_buffer.pop();
                    self.message = format!("Chat: {}", self.input_buffer);
                }
                _ => {}
            },
            _ => {}
        }
        Ok(())
    }

    fn apply_chat(&mut self, result: &ChatResult) {
        if let Some(signal) = result.signal_type {
            self.sim.reward(signal);
        }

        if let Some((x, y)) = self.avatar_chat_target
            && let Some(cell) = self.sim.query_cell(x, y)
        {
            let history = self.sim.cell_transcript(x, y).to_vec();
            let reply = self.chat.cell_dialogue(&cell, &result.intent, &history);
            self.sim
                .record_exchange(x, y, result.intent.clone(), reply.clone());
            self.message = format!("{} replies: {}", cell.name_string(), reply);
            self.avatar_chat_target = None;
            return;
        }

        self.message = format!(
            "Chat '{}' -> signal {:?} {:?}",
            result.intent, result.signal_type, result.perturbation
        );
    }

    fn apply_pending_chat(&mut self) {
        if let Some(ref mut result) = self.chat_pending {
            if result.reply_window > 0 {
                result.reply_window -= 1;
            } else {
                let dominant = self.sim.dominant_signal_type();
                let reply = self.chat.reply(dominant);
                self.message = format!("Colony replies: {}", reply);
                self.chat_pending = None;
            }
        }
    }

    fn draw(&self, frame: &mut Frame, asleep: bool) {
        let area = frame.area();
        let theme = self.theme();

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(5), Constraint::Length(6)])
            .split(area);

        let grid_area = chunks[0];
        let status_area = chunks[1];

        let title = match self.view {
            ViewMode::Cells => "Conway AGI — Cells",
            ViewMode::Signals => "Conway AGI — Signals",
        };
        let grid_block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.dim));
        let inner = grid_block.inner(grid_area);
        frame.render_widget(grid_block, grid_area);

        let frame_number = self.sim.generation();
        let grid_view = GridView {
            sim: &self.sim,
            theme: &theme,
            mode: self.visual_mode,
            view: self.view,
            asleep,
            frame: frame_number,
        };
        grid_view.render(inner, frame.buffer_mut());

        let history: Vec<usize> = self.population_history.iter().copied().collect();
        let hud = HudPanel {
            sim: &self.sim,
            theme: &theme,
            mode: self.visual_mode,
            message: &self.message,
            tick_ms: self.tick_ms,
            asleep,
            population_history: &history,
            input_mode: self.input_mode == InputMode::Chat,
            input_buffer: &self.input_buffer,
        };
        hud.render(status_area, frame.buffer_mut());

        if self.legend_visible && grid_area.width >= 30 && grid_area.height >= 12 {
            let legend = Legend {
                theme: theme.clone(),
                view: self.view,
                asleep,
            };
            legend.render(grid_area, frame.buffer_mut());
        }

        if self.input_mode == InputMode::Chat {
            self.draw_chat_prompt(frame, &theme);
        }

        let crt_on = self.crt_enabled || self.visual_mode == VisualMode::Crt;
        if crt_on {
            let effects = Effects {
                theme: theme.clone(),
                frame: frame_number,
                enabled: crt_on,
                rng_seed: 0xC8F0_1A7E,
            };
            effects.render(area, frame.buffer_mut());
        }

        if let Some((qx, qy)) = self.query_popup {
            self.draw_query_popup(frame, qx, qy, &theme);
        }
    }

    fn draw_chat_prompt(&self, frame: &mut Frame, theme: &Theme) {
        let area = frame.area();
        let text = format!("Chat: {}_", self.input_buffer);
        let popup_area = Rect {
            x: area.x + 2,
            y: area.y + area.height.saturating_sub(4),
            width: (text.len() + 4).min(area.width as usize - 4) as u16,
            height: 3,
        };
        Clear.render(popup_area, frame.buffer_mut());
        Paragraph::new(vec![Line::from(Span::styled(
            text,
            Style::default()
                .fg(theme.primary)
                .add_modifier(Modifier::BOLD),
        ))])
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.primary))
                .title(Span::styled(
                    "Say something",
                    Style::default().fg(theme.text),
                )),
        )
        .render(popup_area, frame.buffer_mut());
    }

    fn draw_query_popup(&self, frame: &mut Frame, x: usize, y: usize, theme: &Theme) {
        let Some(cell) = self.sim.query_cell(x, y) else {
            return;
        };
        let lines = vec![
            Line::from(Span::styled(
                format!("Cell {} at ({}, {})", cell.name_string(), x, y),
                Style::default()
                    .fg(theme.tertiary)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(Span::styled(
                format!(
                    "State: {:?} | Age: {} | Energy: {} | Signal: {}",
                    cell.state,
                    cell.age,
                    cell.energy,
                    Chat::describe(cell.genome.signal_type())
                ),
                Style::default().fg(theme.text),
            )),
            Line::from(Span::styled(
                format!(
                    "Attachments: bud {} emit {} rest {} crowd {} solitude {}",
                    cell.attachment.to_budding,
                    cell.attachment.to_emitting,
                    cell.attachment.to_resting,
                    cell.attachment.to_crowds,
                    cell.attachment.to_solitude
                ),
                Style::default().fg(theme.muted),
            )),
            Line::from(Span::styled(
                format!(
                    "Memory: {:08b} | Agit: {} | Viol: {} | Peace: {}",
                    cell.memory, cell.agitation, cell.violence, cell.peace
                ),
                Style::default().fg(theme.muted),
            )),
        ];

        let transcript = self.sim.cell_transcript(x, y);
        let mut text_lines: Vec<Line> = lines
            .into_iter()
            .map(|l| Text::from(l).lines.into_iter().next().unwrap())
            .collect();
        if !transcript.is_empty() {
            text_lines.push(Line::from(Span::styled(
                "Last exchanges:",
                Style::default().fg(theme.primary),
            )));
            for (player, reply) in transcript.iter().rev().take(3) {
                text_lines.push(Line::from(vec![
                    Span::styled(format!("You: {}", player), Style::default().fg(theme.dim)),
                    Span::raw("  "),
                    Span::styled(format!("Cell: {}", reply), Style::default().fg(theme.text)),
                ]));
            }
        } else {
            text_lines.push(Line::from(Span::styled(
                "No exchanges yet. Walk onto the cell and press : to chat.",
                Style::default().fg(theme.dim),
            )));
        }

        let width = text_lines
            .iter()
            .map(|l| l.width())
            .max()
            .unwrap_or(20)
            .min(frame.area().width as usize - 4)
            .max(20);
        let height = text_lines.len().min(frame.area().height as usize - 4) + 2;
        let area = frame.area();
        let popup_area = Rect {
            x: area.x + 2,
            y: area.y + 2,
            width: width as u16,
            height: height as u16,
        };

        Clear.render(popup_area, frame.buffer_mut());
        Paragraph::new(text_lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.primary))
                    .title(Span::styled("Cell Query", Style::default().fg(theme.text))),
            )
            .wrap(Wrap { trim: true })
            .render(popup_area, frame.buffer_mut());
    }
}

fn grid_offset_from_mouse(mx: usize, my: usize) -> (usize, usize) {
    let gx = mx.saturating_sub(1);
    let gy = my.saturating_sub(1);
    (gx, gy)
}

pub fn run_app(sim: Simulation) -> Result<()> {
    let mut terminal = ratatui::init();
    let result = App::new(sim).run(&mut terminal);
    ratatui::restore();
    result
}
