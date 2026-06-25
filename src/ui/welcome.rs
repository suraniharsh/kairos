//! First-run welcome overlay: shown when `kairos` is launched with no target
//! and no `./todo.txt` exists. Offers to create a `./todo.txt` here or open
//! the bundled sample. Key handling lives in `handle_welcome` (main.rs);
//! `q`/`Esc` quits without creating anything.

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::app::App;

/// Natural (width, height) for the overlay. The caller centers and `Clear`s
/// a rect of this size; `render` fills it.
pub const WIDTH: u16 = 56;
pub const HEIGHT: u16 = 16;

const CHOICES: &[(&str, &str)] = &[
    ("c", "create ./todo.txt here"),
    ("s", "open the sample"),
    ("q", "quit"),
];

/// Render the welcome box, filling `area`. The caller is responsible for
/// centering and clearing — see [`WIDTH`]/[`HEIGHT`].
pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let theme = app.theme();

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.border).bg(theme.panel))
        .title(Line::from(vec![
            Span::raw(" "),
            Span::styled(
                "kairos",
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" · welcome ", Style::default().fg(theme.dim)),
        ]))
        .style(Style::default().bg(theme.panel));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let mut lines: Vec<Line> = Vec::new();
    if inner.width >= super::logo::WIDTH {
        lines.extend(super::logo::centered_lines(theme, inner.width));
        lines.push(Line::raw(""));
    }
    lines.push(Line::from(Span::styled(
        "  no todo.txt in this folder yet".to_string(),
        Style::default().fg(theme.fg),
    )));
    lines.push(Line::raw(""));
    for (k, label) in CHOICES {
        lines.push(Line::from(vec![
            Span::raw("   "),
            Span::styled(
                pad_key(k, 4),
                Style::default()
                    .fg(theme.context)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(label.to_string(), Style::default().fg(theme.fg)),
        ]));
    }

    let para = Paragraph::new(lines).style(Style::default().bg(theme.panel).fg(theme.fg));
    frame.render_widget(para, inner);
}

fn pad_key(s: &str, w: usize) -> String {
    let len = s.chars().count();
    if len >= w {
        s.to_string()
    } else {
        let mut o = s.to_string();
        o.push_str(&" ".repeat(w - len));
        o
    }
}
