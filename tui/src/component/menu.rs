use crate::Message;
use ratatui::{
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Tabs},
};
use tuirealm::{
    command::{Cmd, CmdResult},
    event::{Key, KeyEvent},
    ratatui::prelude::Rect,
    AttrValue, Attribute, Component, Event, Frame, MockComponent, NoUserEvent, Props, State,
    StateValue,
};

/// The upper menu component in the UI.
/// It is displaying the available menu titles, and highlights the currently selected tab.
///
/// Controls:
/// * Tab => sends a message to the app to change the currently selected menu item to the next one
/// * Backspace => sends message to the app to change the currently selected menu item to the previous one
///
/// # Example:
/// ```norun
/// let menu = Menu::default().with_tab_index(0);
/// ```
#[derive(Default)]
pub struct Menu {
    properties: Props,
}

impl Menu {
    /// Sets the initial tab index
    pub fn with_tab_index(mut self, idx: usize) -> Self {
        self.properties
            .set(Attribute::Value, AttrValue::Length(idx));
        self
    }
}

impl MockComponent for Menu {
    fn attr(&mut self, attr: Attribute, value: AttrValue) {
        self.properties.set(attr, value);
    }

    fn perform(&mut self, _cmd: Cmd) -> CmdResult {
        CmdResult::None
    }

    fn query(&self, attr: Attribute) -> Option<AttrValue> {
        self.properties.get(attr)
    }

    fn state(&self) -> State {
        State::One(StateValue::Usize(
            self.properties
                .get_or(Attribute::Value, AttrValue::Length(0))
                .unwrap_length(),
        ))
    }

    fn view(&mut self, frame: &mut Frame, area: Rect) {
        let titles = ["Overview", "CPU & Memory", "Processes", "Disk", "Network"]
            .iter()
            .map(|t| (*t).into())
            .collect::<Vec<String>>();

        let tab_index = self
            .properties
            .get_or(Attribute::Value, AttrValue::Length(0))
            .unwrap_length();

        let tabs = Tabs::new(titles)
            .block(Block::default().borders(Borders::ALL))
            .select(tab_index)
            .highlight_style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )
            .style(Style::default().fg(Color::White));

        frame.render_widget(tabs, area);
    }
}

impl Component<Message, NoUserEvent> for Menu {
    fn on(&mut self, event: Event<NoUserEvent>) -> Option<Message> {
        match event {
            Event::Keyboard(KeyEvent { code: Key::Tab, .. }) => Some(Message::ChangeNextMenu),
            Event::Keyboard(KeyEvent {
                code: Key::Backspace,
                ..
            }) => Some(Message::ChangePreviousMenu),
            Event::Keyboard(KeyEvent {
                code: Key::Char('q') | Key::Esc | Key::Function(10),
                ..
            }) => Some(Message::Quit),
            _ => None,
        }
    }
}

/// Describes the selectable tabs in the upper menu.
#[derive(Default)]
pub enum MenuState {
    #[default]
    OverView,
    CpuMemoryDetails,
    ProcessDetails,
    DiskDetails,
    NetworkDetails,
}

impl MenuState {
    pub fn index(&self) -> usize {
        match *self {
            Self::OverView => 0,
            Self::CpuMemoryDetails => 1,
            Self::ProcessDetails => 2,
            Self::DiskDetails => 3,
            Self::NetworkDetails => 4,
        }
    }

    pub fn next(&mut self) {
        match self {
            Self::OverView => *self = Self::CpuMemoryDetails,
            Self::CpuMemoryDetails => *self = Self::ProcessDetails,
            Self::ProcessDetails => *self = Self::DiskDetails,
            Self::DiskDetails => *self = Self::NetworkDetails,
            Self::NetworkDetails => *self = Self::OverView,
        }
    }

    pub fn previous(&mut self) {
        match self {
            Self::OverView => *self = Self::CpuMemoryDetails,
            Self::CpuMemoryDetails => *self = Self::ProcessDetails,
            Self::ProcessDetails => *self = Self::DiskDetails,
            Self::DiskDetails => *self = Self::NetworkDetails,
            Self::NetworkDetails => *self = Self::OverView,
        }
    }
}
