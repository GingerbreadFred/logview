use clap::Parser;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use mlua::Lua;
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::Span,
    widgets::{Block, Borders, List, ListItem, Paragraph},
};
use serde::{Deserialize, Serialize};
use std::{error::Error, fs, io, path::PathBuf};

#[derive(Parser)]
#[command(name = "logview")]
#[command(about = "A terminal-based log file viewer with Lua scripting")]
struct Args {
    #[arg(help = "Log file to view")]
    file: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Config {
    // Empty for now, can be extended later
}

impl Default for Config {
    fn default() -> Self {
        Config {}
    }
}

impl Config {
    fn load() -> Result<Config, Box<dyn Error>> {
        let config_path = dirs::home_dir()
            .ok_or("Could not find home directory")?
            .join(".logview.yml");

        if !config_path.exists() {
            let default_config = Config::default();
            let yaml = serde_yaml::to_string(&default_config)?;
            fs::write(&config_path, yaml)?;
            return Ok(default_config);
        }

        let contents = fs::read_to_string(&config_path)?;
        match serde_yaml::from_str::<Config>(&contents) {
            Ok(config) => Ok(config),
            Err(_) => {
                let default_config = Config::default();
                let yaml = serde_yaml::to_string(&default_config)?;
                fs::write(&config_path, yaml)?;
                Ok(default_config)
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum InputMode {
    Normal,
    Command,
}

struct App {
    content: Vec<String>,
    should_quit: bool,
    input_mode: InputMode,
    input_buffer: String,
    lua: Lua,
}

impl App {
    fn new(file_path: Option<PathBuf>) -> Result<App, Box<dyn Error>> {
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

        let lua = Lua::new();

        Ok(App {
            content,
            should_quit: false,
            input_mode: InputMode::Normal,
            input_buffer: String::new(),
            lua,
        })
    }

    fn handle_key_event(&mut self, key: KeyCode) {
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

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let _config = Config::load()?;

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(args.file)?;

    let res = run_app(&mut terminal, &mut app);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{err:?}");
    }

    Ok(())
}

fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, app))?;

        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                app.handle_key_event(key.code);
            }
        }

        if app.should_quit {
            return Ok(());
        }
    }
}

fn ui(f: &mut ratatui::Frame, app: &App) {
    let main_area = if app.input_mode == InputMode::Command {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(3), Constraint::Length(3)])
            .split(f.area());

        let prompt = Paragraph::new(format!(":{}", app.input_buffer))
            .block(Block::default().borders(Borders::ALL).title("Command"));
        f.render_widget(prompt, chunks[1]);

        chunks[0]
    } else {
        f.area()
    };

    let content_lines: Vec<ListItem> = app
        .content
        .iter()
        .map(|line| ListItem::new(Span::styled(line.clone(), Style::default())))
        .collect();

    let list = List::new(content_lines).block(
        Block::default()
            .borders(Borders::ALL)
            .title("Log View")
            .border_style(Style::default().fg(Color::Yellow)),
    );

    f.render_widget(list, main_area);
}
