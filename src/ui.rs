use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Modifier, Style},
    widgets::{Block, Borders, Gauge, Paragraph},
    Terminal,
};
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

pub fn run_tui(state: Arc<Mutex<RuntimeState>>, profile_name: String, addr: String) {
    enable_raw_mode().unwrap();
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen).unwrap();

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).unwrap();

    loop {
        terminal
            .draw(|f| {
                let s = state.lock().unwrap().clone();

                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Length(3),
                        Constraint::Length(5),
                        Constraint::Length(5),
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

                // Footer
                f.render_widget(
                    Paragraph::new("q: quit   p: pause   r: reset SoC")
                        .block(Block::default().borders(Borders::ALL)),
                    chunks[3],
                );
            })
            .unwrap();

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
