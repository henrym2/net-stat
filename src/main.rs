use std::{collections::HashMap, iter::zip};

use anyhow::Result;
use crossterm::{
    event::{self, Event::Key, KeyCode::Char},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    prelude::{Constraint, CrosstermBackend, Direction, Layout, Margin, Rect, Terminal},
    text::{Line, Text},
    widgets::{Block, Borders, Paragraph, Sparkline},
};
use sysinfo::{MacAddr, NetworkData, NetworkExt, System, SystemExt};

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
    interfaces: Vec<InterfaceData>,
    interface_graphs: HashMap<String, Vec<u64>>,
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
        InterfaceData {
            name: name.to_string(),
            sent_total: data.total_transmitted(),
            rec_total: data.total_received(),
            sent: data.transmitted(),
            rec: data.received(),
            mac: data.mac_address(),
        }
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
    calc_network_status(f, app, None);
}

fn create_interface_paragraph(interface: &InterfaceData) -> Paragraph {
    let lines = vec![
        Line::from(format!("Interface: {}", interface.name)),
        Line::from(format!(
            "Sent/Recieved: {} / {}",
            interface.sent, interface.rec
        )),
        Line::from(format!(
            "Total Send/Recieved {} / {}",
            interface.sent_total, interface.rec_total
        )),
        Line::from(format!("Mac Address {}", interface.mac)),
    ];
    let text = Text::from(lines);
    let block = Block::default().borders(Borders::ALL);
    return Paragraph::new(text).block(block);
}

fn create_interface_graph<'a>(name: &'a String, val: &'a Vec<u64>) -> Sparkline<'a> {
    let block = Block::default()
        .title(name.to_string())
        .borders(Borders::all());
    return Sparkline::default().block(block).data(val);
}

fn calc_network_status(f: &mut Frame<'_>, app: &App, inner_layout: Option<Rect>) {
    let mut network_data = Vec::new();
    let mut network_spark = Vec::new();

    app.interfaces.iter().for_each(|interface| {
        let paragraph = create_interface_paragraph(interface);
        network_data.push(paragraph);
    });

    app.interface_graphs.iter().for_each(|(k, v)| {
        let spark = create_interface_graph(k, v);
        network_spark.push(spark);
    });

    let percentage: u16 = (100 / network_data.len()).try_into().unwrap();
    let constraints: Vec<Constraint> = network_data
        .iter()
        .map(|_| Constraint::Percentage(percentage))
        .collect();
    let inner_slot = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints);

    let slot = match inner_layout {
        Some(layout) => inner_slot.split(layout.inner(&Margin {
            horizontal: 1,
            vertical: 1,
        })),
        None => inner_slot.split(f.size().inner(&Margin {
            horizontal: 1,
            vertical: 1,
        })),
    };

    let widgets_zip = zip(network_data, network_spark);
    for (i, (data, spark)) in widgets_zip.enumerate() {
        let inner_slot = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
            .split(slot[i]);
        f.render_widget(data, inner_slot[0]);
        f.render_widget(spark, inner_slot[1]);
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
            update_net_data(app);
            update_graph_data(app)
        }
        _ => {}
    };
}

fn update_net_data(app: &mut App) {
    let interfaces = app
        .sys
        .networks()
        .into_iter()
        .map(|(name, data)| InterfaceData::from(name, data))
        .collect();
    app.interfaces = interfaces;
}

fn update_graph_data(app: &mut App) {
    app.interfaces.iter().for_each(|interface| {
        app.interface_graphs
            .entry(interface.name.to_string())
            .and_modify(|l| {
                l.push(interface.sent);
            })
            .or_insert(vec![interface.sent]);
    });
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
        interfaces: interfaces,
        interface_graphs: set,
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
