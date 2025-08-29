use crossterm::event::KeyCode;
use std::{error::Error, path::PathBuf};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InputMode {
    Normal,
    Command,
}

pub struct App {
    pub should_quit: bool,
    pub input_mode: InputMode,
    pub input_buffer: String,
    pub buffer: super::buffer::Buffer,
}

impl App {
    pub fn new(file_path: Option<PathBuf>) -> Result<App, Box<dyn Error>> {
        Ok(App {
            should_quit: false,
            input_mode: InputMode::Normal,
            input_buffer: String::new(),
            buffer: super::buffer::Buffer::new(file_path)?,
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
                    if command == "quit" {
                        self.should_quit = true;
                    } else if command.starts_with("open ") {
                        let file_path = command[5..].trim();
                        if let Ok(buffer) = crate::buffer::Buffer::new(Some(PathBuf::from(file_path))) {
                            self.buffer = buffer;
                        }
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

    pub fn get_content(&mut self) -> &Vec<String> {
        self.buffer.get_content()
    }
}
