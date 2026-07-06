use ratatui_core::layout::{Constraint, Direction, Layout};
use ratatui_core::style::{Color, Modifier, Style};
use ratatui_core::terminal::Terminal;
use ratatui_crossterm::crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen};
use ratatui_crossterm::{CrosstermBackend, crossterm};
use ratatui_textarea::{Input, Key, TextArea};
use ratatui_widgets::paragraph::Paragraph;
use std::io;

use crate::Result;
use crate::crypt::SecretString;

pub(super) fn show(title: &str) -> Result<SecretString> {
    Prompt::prompt(title)
}

struct Prompt<'a> {
    textarea: TextArea<'a>,
    term: Terminal<CrosstermBackend<io::Stderr>>,
}

impl Prompt<'_> {
    fn prompt(title: &str) -> Result<SecretString> {
        let mut textarea =
            TextArea::new("".lines().map(|line| line.to_owned()).collect::<Vec<_>>());
        textarea.set_line_number_style(Style::default().fg(Color::DarkGray));

        // let mut stdout = io::stdout();
        let mut stdout = io::stderr();
        crossterm::terminal::enable_raw_mode()?;
        // crossterm::execute!(stdout, EnterAlternateScreen)?;
        crossterm::execute!(
            stdout,
            EnterAlternateScreen,
            crossterm::event::DisableMouseCapture
        )?;
        let backend = CrosstermBackend::new(stdout);
        let term = Terminal::new(backend)?;
        let mut prompt = Self { textarea, term };
        prompt.run(title)?;
        let text = SecretString::new(prompt.textarea.lines().join("").trim().to_owned());
        close(prompt)?;
        Ok(text)
    }

    fn run(&mut self, title: &str) -> Result<()> {
        loop {
            let layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(1), Constraint::Length(1)].as_ref());

            self.term.draw(|f| {
                let chunks = layout.split(f.area());

                let textarea = &self.textarea;
                f.render_widget(textarea, chunks[0]);

                // Render status line
                let status_chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Min(1)].as_ref())
                    .split(chunks[1]);
                let status_style = Style::default().add_modifier(Modifier::REVERSED);
                f.render_widget(
                    Paragraph::new(format!("Enter password for {title}; CTRL+Q to exit"))
                        .style(status_style),
                    status_chunks[0],
                );
            })?;

            match crossterm::event::read()?.into() {
                Input {
                    key: Key::Char('q'),
                    ctrl: true,
                    ..
                } => break,
                input => {
                    self.textarea.input(input);
                }
            }
        }

        Ok(())
    }
}

fn close(mut prompt: Prompt) -> Result<()> {
    prompt.term.show_cursor()?;
    crossterm::terminal::disable_raw_mode()?;
    crossterm::execute!(
        prompt.term.backend_mut(),
        LeaveAlternateScreen,
        crossterm::event::DisableMouseCapture
    )?;

    Ok(())
}
