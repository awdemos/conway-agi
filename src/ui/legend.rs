use crate::ui::grid::ViewMode;
use crate::ui::theme::Theme;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Widget};

pub struct Legend {
    pub theme: Theme,
    pub view: ViewMode,
    pub asleep: bool,
}

impl Widget for Legend {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let legend_w = 26u16;
        let legend_h = match self.view {
            ViewMode::Cells => 10u16,
            ViewMode::Signals => 8u16,
        };
        let x = area.x + area.width.saturating_sub(legend_w + 1);
        let y = area.y + 1;
        let legend_area = Rect {
            x,
            y,
            width: legend_w,
            height: legend_h,
        };

        Clear.render(legend_area, buf);

        let dim = Style::default().fg(self.theme.dim);
        let mut lines = Vec::new();
        lines.push(Line::from(Span::styled(
            " Legend ",
            Style::default()
                .fg(self.theme.text)
                .add_modifier(Modifier::BOLD),
        )));

        match self.view {
            ViewMode::Cells => {
                lines.push(Line::from(Span::styled("Symbol = alive cell", dim)));
                lines.push(Line::from(Span::styled("Color = genome", dim)));
                lines.push(genome_legend_line(0..4, &self.theme, self.asleep));
                lines.push(genome_legend_line(4..8, &self.theme, self.asleep));
                lines.push(genome_legend_line(8..12, &self.theme, self.asleep));
                lines.push(genome_legend_line(12..16, &self.theme, self.asleep));
                lines.push(Line::from(Span::styled("Brightness = energy", dim)));
                lines.push(Line::from(Span::styled("▓▒░ ·∙ = high→low", dim)));
            }
            ViewMode::Signals => {
                lines.push(Line::from(Span::styled(
                    "Signal strength",
                    Style::default()
                        .fg(self.theme.text)
                        .add_modifier(Modifier::BOLD),
                )));
                lines.push(signal_legend_row(0..=51, "weak ", 0, &self.theme));
                lines.push(signal_legend_row(52..=103, "low  ", 1, &self.theme));
                lines.push(signal_legend_row(104..=155, "mid  ", 2, &self.theme));
                lines.push(signal_legend_row(156..=207, "high ", 3, &self.theme));
                lines.push(signal_legend_row(208..=255, "max  ", 4, &self.theme));
            }
        }

        Paragraph::new(lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(self.theme.dim)),
            )
            .render(legend_area, buf);
    }
}

fn genome_legend_line(range: std::ops::Range<usize>, theme: &Theme, asleep: bool) -> Line<'static> {
    let spans: Vec<Span<'static>> = range
        .map(|i| {
            let color = if asleep {
                theme.dim
            } else {
                theme.genome_colors[i]
            };
            Span::styled(
                format!(" {:02X} ", i),
                Style::default().fg(theme.background).bg(color),
            )
        })
        .collect();
    Line::from(spans)
}

fn signal_legend_row(
    range: std::ops::RangeInclusive<u8>,
    label: &'static str,
    color_idx: usize,
    theme: &Theme,
) -> Line<'static> {
    let start = *range.start();
    let end = *range.end();
    let range_text = if start == end {
        format!("{start:3}")
    } else {
        format!("{start:3}-{end:3}")
    };
    let color = theme.signal_colors[color_idx.min(theme.signal_colors.len() - 1)];
    Line::from(vec![
        Span::styled(label, Style::default().fg(color)),
        Span::styled(" █ ", Style::default().fg(color)),
        Span::styled(range_text, Style::default().fg(theme.muted)),
    ])
}
