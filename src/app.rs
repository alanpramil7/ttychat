use std::io::{Write, stdout};

use anyhow::{Error, Result};
use crossterm::style::Stylize;
use crossterm::terminal::{self, Clear, ClearType};
use futures::stream::StreamExt;

use crossterm::event::KeyCode;
use crossterm::{
    cursor::MoveTo,
    event::{Event, EventStream, KeyEvent, KeyModifiers},
    execute,
    style::Print,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};

#[derive(Clone)]
pub struct Message {
    content: String,
    is_user_message: bool,
}

#[derive(Clone)]
pub struct App {
    messages: Vec<Message>,
    user_message: String,
    should_quit: bool,
}

impl App {
    pub fn new() -> Self {
        App {
            messages: Vec::new(),
            user_message: String::new(),
            should_quit: false,
        }
    }

    pub fn handle_keyevent(&mut self, event: KeyEvent) {
        match event {
            KeyEvent {
                code: KeyCode::Char('c'),
                modifiers: KeyModifiers::CONTROL,
                ..
            } => {
                self.should_quit = true;
            }

            KeyEvent {
                code: KeyCode::Char(c),
                modifiers: KeyModifiers::SHIFT | KeyModifiers::NONE,
                ..
            } => {
                self.user_message.push(c);
            }

            KeyEvent {
                code: KeyCode::Enter,
                ..
            } => {
                if !self.user_message.is_empty() {
                    let message = Message {
                        content: self.user_message.clone(),
                        is_user_message: true,
                    };
                    self.messages.push(message);
                }
                self.user_message = "".into();
            }
            KeyEvent {
                code: KeyCode::Backspace,
                ..
            } => {
                self.user_message.pop();
            }
            _ => {}
        }
    }

    pub async fn render(&mut self) -> Result<(), Error> {
        let mut stream = EventStream::new();

        let (_, row) = match terminal::size() {
            Ok((c, r)) => (c, r),
            Err(e) => {
                eprintln!("Errror getting terminal size: {}", e);
                (0, 0)
            }
        };

        execute!(
            stdout(),
            MoveTo(0, row.saturating_sub(2)),
            Clear(ClearType::CurrentLine),
            Print("You: ".blue()),
            Print(self.user_message.clone())
        )?;
        // Move cursor to bottom
        _ = execute!(stdout(), MoveTo(5, row.saturating_sub(2)));

        while !self.should_quit {
            let event = stream.next().await.unwrap();

            match event {
                Ok(Event::Key(key_event)) => {
                    self.handle_keyevent(key_event);
                }
                _ => {}
            }

            self.render_messsages()?;
            execute!(
                stdout(),
                MoveTo(0, row.saturating_sub(2)),
                Clear(ClearType::CurrentLine),
                Print("You: ".blue()),
                Print(self.user_message.clone())
            )?;

            stdout().flush()?;
        }

        Ok(())
    }

    pub fn render_messsages(&self) -> Result<(), Error> {
        for (i, msg) in self.messages.iter().enumerate() {
            if msg.is_user_message {
                execute!(
                    stdout(),
                    MoveTo(0, i as u16),
                    Print("You: ".blue()),
                    Print(msg.content.clone())
                )?;
            } else {
                execute!(
                    stdout(),
                    MoveTo(0, i as u16),
                    Print("Assistant: ".green()),
                    Print(msg.content.clone())
                )?;
            };
        }

        stdout().flush()?;

        Ok(())
    }

    pub async fn run(&mut self) -> Result<(), Error> {
        enable_raw_mode()?;

        execute!(stdout(), EnterAlternateScreen)?;

        self.render().await?;

        execute!(stdout(), LeaveAlternateScreen)?;

        disable_raw_mode()?;

        Ok(())
    }
}
