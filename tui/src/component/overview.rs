use super::get_color_for;
use crate::view::Message;
use core::model::SystemOverviewInfo;
use humansize::{BaseUnit, FormatSize, FormatSizeOptions, Kilo};
use ratatui::widgets::{Block, Gauge};
use tuirealm::{
    command::{Cmd, CmdResult},
    props::Layout,
    ratatui::{
        layout::{Constraint, Direction},
        prelude::Rect,
        widgets::{Borders, Paragraph},
    },
    AttrValue, Attribute, Component, Event, Frame, MockComponent, NoUserEvent, Props, State,
};

#[derive(Default)]
pub struct OverView {
    properties: Props,
    sysinfo: SystemOverviewInfo,
    /// Pre-calculated information for the top 3 used space drive
    disk_usage: String,
}

impl OverView {
    /// Sets the system information during initalization of the component.
    pub fn with_system_info(mut self, system_info: SystemOverviewInfo) -> Self {
        self.sysinfo = system_info;
        self.disk_usage = self.calculate_disk_usage_info();
        self
    }

    fn calculate_disk_usage_info(&self) -> String {
        let format_opts = FormatSizeOptions::default()
            .base_unit(BaseUnit::Byte)
            .decimal_places(1)
            .decimal_zeroes(0)
            .kilo(humansize::Kilo::Binary)
            .long_units(false)
            .space_after_value(true);

        let text = self
            .sysinfo
            .disks
            .disks
            .iter()
            .take(3)
            .map(|d| {
                let percent = if d.total_space == 0 {
                    0.0
                } else {
                    d.used_space as f64 / d.total_space as f64 * 100.0
                };
                format!(
                    "{:<10} {:>5.1}%  {:>8} / {:<8}",
                    d.mount,
                    percent,
                    d.used_space.format_size(format_opts),
                    d.total_space.format_size(format_opts),
                )
            })
            .collect::<Vec<_>>()
            .join("\n");
        text
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
        self.render_disks_info(frame, chunks[2]);
        self.render_network_info(frame, chunks[3]);
    }
}

impl Component<Message, NoUserEvent> for OverView {
    fn on(&mut self, _event: Event<NoUserEvent>) -> Option<Message> {
        None
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

    fn render_disks_info(&self, frame: &mut Frame, area: Rect) {
        let disks_area = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(&[
                Constraint::Percentage(25),
                Constraint::Percentage(25),
                Constraint::Percentage(25),
                Constraint::Percentage(25),
            ])
            .margin(1)
            .chunks(area);
        let block = Block::default()
            .border_type(tuirealm::props::BorderType::Rounded)
            .borders(Borders::ALL)
            .title("Mass storage")
            .title_alignment(ratatui::layout::Alignment::Left);

        let format_size_options = FormatSizeOptions::default()
            .base_unit(BaseUnit::Byte)
            .decimal_places(1)
            .decimal_zeroes(0)
            .kilo(humansize::Kilo::Binary)
            .long_units(false)
            .space_after_value(true);

        let total_space: u64 = self.sysinfo.disks.disks.iter().map(|d| d.total_space).sum();
        let used_space: u64 = self.sysinfo.disks.disks.iter().map(|d| d.used_space).sum();
        let device_count = self.sysinfo.disks.disks.len();
        let available_space = total_space - used_space;
        let text = format!(
            "Total mass storage space: {}\nUsed space: {}\nAvailable space: {}\nDevice count: {}",
            total_space.format_size(format_size_options),
            used_space.format_size(format_size_options),
            available_space.format_size(format_size_options),
            device_count
        );
        let paragraph = Paragraph::new(text);

        let percent = (used_space as f64 / total_space as f64) * 100.0;
        let gauge = Gauge::default()
            .percent(percent as u16)
            .gauge_style(get_color_for(percent));

        let top3_usage = Paragraph::new(self.disk_usage.clone());

        let read_bytes_sum = self
            .sysinfo
            .disks
            .disks
            .iter()
            .map(|s| s.bytes_read)
            .sum::<u64>();
        let written_bytes_sum = self
            .sysinfo
            .disks
            .disks
            .iter()
            .map(|s| s.bytes_written)
            .sum::<u64>();
        let io_format_opts = FormatSizeOptions::default()
            .base_unit(BaseUnit::Byte)
            .kilo(Kilo::Binary)
            .decimal_places(1)
            .long_units(false);
        let read_speed = (read_bytes_sum / 3).format_size(io_format_opts);
        let write_speed = (written_bytes_sum / 3).format_size(io_format_opts);
        let io_stat_text = format!("Read: {} /s Write: {} /s", read_speed, write_speed);
        let io_stat = Paragraph::new(io_stat_text);

        frame.render_widget(block, area);
        frame.render_widget(paragraph, disks_area[0]);
        frame.render_widget(gauge, disks_area[1]);
        frame.render_widget(top3_usage, disks_area[2]);
        frame.render_widget(io_stat, disks_area[3]);
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

    fn render_network_info(&self, frame: &mut Frame, area: Rect) {
        let network_area = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(&[Constraint::Fill(1)])
            .chunks(area);

        let block = Block::default()
            .title("Network")
            .borders(Borders::ALL)
            .border_type(tuirealm::props::BorderType::Rounded);

        let format_opts = FormatSizeOptions::default()
            .base_unit(BaseUnit::Byte)
            .kilo(Kilo::Binary)
            .decimal_places(1)
            .space_after_value(true)
            .long_units(false);

        let text = format!(
            "Interfaces: {}\nTotal received: {} Total packets received: {} Total errors on receive: {}\nTotal transmitted: {} Total packets transmitted: {} Total errors on transmitted: {}",
            self.sysinfo.network.interfaces,
            self.sysinfo.network.total_received.format_size(format_opts),
            self.sysinfo.network.total_packets_received,
            self.sysinfo.network.total_errors_on_received,
            self.sysinfo
                .network
                .total_transmitted
                .format_size(format_opts),
                self.sysinfo.network.total_packets_transmitted,
                self.sysinfo.network.total_errors_on_transmitted                
        );

        let paragraph = Paragraph::new(text).block(block);
        frame.render_widget(paragraph, network_area[0]);
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
