use ratatui::{
    text::{Line, Text},
    widgets::{Block, Borders, Paragraph, Sparkline},
};
use sysinfo::{MacAddr, NetworkData, NetworkExt, SystemExt};

use crate::app::App;

pub struct InterfaceData {
    pub name: String,
    pub sent_total: u64,
    pub rec_total: u64,
    pub sent: u64,
    pub rec: u64,
    pub mac: MacAddr,
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

pub fn to_network_stat_widgets(app: &App) -> (Vec<Paragraph>, Vec<Sparkline>) {
    let mut network_data = Vec::new();
    let mut network_spark = Vec::new();

    app.net_interfaces.iter().for_each(|interface| {
        let paragraph = create_interface_paragraph(interface);
        let spark = app
            .net_interface_graphs
            .get(&interface.name)
            .and_then(|data| Some(create_interface_graph(&interface.name, data)))
            .unwrap();
        network_data.push(paragraph);
        network_spark.push(spark);
    });
    (network_data, network_spark)
}

pub fn update_net_data(app: &mut App) {
    let interfaces = app
        .sys
        .networks()
        .into_iter()
        .map(|(name, data)| InterfaceData::from(name, data))
        .collect();
    app.net_interfaces = interfaces;
}

pub fn update_graph_data(app: &mut App) {
    app.net_interfaces.iter().for_each(|interface| {
        app.net_interface_graphs
            .entry(interface.name.to_string())
            .and_modify(|l| {
                l.push(interface.sent);
            })
            .or_insert(vec![interface.sent]);
    });
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
