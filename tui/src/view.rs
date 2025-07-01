use crate::component::{CpuMemoryDetails, Menu, MenuState, OverView};
use core::{SharedSystemInfoPoller, SystemInfoPoller, SystemInfoPollingContext, SystemInfoUpdate};
use ratatui::layout::{Constraint, Layout};
use std::sync::mpsc::{self, Receiver};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use tuirealm::terminal::{TerminalBridge, TermionTerminalAdapter};
use tuirealm::{
    Application, AttrValue, Attribute, EventListenerCfg, NoUserEvent, PollStrategy, Sub, SubClause,
    Update,
};

#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub enum Components {
    CpuDetails,
    Menu,
    Overvieww,
}

impl From<&MenuState> for Components {
    fn from(menu_state: &MenuState) -> Self {
        match menu_state {
            MenuState::OverView => Self::Overvieww,
            MenuState::CpuMemoryDetails => Self::CpuDetails,
            _ => Self::Overvieww,
        }
    }
}

#[derive(PartialEq)]
pub enum Message {
    ChangeNextMenu,
    ChangePreviousMenu,
    Quit,
    Tick,
}

pub struct View {
    /// The currently selected tab in the upper menu
    current_tab: MenuState,

    /// Signals the main loop to quit, thus starting to close the app
    quit: bool,

    /// Signals the main loop to re-render the user interface.
    /// Should only set to true when there are changes in the user interface.
    redraw: bool,

    tuirealm: Application<Components, Message, NoUserEvent>,
    terminal: TerminalBridge<TermionTerminalAdapter>,

    system_info: SharedSystemInfoPoller,

    /// Receives updates from the background thread.
    sysinfo_rx: Receiver<SystemInfoUpdate>,
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

        let overview = OverView::default();

        tuirealm
            .mount(
                Components::Menu,
                Box::new(Menu::default()),
                vec![Sub::new(tuirealm::SubEventClause::Any, SubClause::Always)],
            )
            .unwrap();
        tuirealm
            .mount(Components::Overvieww, Box::new(overview), vec![])
            .expect("Failed to mount overview component!");
        tuirealm.active(&Components::Overvieww).unwrap();

        let mut poller = SystemInfoPoller::default();
        poller.init();
        let shared_poller = Arc::new(Mutex::new(poller));
        let poller_clone = shared_poller.clone();

        let (tx, rx) = mpsc::channel();
        thread::spawn(move || loop {
            match poller_clone.lock() {
                Ok(mut poller) => {
                    let ctx = poller.polling_context();
                    let update = SystemInfoUpdate::from((&ctx, &mut *poller));

                    if let Err(error) = tx.send(update) {
                        eprintln!("Failed to send system info update: {}", error);
                        break;
                    }
                }
                Err(error) => eprintln!("Error acquiring polling context lock: {}", error),
            }

            thread::sleep(Duration::from_secs(3));
        });

        View {
            current_tab: MenuState::default(),
            quit: false,
            // render the screen at least one time
            redraw: true,
            terminal,
            tuirealm,
            system_info: shared_poller,
            sysinfo_rx: rx,
        }
    }
}

impl View {
    pub fn render(&mut self) {
        assert!(self
            .terminal
            .draw(|frame| {
                let layout = Layout::vertical([Constraint::Length(3), Constraint::Fill(1)])
                    .split(frame.area());
                let current_view = Components::from(&self.current_tab);
                self.tuirealm.view(&Components::Menu, frame, layout[0]);
                self.tuirealm.view(&current_view, frame, layout[1]);
            })
            .is_ok())
    }

    pub fn run(&mut self) {
        while !self.quit {
            // if have update from the backend, receive it and convert it to json,
            // then update the Overview Component
            if let Ok(update) = self.sysinfo_rx.try_recv() {
                self.handle_update(update);
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

    fn handle_update(&mut self, update: SystemInfoUpdate) {
        match update {
            SystemInfoUpdate::CpuAndMemory(cpu_update) => match cpu_update.to_json() {
                Ok(cpu_update_json) => assert!(self
                    .tuirealm
                    .attr(
                        &Components::CpuDetails,
                        Attribute::Value,
                        AttrValue::String(cpu_update_json)
                    )
                    .is_ok()),
                Err(error) => eprint!("Failed to create JSON from CpuAndMemory: {}", error),
            },
            SystemInfoUpdate::Disk => {}
            SystemInfoUpdate::Network => {}
            SystemInfoUpdate::OverView(overview_update) => match overview_update.to_json() {
                Ok(json) => {
                    assert!(self
                        .tuirealm
                        .attr(
                            &Components::Overvieww,
                            tuirealm::Attribute::Custom("_SYSTEM_OVERVIEW"),
                            tuirealm::AttrValue::String(json),
                        )
                        .is_ok());
                }
                Err(error) => {
                    eprint!("Failed to create JSON from SystemOverviewInfo: {}", error)
                }
            },
            SystemInfoUpdate::Process => {}
        }

        self.redraw = true;
    }

    fn switch_view(&mut self, tab: MenuState) {
        match tab {
            MenuState::CpuMemoryDetails => {
                if !self.tuirealm.mounted(&Components::CpuDetails) {
                    let cpu_info = self.system_info.lock().unwrap().get_cpu_info();
                    self.tuirealm
                        .mount(
                            Components::CpuDetails,
                            Box::new(
                                CpuMemoryDetails::default()
                                    .with_core_count(cpu_info.core_count)
                                    .with_cpu_name(cpu_info.name),
                            ),
                            vec![],
                        )
                        .unwrap();
                }
                self.system_info
                    .lock()
                    .unwrap()
                    .set_polling_context(SystemInfoPollingContext::CpuAndMemory);
                self.tuirealm.blur().unwrap();
                self.tuirealm.active(&Components::CpuDetails).unwrap();
            }
            MenuState::DiskDetails => {}
            MenuState::NetworkDetails => {}
            MenuState::OverView => {
                self.system_info
                    .lock()
                    .unwrap()
                    .set_polling_context(SystemInfoPollingContext::Overview);
            }
            MenuState::ProcessDetails => {}
        }

        self.tuirealm
            .attr(
                &Components::Menu,
                Attribute::Value,
                AttrValue::Length(self.current_tab.index()),
            )
            .unwrap();
    }
}

impl Update<Message> for View {
    fn update(&mut self, msg: Option<Message>) -> Option<Message> {
        if let Some(message) = msg {
            match message {
                Message::ChangeNextMenu => {
                    self.current_tab.next();
                    self.switch_view(self.current_tab);
                }
                Message::ChangePreviousMenu => {
                    self.current_tab.previous();
                    self.switch_view(self.current_tab);
                }
                Message::Tick => {
                    self.redraw = true;
                    self.switch_view(self.current_tab);
                }
                Message::Quit => self.quit = true,
            }
        }

        None
    }
}
