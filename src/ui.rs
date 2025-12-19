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
pub struct ChannelState {
    pub soc: f64,
    pub voltage: f64,
    pub current: f64,
    pub power: f64,
    pub ocv: f64,
    pub profile_name: String,
    pub enabled: bool,
}

#[derive(Clone, Default)]
pub struct RuntimeState {
    pub channels: [ChannelState; 3],
    pub running: bool,
    pub log_messages: VecDeque<String>,
    pub scpi_log_messages: VecDeque<String>,
}

impl RuntimeState {
    pub fn add_log(&mut self, message: String) {
        self.log_messages.push_back(message);
        // Keep last 100 messages
        if self.log_messages.len() > 100 {
            self.log_messages.pop_front();
        }
    }
    
    pub fn add_scpi_log(&mut self, message: String) {
        self.scpi_log_messages.push_back(message);
        // Keep last 200 SCPI messages (more detailed)
        if self.scpi_log_messages.len() > 200 {
            self.scpi_log_messages.pop_front();
        }
    }
}

struct ChannelHistory {
    voltage: VecDeque<(f64, f64)>,
    current: VecDeque<(f64, f64)>,
    power: VecDeque<(f64, f64)>,
}

impl ChannelHistory {
    fn new() -> Self {
        Self {
            voltage: VecDeque::new(),
            current: VecDeque::new(),
            power: VecDeque::new(),
        }
    }

    fn add_sample(&mut self, time: f64, voltage: f64, current: f64, power: f64, max_points: usize) {
        self.voltage.push_back((time, voltage));
        self.current.push_back((time, current));
        self.power.push_back((time, power));

        if self.voltage.len() > max_points {
            self.voltage.pop_front();
        }
        if self.current.len() > max_points {
            self.current.pop_front();
        }
        if self.power.len() > max_points {
            self.power.pop_front();
        }
    }

    fn is_empty(&self) -> bool {
        self.voltage.is_empty()
    }
}

struct HistoryData {
    channels: [ChannelHistory; 3],
    time: f64,
    max_points: usize,
}

impl HistoryData {
    fn new(max_points: usize) -> Self {
        Self {
            channels: [ChannelHistory::new(), ChannelHistory::new(), ChannelHistory::new()],
            time: 0.0,
            max_points,
        }
    }

    fn update_time(&mut self, dt: f64) {
        self.time += dt;
    }

    fn add_sample(&mut self, channel: usize, voltage: f64, current: f64, power: f64) {
        if channel < 3 {
            self.channels[channel].add_sample(self.time, voltage, current, power, self.max_points);
        }
    }

    fn get_time_bounds(&self) -> (f64, f64) {
        let mut min_time = f64::INFINITY;
        let mut max_time = f64::NEG_INFINITY;

        for ch in &self.channels {
            if let Some(&(t, _)) = ch.voltage.front() {
                min_time = min_time.min(t);
            }
            if let Some(&(t, _)) = ch.voltage.back() {
                max_time = max_time.max(t);
            }
        }

        if min_time.is_infinite() {
            (0.0, 10.0)
        } else {
            (min_time, max_time.max(min_time + 1.0))
        }
    }

    fn get_voltage_bounds(&self, channel: usize) -> (f64, f64) {
        if channel >= 3 || self.channels[channel].is_empty() {
            (0.0, 5.0)
        } else {
            let values: Vec<f64> = self.channels[channel].voltage.iter().map(|(_, v)| *v).collect();
            let min = values.iter().cloned().fold(f64::INFINITY, f64::min);
            let max = values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
            let margin = (max - min) * 0.1;
            (min - margin, max + margin)
        }
    }

    fn get_current_bounds(&self, channel: usize) -> (f64, f64) {
        if channel >= 3 || self.channels[channel].is_empty() {
            (0.0, 5.0)
        } else {
            let values: Vec<f64> = self.channels[channel].current.iter().map(|(_, v)| *v).collect();
            let min = values.iter().cloned().fold(f64::INFINITY, f64::min);
            let max = values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
            let margin = (max - min).abs() * 0.1 + 0.1;
            (min - margin, max + margin)
        }
    }

    fn get_power_bounds(&self, channel: usize) -> (f64, f64) {
        if channel >= 3 || self.channels[channel].is_empty() {
            (0.0, 5.0)
        } else {
            let values: Vec<f64> = self.channels[channel].power.iter().map(|(_, v)| *v).collect();
            let min = values.iter().cloned().fold(f64::INFINITY, f64::min);
            let max = values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
            let margin = (max - min).abs() * 0.1 + 0.1;
            (min - margin, max + margin)
        }
    }
}

