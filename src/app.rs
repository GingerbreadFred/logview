use crossterm::event::KeyCode;
use mlua::Lua;
use std::{error::Error, fs, path::PathBuf};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InputMode {
    Normal,
    Command,
}

pub struct App {
    pub content: Vec<String>,
    pub should_quit: bool,
    pub input_mode: InputMode,
    pub input_buffer: String,
    pub lua: Lua,
}

impl App {
    pub fn new(file_path: Option<PathBuf>) -> Result<App, Box<dyn Error>> {
        let content = if let Some(path) = file_path {
            fs::read_to_string(&path)?
                .lines()
                .map(|s| s.to_string())
                .collect()
        } else {
            vec![
                "Welcome to logview!".to_string(),
                "Press ':' to open command prompt, 'q' to quit.".to_string(),
            ]
        };

        Ok(App {
            content,
            should_quit: false,
            input_mode: InputMode::Normal,
            input_buffer: String::new(),
            lua: Lua::new(),
        })
    }

    pub fn handle_key_event(&mut self, key: KeyCode) {
        match self.input_mode {
            InputMode::Normal => match key {
                KeyCode::Char('q') => self.should_quit = true,
                KeyCode::Char(':') => {
                    self.input_mode = InputMode::Command;
                    self.input_buffer.clear();
                }
                _ => {}
            },
            InputMode::Command => match key {
                KeyCode::Enter => {
                    let command = self.input_buffer.clone();
                    if command == "quit()" {
                        self.should_quit = true;
                    } else {
                        let _ = self.lua.load(&command).exec();
                    }
                    self.input_mode = InputMode::Normal;
                    self.input_buffer.clear();
                }
                KeyCode::Esc => {
                    self.input_mode = InputMode::Normal;
                    self.input_buffer.clear();
                }
                KeyCode::Backspace => {
                    self.input_buffer.pop();
                }
                KeyCode::Char(c) => {
                    self.input_buffer.push(c);
                }
                _ => {}
            },
        }
    }
}
