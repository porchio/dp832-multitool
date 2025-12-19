use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    symbols,
    widgets::{Axis, Block, Borders, Chart, Dataset, Gauge, GraphType, Paragraph},
    Terminal,
};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[derive(Clone, Default)]
pub struct RuntimeState {
    pub soc: f64,
    pub voltage: f64,
    pub current: f64,
    pub power: f64,
    pub ocv: f64,
    pub running: bool,
}

struct HistoryData {
    voltage: VecDeque<(f64, f64)>,
    current: VecDeque<(f64, f64)>,
    power: VecDeque<(f64, f64)>,
    time: f64,
    max_points: usize,
}

impl HistoryData {
    fn new(max_points: usize) -> Self {
        Self {
            voltage: VecDeque::new(),
            current: VecDeque::new(),
            power: VecDeque::new(),
            time: 0.0,
            max_points,
        }
    }

    fn add_sample(&mut self, voltage: f64, current: f64, power: f64, dt: f64) {
        self.time += dt;

        self.voltage.push_back((self.time, voltage));
        self.current.push_back((self.time, current));
        self.power.push_back((self.time, power));

        if self.voltage.len() > self.max_points {
            self.voltage.pop_front();
        }
        if self.current.len() > self.max_points {
            self.current.pop_front();
        }
        if self.power.len() > self.max_points {
            self.power.pop_front();
        }
    }

    fn get_time_bounds(&self) -> (f64, f64) {
        if self.voltage.is_empty() {
            (0.0, 10.0)
        } else {
            let min_time = self.voltage.front().map(|(t, _)| *t).unwrap_or(0.0);
            let max_time = self.voltage.back().map(|(t, _)| *t).unwrap_or(10.0);
            (min_time, max_time.max(min_time + 1.0))
        }
    }

    fn get_voltage_bounds(&self) -> (f64, f64) {
        if self.voltage.is_empty() {
            (0.0, 5.0)
        } else {
            let values: Vec<f64> = self.voltage.iter().map(|(_, v)| *v).collect();
            let min = values.iter().cloned().fold(f64::INFINITY, f64::min);
            let max = values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
            let margin = (max - min) * 0.1;
            (min - margin, max + margin)
        }
    }

    fn get_current_bounds(&self) -> (f64, f64) {
        if self.current.is_empty() {
            (0.0, 5.0)
        } else {
            let values: Vec<f64> = self.current.iter().map(|(_, v)| *v).collect();
            let min = values.iter().cloned().fold(f64::INFINITY, f64::min);
            let max = values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
            let margin = (max - min).abs() * 0.1 + 0.1;
            (min - margin, max + margin)
        }
    }

    fn get_power_bounds(&self) -> (f64, f64) {
        if self.power.is_empty() {
            (0.0, 5.0)
        } else {
            let values: Vec<f64> = self.power.iter().map(|(_, v)| *v).collect();
            let min = values.iter().cloned().fold(f64::INFINITY, f64::min);
            let max = values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
            let margin = (max - min).abs() * 0.1 + 0.1;
            (min - margin, max + margin)
        }
    }
}

