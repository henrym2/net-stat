use anyhow::Result;
use crossterm::{
    event::{self, Event::Key, KeyCode::Char},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    prelude::{CrosstermBackend, Terminal, Layout, Direction, Constraint, Margin, Rect},
    widgets::{Block, Borders, Paragraph}, text::{Text, Line},
};
use sysinfo::{System, SystemExt, NetworkExt, NetworksExt, MacAddr, NetworkData};

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
struct App {
    should_quit: bool,
    sys: System,
    interfaces: Vec<InterfaceData> 
}


struct InterfaceData {
    name: String,
    sent_total: u64,
    rec_total: u64,
    sent: u64,
    rec: u64,
    mac: MacAddr,
}

impl InterfaceData {
    pub fn from(name: &String, data: &NetworkData) -> InterfaceData {
        InterfaceData { name: name.to_string(), sent_total: data.total_transmitted(), rec_total: data.total_received(), sent:data.transmitted(), rec:data.received(), mac:data.mac_address() }
    }
}



// App actions
pub enum Action {
    Tick,
    Quit,
    None,
}

// App ui render function
fn ui(f: &mut Frame<'_>, app: &App) {
   calc_network_interfaces(f, app, None);
}


fn calc_network_interfaces(f: &mut Frame<'_>, app: &App, inner_layout: Option<Rect>) {
    let networks = app.sys.networks() ;
    let mut widgets = Vec::new();

    for (interface_name, data) in networks {
        let lines = vec![
            Line::from(format!("Interface: {}", interface_name)),
            Line::from(format!("Sent/Recieved: {} / {}", data.transmitted(), data.received())),
            Line::from(format!("Total Send/Recieved {} / {}", data.total_transmitted(), data.total_received())),
            Line::from(format!("Mac Address {}", data.mac_address()))
        ];
        let text = Text::from(lines);
        let block = Block::default()
            .borders(Borders::ALL);
        let paragraph = Paragraph::new(text).block(block);
        widgets.push(paragraph);
    };
    let percentage: u16 = (100 / widgets.len()).try_into().unwrap();
    let constraints: Vec<Constraint> = widgets.iter().map(|_| Constraint::Percentage(percentage)).collect();
    let inner_slot = Layout::default()
        .direction(Direction::Vertical) 
        .constraints(
            constraints
        );
        
    let slot = match inner_layout {
        Some(layout) => inner_slot.split(layout.inner(&Margin { horizontal: 1, vertical: 1 })),
        None => inner_slot.split(f.size().inner(&Margin { horizontal: 1, vertical: 1 })),
    };
    for (i, p) in widgets.into_iter().enumerate() {
        f.render_widget(p, slot[i])
    } 

}

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

fn update(app: &mut App, action: Action) {
    match action {
        Action::Quit => app.should_quit = true,
        Action::Tick => {
            app.sys.refresh_networks();
            app.sys.refresh_networks_list();
        }
        _ => {}
    };
}


fn run() -> Result<()> {
    // ratatui terminal
    let mut t = Terminal::new(CrosstermBackend::new(std::io::stderr()))?;

    let mut sys = System::new_all();
    sys.refresh_all();

    let interfaces = sys.networks().into_iter().map(|(name, data)| InterfaceData::from(name, data)).collect();
    // application state

    let mut app = App {
        should_quit: false,
        sys: sys,
        interfaces: interfaces
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
