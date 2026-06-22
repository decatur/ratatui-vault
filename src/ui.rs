use ratatui_core::layout::{Constraint, Direction, Layout};
use ratatui_core::style::{Color, Modifier, Style};
use ratatui_core::terminal::Terminal;
use ratatui_core::text::{Line, Span};
use ratatui_crossterm::crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen};
use ratatui_crossterm::{CrosstermBackend, crossterm};
use ratatui_textarea::{CursorMove, Input, Key, TextArea};
use ratatui_widgets::block::Block;
use ratatui_widgets::borders::Borders;
use ratatui_widgets::paragraph::Paragraph;
use std::borrow::Cow;
use std::fmt::Display;
use std::io;
use std::path::PathBuf;

use crate::Result;
use crate::crypt::{self, SecretString, decrypt_from_file, encrypt_to_file};

pub fn run(path: Option<String>) -> Result<()> {
    if let Some(path) = path {
        let path = std::path::Path::new(&path);
        assert!(path.is_file());
        let mut editor = Editor::new(path)?;
        editor.run()?;
        close(editor)?;
    } else {
        println!("The edit command needs one argument of type path");
    }
    Ok(())
}
struct SearchBox<'a> {
    textarea: TextArea<'a>,
    open: bool,
}

impl Default for SearchBox<'_> {
    fn default() -> Self {
        let mut textarea = TextArea::default();

        textarea.set_block(Block::default().borders(Borders::ALL).title("Search"));
        Self {
            textarea,
            open: false,
        }
    }
}

impl SearchBox<'_> {
    fn open(&mut self) {
        self.textarea.insert_str("(?i)");
        self.open = true;
    }

    fn close(&mut self) {
        self.open = false;
        // Remove input for next search. Do not recreate `self.textarea` instance to keep undo history so that users can
        // restore previous input easily.
        self.textarea.move_cursor(CursorMove::End);
        self.textarea.delete_line_by_head();
    }

    fn height(&self) -> u16 {
        if self.open { 3 } else { 0 }
    }

    fn input(&mut self, input: Input) -> Option<&'_ str> {
        match input {
            Input {
                key: Key::Enter, ..
            }
            | Input {
                key: Key::Char('m'),
                ctrl: true,
                ..
            } => None, // Disable shortcuts which inserts a newline. See `single_line` example
            input => {
                let modified = self.textarea.input(input);
                modified.then(|| self.textarea.lines()[0].as_str())
            }
        }
    }

    fn set_error(&mut self, err: Option<impl Display>) {
        let b = if let Some(err) = err {
            Block::default()
                .borders(Borders::ALL)
                .title(format!("Search: {err}"))
                .style(Style::default().fg(Color::Red))
        } else {
            Block::default().borders(Borders::ALL).title("Search")
        };
        self.textarea.set_block(b);
    }
}

struct DocumentView<'a> {
    textarea: TextArea<'a>,
    path: PathBuf,
    modified: bool,
    password: SecretString,
}

impl DocumentView<'_> {
    fn new(path: PathBuf) -> Result<Self> {
        let password = crypt::prompt_secret("Please enter password:");
        let plaintext = decrypt_from_file(&path, &password)?;

        let mut textarea = TextArea::new(
            plaintext
                .lines()
                .map(|line| line.to_owned())
                .collect::<Vec<_>>(),
        );

        // textarea.move_cursor(CursorMove::Jump(10, 0));

        // textarea.set_hard_tab_indent(true);
        textarea.set_line_number_style(Style::default().fg(Color::DarkGray));

        Ok(Self {
            textarea,
            path,
            modified: false,
            password,
        })
    }

    fn save(&mut self) -> Result<()> {
        if self.modified {
            loop {
                let yes_no = crypt::prompt("Save modifications (y|n)");
                match yes_no.as_str() {
                    "y" => {
                        let plaintext = self.textarea.lines().join("\n");
                        encrypt_to_file(plaintext, &self.path, &self.password)?;
                        self.modified = false;
                        break;
                    }
                    "n" => break,
                    _ => (),
                }
            }
        };

        // let password = prompt_secret("Please enter password:");
        // if password_changed(&self.path, &password) {
        //     let password2 = prompt_secret("Please repeat password:");
        //     assert_eq!(password, password2);
        // }

        Ok(())
    }
}