pub fn run_tui(state: Arc<Mutex<RuntimeState>>, addr: String) {
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

                // Count enabled channels
                let enabled_channels: Vec<usize> = s.channels.iter()
                    .enumerate()
                    .filter(|(_, ch)| ch.enabled)
                    .map(|(i, _)| i)
                    .collect();

                let num_enabled = enabled_channels.len();

                if num_enabled == 0 {
                    // No channels enabled - show simple message
                    let chunks = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints([Constraint::Length(3), Constraint::Min(0)])
                        .split(f.size());

                    f.render_widget(
                        Paragraph::new(format!("Device: {}", addr))
                            .block(Block::default().borders(Borders::ALL).title("DP832 Battery Simulator")),
                        chunks[0],
                    );

                    f.render_widget(
                        Paragraph::new("No channels configured\n\nPress 'q' to quit")
                            .block(Block::default().borders(Borders::ALL)),
                        chunks[1],
                    );
                } else {
                    // Split screen: channels + two log windows at bottom
                    let vertical_split = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints([
                            Constraint::Min(20),      // Main area (channels)
                            Constraint::Length(10),   // Log windows
                        ])
                        .split(f.size());

                    // Split main area vertically for channels + footer
                    let mut constraints = vec![Constraint::Length(3)]; // Header
                    for _ in 0..num_enabled {
                        constraints.push(Constraint::Percentage((100 / num_enabled as u16).max(1)));
                    }
                    constraints.push(Constraint::Length(3)); // Footer

                    let main_chunks = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints(constraints)
                        .split(vertical_split[0]);

                    // Header
                    f.render_widget(
                        Paragraph::new(format!("Device: {}   Active Channels: {}", addr, num_enabled))
                            .block(Block::default().borders(Borders::ALL).title("DP832 Battery Simulator")),
                        main_chunks[0],
                    );

                    // Render each enabled channel
                    for (idx, &ch_num) in enabled_channels.iter().enumerate() {
                        render_channel(
                            f,
                            main_chunks[idx + 1],
                            &s.channels[ch_num],
                            &history,
                            ch_num,
                        );
                    }

                    // Footer
                    f.render_widget(
                        Paragraph::new("q: quit   r: reset SoC   l: clear event log   s: clear SCPI log")
                            .block(Block::default().borders(Borders::ALL)),
                        main_chunks[main_chunks.len() - 1],
                    );

                    // Split bottom area for two log windows side by side
                    let log_split = Layout::default()
                        .direction(Direction::Horizontal)
                        .constraints([
                            Constraint::Percentage(50),  // Event log
                            Constraint::Percentage(50),  // SCPI log
                        ])
                        .split(vertical_split[1]);

                    // Event log window - calculate scroll to show most recent
                    let log_height = log_split[0].height.saturating_sub(2) as usize; // Subtract borders
                    let log_lines = s.log_messages.len();
                    let log_scroll = if log_lines > log_height {
                        (log_lines - log_height) as u16
                    } else {
                        0
                    };
                    
                    let log_text: String = s.log_messages
                        .iter()
                        .map(|msg| format!("{}\n", msg))
                        .collect();
                    
                    f.render_widget(
                        Paragraph::new(log_text)
                            .block(Block::default().borders(Borders::ALL).title("Event Log"))
                            .style(Style::default().fg(Color::Gray))
                            .scroll((log_scroll, 0)),
                        log_split[0],
                    );

                    // SCPI log window - calculate scroll to show most recent
                    let scpi_height = log_split[1].height.saturating_sub(2) as usize; // Subtract borders
                    let scpi_lines = s.scpi_log_messages.len();
                    let scpi_scroll = if scpi_lines > scpi_height {
                        (scpi_lines - scpi_height) as u16
                    } else {
                        0
                    };
                    
                    let scpi_log_text: String = s.scpi_log_messages
                        .iter()
                        .map(|msg| format!("{}\n", msg))
                        .collect();
                    
                    f.render_widget(
                        Paragraph::new(scpi_log_text)
                            .block(Block::default().borders(Borders::ALL).title("SCPI Commands"))
                            .style(Style::default().fg(Color::DarkGray))
                            .scroll((scpi_scroll, 0)),
                        log_split[1],
                    );
                }
            })
            .unwrap();

        // Update history every 100ms
        if dt >= 0.1 {
            let s = state.lock().unwrap().clone();
            history.update_time(dt);
            for (ch_num, ch) in s.channels.iter().enumerate() {
                if ch.enabled {
                    history.add_sample(ch_num, ch.voltage, ch.current, ch.power);
                }
            }
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
                        let mut s = state.lock().unwrap();
                        for ch in &mut s.channels {
                            if ch.enabled {
                                ch.soc = 1.0;
                            }
                        }
                    }
                    KeyCode::Char('l') => {
                        let mut s = state.lock().unwrap();
                        s.log_messages.clear();
                    }
                    KeyCode::Char('s') => {
                        let mut s = state.lock().unwrap();
                        s.scpi_log_messages.clear();
                    }
                    _ => {}
                }
            }
        }
    }

    disable_raw_mode().unwrap();
    execute!(terminal.backend_mut(), LeaveAlternateScreen).unwrap();
}

