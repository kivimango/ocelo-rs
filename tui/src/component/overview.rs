use core::model::SystemOverviewInfo;
use humansize::{BaseUnit, FormatSize, FormatSizeOptions};
use ratatui::widgets::Gauge;
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

use super::get_color_for;

#[derive(Default)]
pub struct OverView {
    properties: Props,
    sysinfo: SystemOverviewInfo,
}

impl OverView {
    /// Sets the system information during initalization of the component.
    pub fn with_system_info(mut self, system_info: SystemOverviewInfo) -> Self {
        self.sysinfo = system_info;
        self
    }
}

impl MockComponent for OverView {
    fn attr(&mut self, attr: Attribute, value: AttrValue) {
        if matches!(attr, Attribute::Custom(_)) {
            let str = value.as_string().unwrap();
            match SystemOverviewInfo::from_json(str) {
                Ok(update) => {
                    self.sysinfo = update;
                }
                Err(error) => eprintln!("Cannot convert SystemOverviewInfo from JSON: {}", error),
            }
        }
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
        let cpu_memory_chunk = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(&[Constraint::Percentage(50), Constraint::Percentage(50)])
            .chunks(chunks[1]);
        self.render_system_info(frame, chunks[0]);
        self.render_cpu_info(frame, cpu_memory_chunk[0]);
        self.render_memory_info(frame, cpu_memory_chunk[1]);
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
            .constraints(&[
                Constraint::Percentage(50),
                Constraint::Length(1),
                Constraint::Percentage(50),
            ])
            .margin(1)
            .chunks(area);

        let block = tuirealm::ratatui::widgets::Block::default()
            .border_type(tuirealm::props::BorderType::Rounded)
            .borders(Borders::ALL)
            .title("CPU")
            .title_alignment(ratatui::layout::Alignment::Left);

        let text = format!(
            "CPU: {}\nCores: {}\nAvg freq: {} MHz\nTemp: {}",
            self.sysinfo.cpu.name,
            self.sysinfo.cpu.core_count,
            self.sysinfo.cpu.frequency,
            self.sysinfo
                .cpu
                .temperature
                .map_or("N/A".into(), |t| format!("{:.1}°C", t))
        );

        let paragraph = Paragraph::new(text);
        let usage = self.sysinfo.cpu.usage;
        let usage_gauge = Gauge::default()
            .percent(usage as u16)
            .gauge_style(get_color_for(usage.into()));

        frame.render_widget(block, area);
        frame.render_widget(paragraph, cpu_area[0]);
        frame.render_widget(usage_gauge, cpu_area[2]);
    }

    fn render_memory_info(&self, frame: &mut Frame, area: Rect) {
        let memory_area = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(&[Constraint::Percentage(50), Constraint::Percentage(50)])
            .margin(1)
            .chunks(area);

        let block = tuirealm::ratatui::widgets::Block::default()
            .border_type(tuirealm::props::BorderType::Rounded)
            .borders(Borders::ALL)
            .title("Memory")
            .title_alignment(ratatui::layout::Alignment::Left);

        let format_size_options = FormatSizeOptions::default()
            .base_unit(BaseUnit::Byte)
            .decimal_places(1)
            .decimal_zeroes(0)
            .kilo(humansize::Kilo::Binary)
            .long_units(false)
            .space_after_value(true);

        let memory_text = format!(
            "Total: {}\nUsed: {}\nAvailable: {}\n",
            self.sysinfo.memory.total.format_size(format_size_options),
            self.sysinfo.memory.used.format_size(format_size_options),
            self.sysinfo
                .memory
                .available
                .format_size(format_size_options),
        );
        let swap_text = format!(
            "Total swap: {}\nUsed swap: {}\nAvailable swap: {}\n",
            self.sysinfo
                .memory
                .swap_total
                .format_size(format_size_options),
            self.sysinfo
                .memory
                .swap_used
                .format_size(format_size_options),
            self.sysinfo
                .memory
                .swap_available
                .format_size(format_size_options),
        );

        let memory_paragraph =
            Paragraph::new(memory_text).alignment(ratatui::layout::Alignment::Left);
        let swap_paragraph = Paragraph::new(swap_text).alignment(ratatui::layout::Alignment::Left);
        frame.render_widget(block, area);
        frame.render_widget(memory_paragraph, memory_area[0]);
        frame.render_widget(swap_paragraph, memory_area[1]);
    }

    fn render_system_info(&self, frame: &mut Frame, area: Rect) {
        let sysinfo_area = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(&[Constraint::Fill(1)])
            .chunks(area);

        let block = tuirealm::ratatui::widgets::Block::default()
            .border_type(tuirealm::props::BorderType::Rounded)
            .borders(Borders::ALL)
            .title("System")
            .title_alignment(ratatui::layout::Alignment::Left);

        let uptime = format_uptime(self.sysinfo.overview.uptime);

        let text = format!(
            "Hostname: {}\nSystem: {}\nUptime: {}\nLoad average: 1m:{}% 5m:{}% 15m:{}%\n",
            self.sysinfo.overview.host_name,
            self.sysinfo.overview.kernel_version,
            uptime,
            self.sysinfo.overview.load_one_minute,
            self.sysinfo.overview.load_five_minutes,
            self.sysinfo.overview.load_fifteen_minutes
        );

        let paragraph = Paragraph::new(text).block(block);
        frame.render_widget(paragraph, sysinfo_area[0]);
    }
}

fn format_uptime(seconds: u64) -> String {
    let days = seconds / 86400;
    let hours = (seconds % 86400) / 3600;
    let minutes = (seconds % 3600) / 60;
    let secs = seconds % 60;

    let mut parts = vec![];

    if days > 0 {
        parts.push(format!("{} days", days));
    }
    if hours > 0 {
        parts.push(format!("{} hours", hours));
    }
    if minutes > 0 {
        parts.push(format!("{} minutes", minutes));
    }
    if secs > 0 || parts.is_empty() {
        parts.push(format!("{} seconds", secs));
    }

    parts.join(", ")
}
