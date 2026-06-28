use crate::app::{AppState, Category};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

pub fn render(frame: &mut Frame<'_>, app: &AppState) {
    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(8),
            Constraint::Length(3),
        ])
        .split(frame.area());

    render_header(frame, outer[0], app);

    let body = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(22), Constraint::Min(40)])
        .split(outer[1]);
    render_categories(frame, body[0], app);
    render_rows(frame, body[1], app);
    render_footer(frame, outer[2], app);
}

fn render_header(frame: &mut Frame<'_>, area: ratatui::layout::Rect, app: &AppState) {
    let dirty = if app.dirty() { "modified" } else { "saved" };
    let line = Line::from(vec![
        Span::styled(
            "honk300 config",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("  "),
        Span::styled(
            dirty,
            Style::default().fg(if app.dirty() {
                Color::Yellow
            } else {
                Color::Green
            }),
        ),
        Span::raw("  "),
        Span::raw(app.path.display().to_string()),
    ]);
    frame.render_widget(
        Paragraph::new(line).block(Block::default().borders(Borders::ALL)),
        area,
    );
}

fn render_categories(frame: &mut Frame<'_>, area: ratatui::layout::Rect, app: &AppState) {
    let items: Vec<ListItem<'_>> = Category::ALL
        .iter()
        .enumerate()
        .map(|(idx, category)| {
            let label = format!("{} {}", idx + 1, category.label());
            let style = if *category == app.active_category {
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            ListItem::new(label).style(style)
        })
        .collect();
    frame.render_widget(
        List::new(items).block(Block::default().title("Categories").borders(Borders::ALL)),
        area,
    );
}

fn render_rows(frame: &mut Frame<'_>, area: ratatui::layout::Rect, app: &AppState) {
    let items: Vec<ListItem<'_>> = app
        .rows()
        .into_iter()
        .enumerate()
        .map(|(idx, (label, value))| {
            let marker = if idx == app.selected_row { "> " } else { "  " };
            let style = if idx == app.selected_row {
                Style::default().fg(Color::Cyan)
            } else {
                Style::default()
            };
            ListItem::new(format!("{marker}{label:<24} {value}")).style(style)
        })
        .collect();
    frame.render_widget(
        List::new(items).block(
            Block::default()
                .title(format!("{} settings", app.active_category.label()))
                .borders(Borders::ALL),
        ),
        area,
    );
}

fn render_footer(frame: &mut Frame<'_>, area: ratatui::layout::Rect, app: &AppState) {
    let status_style = if app.status_is_error {
        Style::default().fg(Color::Red)
    } else {
        Style::default().fg(Color::Green)
    };
    let line = Line::from(vec![
        Span::raw("j/k move  Tab category  Enter toggle  Left/Right adjust  S save  R reload  X stop  G start  h/w/m/e/n/b poke  q quit  "),
        Span::styled(&app.status, status_style),
    ]);
    frame.render_widget(
        Paragraph::new(line).block(Block::default().borders(Borders::ALL)),
        area,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use honk_config::Config;
    use ratatui::{backend::TestBackend, Terminal};
    use std::path::PathBuf;

    #[test]
    fn renders_without_panicking() {
        let app = AppState::new(Config::default(), PathBuf::from("config.toml"));
        let backend = TestBackend::new(100, 30);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|frame| render(frame, &app)).unwrap();
    }
}