struct Editor<'a> {
    document: DocumentView<'a>,
    search: SearchBox<'a>,
    term: Terminal<CrosstermBackend<io::Stdout>>,
    message: Option<Cow<'static, str>>,
}

impl Editor<'_> {
    fn new<I>(path: I) -> Result<Self>
    where
        I: Into<PathBuf>,
    {
        let document = DocumentView::new(path.into())?;

        let mut stdout = io::stdout();
        crossterm::terminal::enable_raw_mode()?;
        // crossterm::execute!(stdout, EnterAlternateScreen)?;
        crossterm::execute!(
            stdout,
            EnterAlternateScreen,
            crossterm::event::DisableMouseCapture
        )?;
        let backend = CrosstermBackend::new(stdout);
        let term = Terminal::new(backend)?;
        Ok(Self {
            document,
            term,
            message: None,
            search: SearchBox::default(),
        })
    }

    fn run(&mut self) -> Result<()> {
        loop {
            let search_height = self.search.height();
            let layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints(
                    [
                        Constraint::Length(search_height),
                        Constraint::Min(1),
                        Constraint::Length(1),
                        Constraint::Length(1),
                    ]
                    .as_ref(),
                );

            self.term.draw(|f| {
                let chunks = layout.split(f.area());

                if search_height > 0 {
                    f.render_widget(&self.search.textarea, chunks[0]);
                }

                let buffer = &self.document;
                let textarea = &buffer.textarea;
                f.render_widget(textarea, chunks[1]);

                // Render status line
                let modified = if buffer.modified { " [modified]" } else { "" };
                let path = format!(" {}{} ", buffer.path.display(), modified);
                let (row, col) = textarea.cursor();
                let cursor = format!("({},{})", row + 1, col + 1);
                let status_chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints(
                        [
                            // Constraint::Length(slot.len() as u16),
                            Constraint::Min(1),
                            Constraint::Length(cursor.len() as u16),
                        ]
                        .as_ref(),
                    )
                    .split(chunks[2]);
                let status_style = Style::default().add_modifier(Modifier::REVERSED);
                // f.render_widget(Paragraph::new(slot).style(status_style), status_chunks[0]);
                f.render_widget(Paragraph::new(path).style(status_style), status_chunks[0]);
                f.render_widget(Paragraph::new(cursor).style(status_style), status_chunks[1]);

                // Render message at bottom
                let message = if let Some(message) = self.message.take() {
                    Line::from(Span::raw(message))
                } else if search_height > 0 {
                    Line::from(vec![
                        Span::raw("Press "),
                        Span::styled("Enter", Style::default().add_modifier(Modifier::BOLD)),
                        Span::raw(" to jump to first match and close, "),
                        Span::styled("Esc", Style::default().add_modifier(Modifier::BOLD)),
                        Span::raw(" to close, "),
                        Span::styled("↓", Style::default().add_modifier(Modifier::BOLD)),
                        Span::raw(" to search next, "),
                        Span::styled("↑", Style::default().add_modifier(Modifier::BOLD)),
                        Span::raw(" to search previous"),
                    ])
                } else {
                    Line::from(vec![
                        Span::styled("^Q", Style::default().add_modifier(Modifier::BOLD)),
                        Span::raw(" to quit, "),
                        Span::styled("^F", Style::default().add_modifier(Modifier::BOLD)),
                        Span::raw(" to search, "),
                        Span::styled("^C", Style::default().add_modifier(Modifier::BOLD)),
                        Span::raw(" yank copy, "),
                        Span::styled("^↑C", Style::default().add_modifier(Modifier::BOLD)),
                        Span::raw(" mouse selectionto clipboard"),
                    ])
                };
                f.render_widget(Paragraph::new(message), chunks[3]);
            })?;

            if search_height > 0 {
                let textarea = &mut self.document.textarea;
                match crossterm::event::read()?.into() {
                    Input { key: Key::Down, .. } => {
                        if !textarea.search_forward(false) {
                            self.search.set_error(Some("Pattern not found"));
                        }
                    }
                    Input { key: Key::Up, .. } => {
                        if !textarea.search_back(false) {
                            self.search.set_error(Some("Pattern not found"));
                        }
                    }
                    Input {
                        key: Key::Enter, ..
                    } => {
                        if !textarea.search_forward(true) {
                            self.message = Some("Pattern not found".into());
                        }
                        self.search.close();
                        textarea.set_search_pattern("")?;
                    }
                    Input { key: Key::Esc, .. } => {
                        self.search.close();
                        textarea.set_search_pattern("")?;
                    }
                    input => {
                        if let Some(query) = self.search.input(input) {
                            let maybe_err = textarea.set_search_pattern(query).err();
                            self.search.set_error(maybe_err);
                        }
                    }
                }
            } else {
                match crossterm::event::read()?.into() {
                    // Input {
                    //     key: Key::Char('o'),
                    //     // ctrl: true,
                    //     alt: false,
                    //     ..
                    // } => {
                    //     if is_raw_mode {
                    //         crossterm::terminal::disable_raw_mode()?;
                    //         println!("###################")
                    //         // crossterm::execute!(self.term.backend_mut(), LeaveAlternateScreen)?;
                    //     } else {
                    //         crossterm::terminal::enable_raw_mode()?;
                    //         // crossterm::execute!(self.term.backend_mut(), EnterAlternateScreen)?;
                    //     }

                    //     is_raw_mode = !is_raw_mode;
                    //     // crossterm::execute!(
                    //     //     self.term.backend_mut(),
                    //     //     // LeaveAlternateScreen,
                    //     //     DisableMouseCapture
                    //     // )?;
                    // }
                    Input {
                        key: Key::Char('c'),
                        ctrl: true,
                        alt: false,
                        ..
                    } => {
                        let textarea = &mut self.document.textarea;
                        textarea.copy();
                        log("Copied selection to yank buffer");
                        // if let Some(clipboard) = clipboard.as_mut() {
                        //     crate::log(&format!("Copied to clipboard: {}", textarea.yank_text()));
                        //     clipboard.set_text(textarea.yank_text())?;
                        // } else {
                        //     crate::log(&format!(
                        //         "Manipulate Selection Data: {}",
                        //         textarea.yank_text()
                        //     ));
                        // }
                    }
                    // Input {
                    //     key: Key::Char('w'),
                    //     ctrl: true,
                    //     alt: false,
                    //     ..
                    // } => {
                    //     let textarea = &mut self.document.textarea;
                    //     textarea.copy();
                    //     // clipboard.set_text(textarea.yank_text().trim())?;
                    //     // crate::log(&format!("Copied to clipboard: {}", textarea.yank_text()));
                    // }
                    Input {
                        key: Key::Char('x'),
                        ctrl: true,
                        alt: false,
                        ..
                    } => {
                        let textarea = &mut self.document.textarea;
                        textarea.cut();
                        // clipboard.set_text(textarea.yank_text())?;
                    }
                    Input {
                        key: Key::Char('v'),
                        ctrl: true,
                        alt: false,
                        ..
                    } => {
                        let s = self.document.textarea.yank_text();
                        // crate::log(&format!("Paste from clipboard: {}", clipboard.get_text()?));
                        log(&format!("Paste from yank buffer: {s}"));
                        self.document.textarea.insert_str(s);
                    }
                    Input {
                        key: Key::Char('q'),
                        ctrl: true,
                        ..
                    } => break,
                    Input {
                        key: Key::Char('f'),
                        ctrl: true,
                        ..
                    } => {
                        self.search.open();
                    }
                    input => {
                        let buffer = &mut self.document;
                        buffer.modified |= buffer.textarea.input(input);
                    }
                }
            }
        }

        Ok(())
    }
}

fn close(mut editor: Editor) -> Result<()> {
    editor.term.show_cursor()?;
    crossterm::terminal::disable_raw_mode()?;
    crossterm::execute!(
        editor.term.backend_mut(),
        LeaveAlternateScreen,
        crossterm::event::DisableMouseCapture
    )?;

    editor.document.save()?;
    Ok(())
}

pub fn log(message: &str) {
    if !std::env::args().any(|arg| arg == "--log") {
        return;
    };
    let filename = "log.txt";
    let mut file = std::fs::File::options()
        .append(true)
        .create(true)
        .open(filename)
        .unwrap();
    use std::io::Write;
    writeln!(&mut file, "{message}").unwrap();
}
