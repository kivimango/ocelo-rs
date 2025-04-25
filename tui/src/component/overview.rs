use core::model::CpuInfo;
use tuirealm::{
    command::{Cmd, CmdResult},
    event::{Key, KeyEvent},
    props::Layout,
    ratatui::{
        layout::{Constraint, Direction},
        prelude::Rect,
        widgets::{Borders, Paragraph},
    },
    AttrValue, Attribute, Component, Event, Frame, MockComponent, NoUserEvent, Props, State,
};

use crate::view::Message;

pub struct OverView {
    cpu_info: CpuInfo,
    properties: Props,
}

impl Default for OverView {
    fn default() -> Self {
        Self {
            cpu_info: CpuInfo {
                name: String::new(),
                frequency: 0,
                core_count: 0,
                temperature: None,
            },
            properties: Props::default(),
        }
    }
}

impl OverView {
    /// Sets the processor information during initalization of the component.
    pub fn with_cpu_info(mut self, cpu_info: CpuInfo) -> Self {
        self.cpu_info = cpu_info;
        self
    }
}

impl MockComponent for OverView {
    fn attr(&mut self, attr: Attribute, value: AttrValue) {
        self.properties.set(attr, value);
    }

    fn perform(&mut self, _cmd: Cmd) -> CmdResult {
        CmdResult::None
    }

    fn query(&self, query: Attribute) -> Option<AttrValue> {
        self.properties.get(query)
    }

    fn state(&self) -> State {
        State::None
    }

    fn view(&mut self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(&[
                Constraint::Percentage(25),
                Constraint::Percentage(25),
                Constraint::Percentage(25),
                Constraint::Percentage(25),
            ])
            .chunks(area);
        self.render_cpu_info(frame, chunks[1]);
    }
}

impl Component<Message, NoUserEvent> for OverView {
    fn on(&mut self, event: Event<NoUserEvent>) -> Option<Message> {
        match event {
            Event::Keyboard(KeyEvent {
                code: Key::Char('q') | Key::Esc | Key::Function(10),
                ..
            }) => Some(Message::Quit),
            _ => None,
        }
    }
}

impl OverView {
    fn render_cpu_info(&self, frame: &mut Frame, area: Rect) {
        let cpu_area = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(&[Constraint::Fill(1)])
            .chunks(area);

        let block = tuirealm::ratatui::widgets::Block::default()
            .border_type(tuirealm::props::BorderType::Rounded)
            .borders(Borders::ALL)
            .title("CPU")
            .title_alignment(ratatui::layout::Alignment::Left);

        let text = format!(
            "CPU: {}\nCores: {}\nAvg freq: {} MHz\nTemp: {}",
            self.cpu_info.name,
            self.cpu_info.core_count,
            self.cpu_info.frequency,
            self.cpu_info
                .temperature
                .map_or("N/A".into(), |t| format!("{:.1}°C", t))
        );

        let paragraph = Paragraph::new(text).block(block);

        frame.render_widget(paragraph, cpu_area[0]);
    }
}
