use crate::component::OverView;
use core::model::SystemOverviewInfo;
use std::sync::mpsc::{self, Receiver};
use std::thread;
use std::time::Duration;
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

    /// Receives updates from the background thread.
    sysinfo_rx: Receiver<SystemOverviewInfo>,
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
        let disks = core::get_disk_info();
        let system_info = core::get_system_info();
        let memory_info = core::get_memory_info();
        let network = core::get_network_info();
        let overview_info = SystemOverviewInfo {
            cpu: cpu_info,
            overview: system_info,
            memory: memory_info,
            disks,
            network,
        };
        let overview = OverView::default().with_system_info(overview_info);

        tuirealm
            .mount(Components::Overvieww, Box::new(overview), vec![])
            .expect("Failed to mount overview component!");
        tuirealm
            .active(&Components::Overvieww)
            .expect("Failed to activate overview component!");

        let (tx, rx) = mpsc::channel();
        thread::spawn(move || loop {
            let cpu = core::get_cpu_info();
            let disks = core::get_disk_info();
            let system_info = core::get_system_info();
            let memory_info = core::get_memory_info();
            let network_info = core::get_network_info();
            let overview = SystemOverviewInfo {
                cpu,
                disks,
                overview: system_info,
                memory: memory_info,
                network: network_info,
            };

            if let Err(error) = tx.send(overview) {
                eprintln!("Failed to send system overview information: {}", error);
                break;
            }

            thread::sleep(Duration::from_secs(3));
        });

        View {
            quit: false,
            // render the screen at least one time
            redraw: true,
            terminal,
            tuirealm,
            sysinfo_rx: rx,
        }
    }
}

impl View {
    pub fn render(&mut self) {
        assert!(self
            .terminal
            .draw(|frame| {
                self.tuirealm
                    .view(&Components::Overvieww, frame, frame.area());
            })
            .is_ok())
    }

    pub fn run(&mut self) {
        while !self.quit {
            // if have update from the backend, receive it and convert it to json,
            // then update the Overview Component
            if let Ok(sysinfo) = self.sysinfo_rx.try_recv() {
                match sysinfo.to_json() {
                    Ok(json) => {
                        assert!(self
                            .tuirealm
                            .attr(
                                &Components::Overvieww,
                                tuirealm::Attribute::Custom("_SYSTEM_OVERVIEW"),
                                tuirealm::AttrValue::String(json),
                            )
                            .is_ok());
                        self.redraw = true;
                    }
                    Err(error) => {
                        eprint!("Failed to create JSON from SystemOverviewInfo: {}", error)
                    }
                }
            }

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
