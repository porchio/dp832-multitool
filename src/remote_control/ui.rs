/// Remote Control UI for DP832

use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect, Alignment},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Paragraph, Row, Table, Cell, BorderType},
    Terminal, Frame,
    text::{Line, Span},
};
use std::io;
use std::time::{Duration, Instant};

use super::controller::DP832Controller;

enum InputMode {
    Normal,
    EditingVoltage(u8),  // channel number
    EditingCurrent(u8),  // channel number
}

pub struct RemoteControlUI {
    controller: DP832Controller,
    selected_channel: usize,
    input_mode: InputMode,
    input_buffer: String,
    status_message: String,
    last_update: Instant,
    update_interval: Duration,
}

impl RemoteControlUI {
    pub fn new(controller: DP832Controller) -> Self {
        Self {
            controller,
            selected_channel: 0,
            input_mode: InputMode::Normal,
            input_buffer: String::new(),
            status_message: String::from("Ready. Use ↑/↓ to select channel, V/C to edit, O to toggle output, R to refresh, Q to quit"),
            last_update: Instant::now(),
            update_interval: Duration::from_secs(2), // Update every 2 seconds instead of constantly
        }
    }
    
    pub fn run(&mut self) -> Result<(), io::Error> {
        // Setup terminal
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        loop {
            // Only update measurements periodically or on explicit refresh
            let now = Instant::now();
            if now.duration_since(self.last_update) >= self.update_interval {
                if let Err(e) = self.controller.update_all_channels() {
                    self.status_message = format!("Error updating: {}", e);
                }
                self.last_update = now;
            }
            
            terminal.draw(|f| self.render(f))?;

            // Check for user input with shorter timeout for responsiveness
            if event::poll(Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    match &self.input_mode {
                        InputMode::Normal => {
                            match key.code {
                                KeyCode::Char('q') | KeyCode::Char('Q') => break,
                                KeyCode::Up => {
                                    if self.selected_channel > 0 {
                                        self.selected_channel -= 1;
                                    }
                                }
                                KeyCode::Down => {
                                    if self.selected_channel < 2 {
                                        self.selected_channel += 1;
                                    }
                                }
                                KeyCode::Char('r') | KeyCode::Char('R') => {
                                    // Explicit refresh
                                    if let Err(e) = self.controller.update_all_channels() {
                                        self.status_message = format!("Error updating: {}", e);
                                    } else {
                                        self.status_message = "Refreshed all channels".to_string();
                                    }
                                    self.last_update = Instant::now();
                                }
                                KeyCode::Char('v') | KeyCode::Char('V') => {
                                    let ch = (self.selected_channel + 1) as u8;
                                    self.input_buffer = format!("{:.3}", self.controller.channels[self.selected_channel].voltage_set);
                                    self.input_mode = InputMode::EditingVoltage(ch);
                                    self.status_message = format!("Enter voltage for CH{} (V): ", ch);
                                }
                                KeyCode::Char('c') | KeyCode::Char('C') => {
                                    let ch = (self.selected_channel + 1) as u8;
                                    self.input_buffer = format!("{:.3}", self.controller.channels[self.selected_channel].current_set);
                                    self.input_mode = InputMode::EditingCurrent(ch);
                                    self.status_message = format!("Enter current for CH{} (A): ", ch);
                                }
                                KeyCode::Char('o') | KeyCode::Char('O') => {
                                    let ch = (self.selected_channel + 1) as u8;
                                    let new_state = !self.controller.channels[self.selected_channel].enabled;
                                    if let Err(e) = self.controller.set_output(ch, new_state) {
                                        self.status_message = format!("Error: {}", e);
                                    } else {
                                        self.status_message = format!("CH{} output {}", ch, if new_state { "ON" } else { "OFF" });
                                        // Update state immediately
                                        self.controller.update_channel(ch).ok();
                                    }
                                }
                                _ => {}
                            }
                        }
                        InputMode::EditingVoltage(ch) | InputMode::EditingCurrent(ch) => {
                            match key.code {
                                KeyCode::Enter => {
                                    if let Ok(value) = self.input_buffer.parse::<f64>() {
                                        let result = match &self.input_mode {
                                            InputMode::EditingVoltage(ch) => {
                                                self.controller.set_voltage(*ch, value)
                                            }
                                            InputMode::EditingCurrent(ch) => {
                                                self.controller.set_current(*ch, value)
                                            }
                                            _ => Ok(()),
                                        };
                                        
                                        if let Err(e) = result {
                                            self.status_message = format!("Error: {}", e);
                                        } else {
                                            self.status_message = format!("CH{} updated", ch);
                                            // Update channel state immediately after change
                                            self.controller.update_channel(*ch).ok();
                                        }
                                    } else {
                                        self.status_message = "Invalid number".to_string();
                                    }
                                    self.input_buffer.clear();
                                    self.input_mode = InputMode::Normal;
                                }
                                KeyCode::Esc => {
                                    self.input_buffer.clear();
                                    self.input_mode = InputMode::Normal;
                                    self.status_message = "Cancelled".to_string();
                                }
                                KeyCode::Char(c) => {
                                    self.input_buffer.push(c);
                                }
                                KeyCode::Backspace => {
                                    self.input_buffer.pop();
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
        }

        // Restore terminal
        disable_raw_mode()?;
        execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
        terminal.show_cursor()?;

        Ok(())
    }
    
    fn render(&self, f: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(5),   // Header (larger)
                Constraint::Min(12),     // Channel table
                Constraint::Length(8),   // Help
                Constraint::Length(3),   // Input/Status
            ])
            .split(f.size());
        
        self.render_header(f, chunks[0]);
        self.render_channels(f, chunks[1]);
        self.render_help(f, chunks[2]);
        self.render_status(f, chunks[3]);
    }
    
    fn render_header(&self, f: &mut Frame, area: Rect) {
        let text = vec![
            Line::from(vec![
                Span::styled("╔═══════════════════════════════════════╗", Style::default().fg(Color::Cyan)),
            ]),
            Line::from(vec![
                Span::styled("║  ", Style::default().fg(Color::Cyan)),
                Span::styled("DP832 Remote Control", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::styled("             ║", Style::default().fg(Color::Cyan)),
            ]),
            Line::from(vec![
                Span::styled("╚═══════════════════════════════════════╝", Style::default().fg(Color::Cyan)),
            ]),
        ];
        let paragraph = Paragraph::new(text)
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::NONE));
        f.render_widget(paragraph, area);
    }
    
    fn render_channels(&self, f: &mut Frame, area: Rect) {
        let header_cells = ["CH", "Voltage Set", "Current Set", "Voltage", "Current", "Power", "Output"]
            .iter()
            .map(|h| Cell::from(*h).style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)));
        let header = Row::new(header_cells).height(1).bottom_margin(1);
        
        let rows = (0..3).map(|i| {
            let ch = &self.controller.channels[i];
            let style = if i == self.selected_channel {
                Style::default().bg(Color::Blue).fg(Color::White).add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            
            let output_cell = if ch.enabled {
                Cell::from(Span::styled("● ON", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)))
            } else {
                Cell::from(Span::styled("○ OFF", Style::default().fg(Color::DarkGray)))
            };
            
            Row::new(vec![
                Cell::from(Span::styled(format!(" {} ", i + 1), Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))),
                Cell::from(format!("{:>7.3} V", ch.voltage_set)),
                Cell::from(format!("{:>7.3} A", ch.current_set)),
                Cell::from(Span::styled(format!("{:>7.3} V", ch.voltage_actual), Style::default().fg(Color::Green))),
                Cell::from(Span::styled(format!("{:>7.3} A", ch.current_actual), Style::default().fg(Color::Green))),
                Cell::from(Span::styled(format!("{:>7.3} W", ch.power_actual), Style::default().fg(Color::Magenta))),
                output_cell,
            ]).style(style).height(2)
        });
        
