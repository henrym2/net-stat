use std::{collections::HashMap, iter::zip};

use ratatui::prelude::{Constraint, Direction, Layout, Margin, Rect};
use sysinfo::{System, SystemExt};

use crate::{
    networks::{to_network_stat_widgets, update_graph_data, update_net_data, InterfaceData},
    Action, Frame,
};

pub struct App {
    pub should_quit: bool,
    pub sys: System,
    pub net_interfaces: Vec<InterfaceData>,
    pub net_interface_graphs: HashMap<String, Vec<u64>>,
}

pub fn ui(f: &mut Frame<'_>, app: &App) {
    calc_network_status(f, app, None);
}

pub fn update(app: &mut App, action: Action) {
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

fn calc_network_status(f: &mut Frame<'_>, app: &App, inner_layout: Option<Rect>) {
    let (network_data, network_spark) = to_network_stat_widgets(app);

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