pub fn run_tui(state: Arc<Mutex<RuntimeState>>, profile_name: String, addr: String) {
    enable_raw_mode().unwrap();
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen).unwrap();

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).unwrap();

    let mut history = HistoryData::new(200);
    let mut last_update = std::time::Instant::now();

    loop {
        let now = std::time::Instant::now();
        let dt = now.duration_since(last_update).as_secs_f64();
        
        terminal
            .draw(|f| {
                let s = state.lock().unwrap().clone();

                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Length(3),
                        Constraint::Length(5),
                        Constraint::Length(5),
                        Constraint::Min(10),
                        Constraint::Length(3),
                    ])
                    .split(f.size());

                // Header
                f.render_widget(
                    Paragraph::new(format!(
                        "Device: {}   Profile: {}",
                        addr, profile_name
                    ))
                    .block(Block::default().borders(Borders::ALL).title("DP832 Battery Simulator")),
                    chunks[0],
                );

                // SoC gauge
                f.render_widget(
                    Gauge::default()
                        .block(Block::default().borders(Borders::ALL).title("State of Charge"))
                        .gauge_style(Style::default().add_modifier(Modifier::BOLD))
                        .percent((s.soc * 100.0) as u16),
                    chunks[1],
                );

                // Metrics
                f.render_widget(
                    Paragraph::new(format!(
                        "Voltage : {:>6.3} V\n\
                         Current : {:>6.3} A\n\
                         Power   : {:>6.2} W\n\
                         OCV     : {:>6.3} V",
                        s.voltage, s.current, s.power, s.ocv
                    ))
                    .block(Block::default().borders(Borders::ALL).title("Measurements")),
                    chunks[2],
                );

                // History chart - split into 3 horizontal sections
                let history_chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([
                        Constraint::Percentage(33),
                        Constraint::Percentage(34),
                        Constraint::Percentage(33),
                    ])
                    .split(chunks[3]);

                let time_bounds = history.get_time_bounds();
                let voltage_bounds = history.get_voltage_bounds();
                let current_bounds = history.get_current_bounds();
                let power_bounds = history.get_power_bounds();

                let voltage_data: Vec<(f64, f64)> = history.voltage.iter().cloned().collect();
                let current_data: Vec<(f64, f64)> = history.current.iter().cloned().collect();
                let power_data: Vec<(f64, f64)> = history.power.iter().cloned().collect();

                // Voltage chart
                let voltage_dataset = vec![
                    Dataset::default()
                        .marker(symbols::Marker::Braille)
                        .style(Style::default().fg(Color::Green))
                        .graph_type(GraphType::Line)
                        .data(&voltage_data),
                ];

                let voltage_chart = Chart::new(voltage_dataset)
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title(format!("Voltage (V)"))
                    )
                    .x_axis(
                        Axis::default()
                            .style(Style::default().fg(Color::Gray))
                            .bounds([time_bounds.0, time_bounds.1]),
                    )
                    .y_axis(
                        Axis::default()
                            .style(Style::default().fg(Color::Gray))
                            .bounds([voltage_bounds.0, voltage_bounds.1])
                            .labels(vec![
                                format!("{:.2}", voltage_bounds.0).into(),
                                format!("{:.2}", voltage_bounds.1).into(),
                            ]),
                    );

                f.render_widget(voltage_chart, history_chunks[0]);

                // Current chart
                let current_dataset = vec![
                    Dataset::default()
                        .marker(symbols::Marker::Braille)
                        .style(Style::default().fg(Color::Yellow))
                        .graph_type(GraphType::Line)
                        .data(&current_data),
                ];

                let current_chart = Chart::new(current_dataset)
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title(format!("Current (A)"))
                    )
                    .x_axis(
                        Axis::default()
                            .style(Style::default().fg(Color::Gray))
                            .bounds([time_bounds.0, time_bounds.1]),
                    )
                    .y_axis(
                        Axis::default()
                            .style(Style::default().fg(Color::Gray))
                            .bounds([current_bounds.0, current_bounds.1])
                            .labels(vec![
                                format!("{:.2}", current_bounds.0).into(),
                                format!("{:.2}", current_bounds.1).into(),
                            ]),
                    );

                f.render_widget(current_chart, history_chunks[1]);

                // Power chart
                let power_dataset = vec![
                    Dataset::default()
                        .marker(symbols::Marker::Braille)
                        .style(Style::default().fg(Color::Cyan))
                        .graph_type(GraphType::Line)
                        .data(&power_data),
                ];

                let power_chart = Chart::new(power_dataset)
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title(format!("Power (W)"))
                    )
                    .x_axis(
                        Axis::default()
                            .style(Style::default().fg(Color::Gray))
                            .bounds([time_bounds.0, time_bounds.1]),
                    )
                    .y_axis(
                        Axis::default()
                            .style(Style::default().fg(Color::Gray))
                            .bounds([power_bounds.0, power_bounds.1])
                            .labels(vec![
                                format!("{:.2}", power_bounds.0).into(),
                                format!("{:.2}", power_bounds.1).into(),
                            ]),
                    );

                f.render_widget(power_chart, history_chunks[2]);

                // Footer
                f.render_widget(
                    Paragraph::new("q: quit   r: reset SoC")
                        .block(Block::default().borders(Borders::ALL)),
                    chunks[4],
                );
            })
            .unwrap();

        // Update history every 100ms
        if dt >= 0.1 {
            let s = state.lock().unwrap().clone();
            history.add_sample(s.voltage, s.current, s.power, dt);
            last_update = now;
        }

        // Input handling
        if event::poll(Duration::from_millis(100)).unwrap() {
            if let Event::Key(k) = event::read().unwrap() {
                match k.code {
                    KeyCode::Char('q') => {
                        state.lock().unwrap().running = false;
                        break;
                    }
                    KeyCode::Char('r') => {
                        state.lock().unwrap().soc = 1.0;
                    }
                    _ => {}
                }
            }
        }
    }

    disable_raw_mode().unwrap();
    execute!(terminal.backend_mut(), LeaveAlternateScreen).unwrap();
}
