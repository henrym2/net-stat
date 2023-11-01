use anyhow::Result;
use app::{ui, update, App};
use std::collections::HashMap;
mod app;
mod networks;
use crossterm::{
    event::{self, Event::Key, KeyCode::Char},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use networks::InterfaceData;
use ratatui::prelude::{CrosstermBackend, Terminal};
use sysinfo::{System, SystemExt};

pub type Frame<'a> = ratatui::Frame<'a, CrosstermBackend<std::io::Stderr>>;

fn startup() -> Result<()> {
    enable_raw_mode()?;
    execute!(std::io::stderr(), EnterAlternateScreen)?;
    Ok(())
}

fn shutdown() -> Result<()> {
    execute!(std::io::stderr(), LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}

// App state

// App actions
pub enum Action {
    Tick,
    Quit,
    None,
}

// App ui render function

fn get_action(_app: &App) -> Action {
    let tick_rate = std::time::Duration::from_millis(250);
    if event::poll(tick_rate).unwrap() {
        if let Key(key) = event::read().unwrap() {
            match key.code {
                Char('q') => Action::Quit,
                _ => Action::None,
            }
        } else {
            Action::None
        }
    } else {
        Action::Tick
    }
}

fn run() -> Result<()> {
    // ratatui terminal
    let mut t = Terminal::new(CrosstermBackend::new(std::io::stderr()))?;

    let mut sys = System::new_all();
    sys.refresh_all();

    let interfaces: Vec<InterfaceData> = sys
        .networks()
        .into_iter()
        .map(|(name, data)| InterfaceData::from(name, data))
        .collect();
    // application state

    let mut set = HashMap::new();
    interfaces.iter().for_each(|x| {
        set.insert(x.name.to_string(), Vec::new());
    });

    let mut app = App {
        should_quit: false,
        sys: sys,
        net_interfaces: interfaces,
        net_interface_graphs: set,
    };

    loop {
        let action = get_action(&mut app);

        // application update
        update(&mut app, action);

        // application render
        t.draw(|f| {
            ui(f, &app);
        })?;

        // application exit
        if app.should_quit {
            break;
        }
    }

    Ok(())
}

fn main() -> Result<()> {
    // setup terminal
    startup()?;

    let result = run();

    // teardown terminal before unwrapping Result of app run
    shutdown()?;

    result?;

    Ok(())
}