        let table = Table::new(rows, [
            Constraint::Length(5),
            Constraint::Length(13),
            Constraint::Length(13),
            Constraint::Length(13),
            Constraint::Length(13),
            Constraint::Length(13),
            Constraint::Length(10),
        ])
        .header(header)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title(Span::styled(" Channel Status ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)))
            .title_alignment(Alignment::Center));
        
        f.render_widget(table, area);
    }
    
    fn render_help(&self, f: &mut Frame, area: Rect) {
        let help_text = vec![
            Line::from(vec![
                Span::styled("  ↑/↓  ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::raw("Select Channel     "),
                Span::styled("  V  ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::raw("Set Voltage     "),
                Span::styled("  C  ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::raw("Set Current"),
            ]),
            Line::from(vec![
                Span::styled("   O  ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::raw("Toggle Output     "),
                Span::styled("  R  ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::raw("Refresh         "),
                Span::styled("  Q  ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::raw("Quit"),
            ]),
        ];
        
        let paragraph = Paragraph::new(help_text)
            .block(Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title(Span::styled(" Commands ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)))
                .title_alignment(Alignment::Center));
        f.render_widget(paragraph, area);
    }
    
    fn render_status(&self, f: &mut Frame, area: Rect) {
        let (text, style) = match &self.input_mode {
            InputMode::Normal => {
                (vec![Line::from(vec![
                    Span::styled("● ", Style::default().fg(Color::Green)),
                    Span::raw(&self.status_message),
                ])], Style::default())
            }
            InputMode::EditingVoltage(_) | InputMode::EditingCurrent(_) => {
                (vec![Line::from(vec![
                    Span::styled("✎ ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                    Span::raw(&self.status_message),
                    Span::styled(&self.input_buffer, Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                    Span::styled("█", Style::default().fg(Color::Yellow)),
                ])], Style::default().fg(Color::Yellow))
            }
        };
        
        let paragraph = Paragraph::new(text)
            .block(Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(style));
        f.render_widget(paragraph, area);
    }
}
