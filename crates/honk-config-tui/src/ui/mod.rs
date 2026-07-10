use crate::app::{AppState, Category};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};

pub fn render(frame: &mut Frame<'_>, app: &AppState) {
    if frame.area().width < 72 || frame.area().height < 20 {
        let notice = format!(
            "honk300 config needs at least 72x20; current terminal is {}x{}",
            frame.area().width,
            frame.area().height
        );
        frame.render_widget(
            Paragraph::new(notice).wrap(Wrap { trim: false }).block(
                Block::default()
                    .title("Terminal too small")
                    .borders(Borders::ALL),
            ),
            frame.area(),
        );
        return;
    }
    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(8),
            Constraint::Length(5),
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
    let rows = app.rows();
    let visible = area.height.saturating_sub(2).max(1) as usize;
    let start = if app.selected_row >= visible {
        app.selected_row + 1 - visible
    } else {
        0
    };
    let end = (start + visible).min(rows.len());
    let items: Vec<ListItem<'_>> = rows
        .into_iter()
        .enumerate()
        .skip(start)
        .take(end.saturating_sub(start))
        .map(|(idx, row)| {
            let marker = if idx == app.selected_row { "> " } else { "  " };
            let style = if idx == app.selected_row {
                Style::default().fg(Color::Cyan)
            } else {
                Style::default()
            };
            ListItem::new(format!("{marker}{:<24} {}", row.label, row.value)).style(style)
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
    let mut lines = vec![Line::raw(
        "j/k move  Tab category  Enter toggle  Left/Right adjust  S save  R reload  U status  X stop  G start  q quit  PgUp/PgDn status",
    )];
    lines.extend(
        app.status
            .lines()
            .map(|line| Line::from(Span::styled(line, status_style))),
    );
    frame.render_widget(
        Paragraph::new(lines)
            .wrap(Wrap { trim: false })
            .scroll((app.status_scroll, 0))
            .block(Block::default().borders(Borders::ALL)),
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

    #[test]
    fn terminals_smaller_than_minimum_show_a_size_notice() {
        let app = AppState::new(Config::default(), PathBuf::from("config.toml"));
        let backend = TestBackend::new(71, 19);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|frame| render(frame, &app)).unwrap();
        let text = buffer_text(&terminal);
        assert!(text.contains("needs at least 72x20"), "{text}");
    }

    #[test]
    fn standard_eighty_by_twenty_four_layout_is_reachable() {
        let app = AppState::new(Config::default(), PathBuf::from("config.toml"));
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|frame| render(frame, &app)).unwrap();
        let text = buffer_text(&terminal);
        assert!(text.contains("honk300 config"), "{text}");
        assert!(text.contains("Categories"), "{text}");
        assert!(!text.contains("needs at least"), "{text}");
    }

    #[test]
    fn long_status_and_errors_wrap_and_can_scroll() {
        let mut app = AppState::new(Config::default(), PathBuf::from("config.toml"));
        app.set_status(
            (0..12)
                .map(|line| format!("status-line-{line:02}"))
                .chain(std::iter::once("TAILTOKEN".into()))
                .collect::<Vec<_>>()
                .join("\n"),
            true,
        );
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|frame| render(frame, &app)).unwrap();
        assert!(!buffer_text(&terminal).contains("TAILTOKEN"));

        app.apply(crate::app::Action::ScrollStatus(12));
        terminal.draw(|frame| render(frame, &app)).unwrap();
        let text = buffer_text(&terminal);
        assert!(text.contains("TAILTOKEN"), "{text}");
    }

    fn buffer_text(terminal: &Terminal<TestBackend>) -> String {
        let buffer = terminal.backend().buffer();
        let width = buffer.area.width as usize;
        buffer
            .content()
            .chunks(width)
            .map(|row| row.iter().map(|cell| cell.symbol()).collect::<String>())
            .collect::<Vec<_>>()
            .join("\n")
    }
}
