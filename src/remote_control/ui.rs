/// Remote Control UI for DP832

use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Paragraph, Row, Table, Cell},
    Terminal, Frame,
    text::{Line, Span},
};
use std::io;
use std::time::Duration;

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
}

impl RemoteControlUI {
    pub fn new(controller: DP832Controller) -> Self {
        Self {
            controller,
            selected_channel: 0,
            input_mode: InputMode::Normal,
            input_buffer: String::new(),
            status_message: String::from("Ready. Use ↑/↓ to select channel, V/C to edit, O to toggle output, Q to quit"),
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
            // Update measurements
            if let Err(e) = self.controller.update_all_channels() {
                self.status_message = format!("Error updating: {}", e);
            }
            
            terminal.draw(|f| self.render(f))?;

            if event::poll(Duration::from_millis(200))? {
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
                Constraint::Length(3),  // Header
                Constraint::Min(10),     // Channel table
                Constraint::Length(3),  // Input/Status
            ])
            .split(f.size());
        
        self.render_header(f, chunks[0]);
        self.render_channels(f, chunks[1]);
        self.render_status(f, chunks[2]);
    }
    
    fn render_header(&self, f: &mut Frame, area: Rect) {
        let text = vec![
            Line::from(vec![
                Span::styled("DP832 Remote Control", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::raw(" - "),
                Span::raw(&self.controller.device_id),
            ]),
        ];
        let paragraph = Paragraph::new(text)
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(paragraph, area);
    }
    
    fn render_channels(&self, f: &mut Frame, area: Rect) {
        let header_cells = ["CH", "V Set", "I Set", "V Act", "I Act", "Power", "Output"]
            .iter()
            .map(|h| Cell::from(*h).style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)));
        let header = Row::new(header_cells).height(1).bottom_margin(1);
        
        let rows = (0..3).map(|i| {
            let ch = &self.controller.channels[i];
            let style = if i == self.selected_channel {
                Style::default().bg(Color::DarkGray)
            } else {
                Style::default()
            };
            
            Row::new(vec![
                Cell::from(format!("{}", i + 1)),
                Cell::from(format!("{:.3} V", ch.voltage_set)),
                Cell::from(format!("{:.3} A", ch.current_set)),
                Cell::from(format!("{:.3} V", ch.voltage_actual)),
                Cell::from(format!("{:.3} A", ch.current_actual)),
                Cell::from(format!("{:.3} W", ch.power_actual)),
                Cell::from(if ch.enabled { 
                    Span::styled("ON", Style::default().fg(Color::Green)) 
                } else { 
                    Span::styled("OFF", Style::default().fg(Color::Red)) 
                }),
            ]).style(style)
        });
        
        let table = Table::new(rows, [
            Constraint::Length(3),
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Length(10),
        ])
        .header(header)
        .block(Block::default()
            .borders(Borders::ALL)
            .title("Channels"));
        
        f.render_widget(table, area);
    }
    
    fn render_status(&self, f: &mut Frame, area: Rect) {
        let text = match &self.input_mode {
            InputMode::Normal => {
                vec![Line::from(self.status_message.as_str())]
            }
            InputMode::EditingVoltage(_) | InputMode::EditingCurrent(_) => {
                vec![Line::from(vec![
                    Span::raw(&self.status_message),
                    Span::styled(&self.input_buffer, Style::default().fg(Color::Yellow)),
                    Span::raw("█"),
                ])]
            }
        };
        
        let paragraph = Paragraph::new(text)
            .block(Block::default().borders(Borders::ALL).title("Status"));
        f.render_widget(paragraph, area);
    }
}