fn render_channel(
    f: &mut ratatui::Frame,
    area: ratatui::layout::Rect,
    channel: &ChannelState,
    history: &HistoryData,
    ch_num: usize,
) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(30),
            Constraint::Min(0),
        ])
        .split(area);

    // Left side: Metrics and SoC
    let left_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5),
            Constraint::Min(0),
        ])
        .split(chunks[0]);

    // SoC gauge
    f.render_widget(
        Gauge::default()
            .block(Block::default().borders(Borders::ALL).title(format!("CH{} SoC", ch_num + 1)))
            .gauge_style(Style::default().fg(get_channel_color(ch_num)).add_modifier(Modifier::BOLD))
            .percent((channel.soc * 100.0) as u16),
        left_chunks[0],
    );

    // Metrics
    f.render_widget(
        Paragraph::new(format!(
            "Profile: {}\n\
             Voltage: {:>6.3} V\n\
             Current: {:>6.3} A\n\
             Power  : {:>6.2} W\n\
             OCV    : {:>6.3} V",
            channel.profile_name,
            channel.voltage,
            channel.current,
            channel.power,
            channel.ocv
        ))
        .block(Block::default().borders(Borders::ALL).title(format!("Channel {}", ch_num + 1))),
        left_chunks[1],
    );

    // Right side: History charts
    let chart_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(33),
            Constraint::Percentage(34),
            Constraint::Percentage(33),
        ])
        .split(chunks[1]);

    let time_bounds = history.get_time_bounds();
    let voltage_bounds = history.get_voltage_bounds(ch_num);
    let current_bounds = history.get_current_bounds(ch_num);
    let power_bounds = history.get_power_bounds(ch_num);

    let channel_color = get_channel_color(ch_num);

    // Voltage chart
    if !history.channels[ch_num].is_empty() {
        let voltage_data: Vec<(f64, f64)> = history.channels[ch_num].voltage.iter().cloned().collect();
        
        let voltage_dataset = vec![
            Dataset::default()
                .marker(symbols::Marker::Braille)
                .style(Style::default().fg(channel_color))
                .graph_type(GraphType::Line)
                .data(&voltage_data),
        ];

        let voltage_chart = Chart::new(voltage_dataset)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Voltage (V)")
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

        f.render_widget(voltage_chart, chart_chunks[0]);
    }

    // Current chart
    if !history.channels[ch_num].is_empty() {
        let current_data: Vec<(f64, f64)> = history.channels[ch_num].current.iter().cloned().collect();
        
        let current_dataset = vec![
            Dataset::default()
                .marker(symbols::Marker::Braille)
                .style(Style::default().fg(channel_color))
                .graph_type(GraphType::Line)
                .data(&current_data),
        ];

        let current_chart = Chart::new(current_dataset)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Current (A)")
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

        f.render_widget(current_chart, chart_chunks[1]);
    }

    // Power chart
    if !history.channels[ch_num].is_empty() {
        let power_data: Vec<(f64, f64)> = history.channels[ch_num].power.iter().cloned().collect();
        
        let power_dataset = vec![
            Dataset::default()
                .marker(symbols::Marker::Braille)
                .style(Style::default().fg(channel_color))
                .graph_type(GraphType::Line)
                .data(&power_data),
        ];

        let power_chart = Chart::new(power_dataset)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Power (W)")
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

        f.render_widget(power_chart, chart_chunks[2]);
    }
}

fn get_channel_color(ch_num: usize) -> Color {
    match ch_num {
        0 => Color::Green,
        1 => Color::Yellow,
        2 => Color::Cyan,
        _ => Color::White,
    }
}
