use clap::{Arg, Command};
use colored::*;
use prettytable::{cell, row, Table};
use std::ops::Deref;
use std::process;
use tokio_wifiscanner as wifiscanner;

#[derive(Debug, Clone, Copy)]
struct NumSignalBars(u8);

impl NumSignalBars {
    fn new(value: u8) -> Self {
        let clamped_value: u8 = value.clamp(3, 255);
        NumSignalBars(clamped_value)
    }
}

impl Deref for NumSignalBars {
    type Target = u8;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Generates a bar representation of signal strength.
/// `filled_bars`: Number of filled symbols to show.
/// `total_bars`: Total number of symbols representing the full scale.
/// `filled_symbol`: Character representing a filled bar.
/// `unfilled_symbol`: Character representing an unfilled bar.
fn generate_bar(
    filled_bars: usize,
    total_bars: usize,
    filled_symbol: char,
    unfilled_symbol: char,
) -> String {
    let mut bars: String = String::new();
    for _ in 0..filled_bars {
        bars.push(filled_symbol);
    }
    for _ in filled_bars..total_bars {
        bars.push(unfilled_symbol);
    }
    bars
}

fn generate_signal_indicator(
    signal_strength: i32,
    total_bars: NumSignalBars,
    min_strength: i32,
    max_strength: i32,
) -> ColoredString {
    if min_strength == max_strength {
        let bars = generate_bar(*total_bars as usize, *total_bars as usize, '▃', '▁');
        return bars.color("green");
    }

    let filled_bars: usize = if signal_strength < min_strength {
        0
    } else if signal_strength > max_strength {
        *total_bars as usize
    } else {
        let range = (max_strength - min_strength) as f32;
        let adjusted_strength = (signal_strength - min_strength) as f32;
        ((adjusted_strength / range) * *total_bars as f32).round() as usize
    };

    let bars: String = generate_bar(filled_bars, *total_bars as usize, '▃', '▁');

    bars.color("green")
}

fn estimate_distance(signal_strength: i32) -> f64 {
    let tx_power: i32 = -30;
    10_f64.powf((tx_power - signal_strength) as f64 / 20.0)
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let matches = Command::new("WiDar: WiFi Distance Estimator")
        .version("0.1.1")
        .author("DJ Magar <dj@djstomp.net>")
        .about("Estimates the distance to nearby WiFi sources based on signal strength")
        .arg(
            Arg::new("interface")
                .short('i')
                .long("interface")
                .value_name("INTERFACE")
                .help("Specifies the network interface to use")
                .default_value("wlan0"),
        )
        .get_matches();

    let _interface = matches.get_one::<String>("interface").unwrap();

    let networks = match wifiscanner::scan().await {
        Ok(n) => n,
        Err(e) => {
            eprintln!("Failed to scan WiFi networks: {}", e.to_string().red());
            process::exit(1);
        }
    };

    let mut table = Table::new();
    table.add_row(row![bFg => "MAC", "SSID", "Channel", "Signal Level", "Security", "Distance"]);

    let total_bars = NumSignalBars::new(5);
    let min_strength = -100;
    let max_strength = -30;

    for network in networks {
        let signal_level = network.signal_level.parse::<i32>().unwrap_or(-100);
        let distance = estimate_distance(signal_level);
        let signal_indicator =
            generate_signal_indicator(signal_level, total_bars, min_strength, max_strength);
        table.add_row(row![
            network.mac,
            network.ssid,
            network.channel,
            format!("{} {}", signal_level, signal_indicator),
            network.security,
            format!("{:.2} meters", distance)
        ]);
    }

    table.printstd();
}
