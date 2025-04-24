use crate::component::OverView;
use std::time::Duration;
use tuirealm::props::Layout;
use tuirealm::ratatui::layout::{Constraint, Direction};
use tuirealm::terminal::{TerminalBridge, TermionTerminalAdapter};
use tuirealm::{Application, EventListenerCfg, NoUserEvent, PollStrategy, Update};

#[derive(Clone, Eq, Hash, PartialEq)]
pub enum Components {
    Overvieww,
}

#[derive(PartialEq)]
pub enum Message {
    Quit,
}

pub struct View {
    /// Signals the main loop to quit, thus starting to close the app
    quit: bool,

    /// Signals the main loop to re-render the user interface.
    /// Should only set to true when there are changes in the user interface.
    redraw: bool,

    tuirealm: Application<Components, Message, NoUserEvent>,
    terminal: TerminalBridge<TermionTerminalAdapter>,
}

impl Default for View {
    /// Initializing terminal with termion terminal backend and ratatui renderer
    fn default() -> Self {
        let mut terminal = TerminalBridge::new_termion();
        terminal.clear_screen().expect("Failed to clear screen!");
        terminal
            .raw_mut()
            .hide_cursor()
            .expect("Failed to hide cursor!");
        let mut tuirealm = Application::init(
            // 30 fps
            EventListenerCfg::default().termion_input_listener(Duration::from_millis(33), 1),
        );

        let cpu_info = core::get_cpu_info();
        let overview = OverView::default().with_cpu_info(cpu_info);

        tuirealm
            .mount(Components::Overvieww, Box::new(overview), vec![])
            .expect("Failed to mount overview component!");
        tuirealm
            .active(&Components::Overvieww)
            .expect("Failed to activate overview component!");

        View {
            quit: false,
            // render the screen at least one time
            redraw: true,
            terminal,
            tuirealm,
        }
    }
}

impl View {
    pub fn render(&mut self) {
        assert!(self
            .terminal
            .draw(|frame| {
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints(&[
                        Constraint::Percentage(25),
                        Constraint::Percentage(25),
                        Constraint::Percentage(25),
                        Constraint::Percentage(25),
                    ])
                    .chunks(frame.area());

                self.tuirealm.view(&Components::Overvieww, frame, chunks[0]);
            })
            .is_ok())
    }

    pub fn run(&mut self) {
        while !self.quit {
            match self.tuirealm.tick(PollStrategy::Once) {
                Ok(messages) if !messages.is_empty() => {
                    self.redraw = true;
                    for msg in messages {
                        let mut message = Some(msg);
                        while let Some(m) = message {
                            message = self.update(Some(m));
                        }
                    }
                }
                Err(error) => {
                    eprintln!("Error rendering ui: {}", error);
                }
                _ => {}
            }

            if self.redraw {
                self.render();
                self.redraw = false;
            }
        }

        self.close();
    }

    /// Restore terminal to its original state and close the application.
    pub fn close(&mut self) {
        self.terminal
            .raw_mut()
            .clear()
            .expect("Failed to clear screen!");
        self.terminal
            .raw_mut()
            .show_cursor()
            .expect("Failed to show cursor!");
    }
}

impl Update<Message> for View {
    fn update(&mut self, msg: Option<Message>) -> Option<Message> {
        if let Some(message) = msg {
            match message {
                Message::Quit => {
                    self.quit = true;
                }
            }
        }

        None
    }
}
