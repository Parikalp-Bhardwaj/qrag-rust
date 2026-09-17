use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
};

use crate::app::{App, Role, Status};

pub fn render(frame: &mut Frame, app: &App) {
    let root = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(1),
            Constraint::Length(3),
        ])
        .split(frame.area());

    render_header(frame, app, root[0]);

    let body = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(65), Constraint::Percentage(35)])
        .split(root[1]);

    render_chat(frame, app, body[0]);
    render_sources(frame, app, body[1]);
    render_input(frame, app, root[2]);
}

fn render_header(frame: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let status = match app.status {
        Status::Idle => Span::styled("● ready", Style::default().fg(Color::Green)),
        Status::Thinking => Span::styled("● thinking…", Style::default().fg(Color::Yellow)),
    };

    let header = Paragraph::new(Line::from(vec![
        Span::styled(
            " qrag-rust ",
            Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("  RAG over ./docs    "),
        status,
    ]));
    frame.render_widget(header, area);
}

fn render_chat(frame: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let mut lines: Vec<Line> = Vec::new();

    for m in &app.messages {
        let (prefix, style) = match m.role {
            Role::You => (
                "you › ",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Role::Bot => (
                "bot › ",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Role::System => ("· ", Style::default().fg(Color::DarkGray)),
        };

        let mut text_lines = m.text.split('\n');
        let first = text_lines.next().unwrap_or_default();
        lines.push(Line::from(vec![
            Span::styled(prefix, style),
            Span::raw(first.to_string()),
        ]));
        for cont in text_lines {
            lines.push(Line::from(format!("      {cont}")));
        }
        lines.push(Line::from(""));
    }

    let inner_height = area.height.saturating_sub(2);
    let total = lines.len() as u16;
    let max_top = total.saturating_sub(inner_height);
    let offset = max_top.saturating_sub(app.scroll);

    let chat = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL).title(" Chat "))
        .wrap(Wrap { trim: false })
        .scroll((offset, 0));

    frame.render_widget(chat, area);
}

fn render_sources(frame: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let items: Vec<ListItem> = if app.sources.is_empty() {
        vec![ListItem::new(Line::from(Span::styled(
            "no sources yet",
            Style::default().fg(Color::DarkGray),
        )))]
    } else {
        app.sources
            .iter()
            .map(|s| {
                let preview: String = s.text.chars().take(90).collect();
                ListItem::new(vec![
                    Line::from(vec![
                        Span::styled(
                            short_path(&s.file_path),
                            Style::default()
                                .fg(Color::Cyan)
                                .add_modifier(Modifier::BOLD),
                        ),
                        Span::styled(
                            format!("  {:.3}", s.score),
                            Style::default().fg(Color::Yellow),
                        ),
                    ]),
                    Line::from(Span::styled(preview, Style::default().fg(Color::Gray))),
                    Line::from(""),
                ])
            })
            .collect()
    };

    let list = List::new(items).block(Block::default().borders(Borders::ALL).title(" Sources "));
    frame.render_widget(list, area);
}

fn render_input(frame: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let input = Paragraph::new(Line::from(vec![
        Span::styled("› ", Style::default().fg(Color::Green)),
        Span::raw(app.input.as_str()),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Ask (Enter to send) "),
    );
    frame.render_widget(input, area);

    let cursor_x = area.x + 2 + 1 + app.input.chars().count() as u16;
    let cursor_y = area.y + 1;
    frame.set_cursor_position((cursor_x, cursor_y));
}

fn short_path(path: &str) -> String {
    path.rsplit('/').next().unwrap_or(path).to_string()
}
