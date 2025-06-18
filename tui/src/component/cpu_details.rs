use crate::Message;
use core::model::CpuCore;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Style, Stylize},
    symbols::Marker,
    widgets::{
        Axis, Bar, BarChart, BarGroup, Block, Borders, Chart, Dataset, GraphType, LegendPosition,
        Paragraph,
    },
};
use tuirealm::{
    command::{Cmd, CmdResult},
    event::{Key, KeyEvent},
    ratatui::prelude::Rect,
    AttrValue, Attribute, Component, Event, Frame, MockComponent, NoUserEvent, Props, State,
};

#[derive(Default)]
pub struct CpuMemoryDetails {
    properties: Props,

    /// Count of physical CPU cores
    core_count: usize,

    /// current frequency of the gobal CPU in Hz
    cpu_frequency: usize,

    /// Global temperature of the CPU in Celsius degree
    cpu_temperature: usize,

    /// Name of the CPU
    cpu_name: String,

    /// CPU load/usage over time in percent
    cpu_usage: Vec<(f64, f64)>,

    /// Indiviudal CPU core stats
    cpu_core_stats: Vec<CpuCore>,

    /// Physical memory usage over time in percent
    memory_usage: Vec<(f64, f64)>,

    /// Swap memory usage over time in percent
    swap_usage: Vec<(f64, f64)>,
}

impl MockComponent for CpuMemoryDetails {
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
        State::None
    }

    fn view(&mut self, frame: &mut Frame, area: Rect) {
        let block = Block::bordered();
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(&[
                Constraint::Percentage(33),
                Constraint::Percentage(33),
                Constraint::Percentage(33),
            ])
            .margin(1)
            .split(area);

        frame.render_widget(block, area);
        self.render_cpu_usage_chart(frame, layout[0]);
        self.render_core_details(frame, layout[1]);
        self.render_memory_details(frame, layout[2]);
    }
}

impl Component<Message, NoUserEvent> for CpuMemoryDetails {
    fn on(&mut self, event: Event<NoUserEvent>) -> Option<Message> {
        match event {
            Event::Keyboard(KeyEvent { code: Key::Tab, .. }) => Some(Message::ChangeNextMenu),
            Event::Keyboard(KeyEvent {
                code: Key::Backspace,
                ..
            }) => Some(Message::ChangePreviousMenu),
            _ => None,
        }
    }
}

impl CpuMemoryDetails {
    /// Sets the CPU name to be displayed.
    pub fn with_cpu_name(mut self, cpu_name: String) -> Self {
        self.cpu_name = cpu_name;
        self
    }

    /// Sets the CPU core count to be displayed.
    pub fn with_core_count(mut self, core_count: usize) -> Self {
        self.core_count = core_count;
        self.cpu_core_stats = Vec::with_capacity(core_count);
        self
    }

    /// Renders the CPU details in the left side of and an usage over time chart in the right side of the top third of the screen.
    fn render_cpu_usage_chart(&self, frame: &mut Frame, area: Rect) {
        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(&[Constraint::Percentage(25), Constraint::Fill(1)])
            .split(area);

        let usage = self.cpu_usage.first().unwrap_or(&(0.0, 0.0));
        let cpu_main_info = format!(
            "Name: {}\nCore count: {}\nUsage: {}%\nFrequency: {}\nTemperature: {}°C",
            self.cpu_name, self.core_count, usage.0, self.cpu_frequency, self.cpu_temperature
        );
        let cpu_label = Paragraph::new(cpu_main_info);

        //--- CPU Usage Over Time ---
        let percent_axis = Axis::default()
            .labels(vec![
                "0".green().bold(),
                "50".yellow().bold(),
                "100".red().bold(),
            ])
            // updates coming at every 3 seconds, keep only last 15 minutes
            .bounds([0.0, (15.0 * 60.0) / 3.0]);
        let time_axis = Axis::default()
            .labels(vec![
                "1m".gray().bold(),
                "5m".gray().bold(),
                "15m".gray().bold(),
            ])
            .bounds([0.0, 100.0]);

        let cpu_dataset = Dataset::default()
            .name("CPU Usage")
            .marker(Marker::Dot)
            .style(Style::default().light_green())
            .graph_type(GraphType::Scatter)
            .data(&self.cpu_usage);

        let cpu_chart = Chart::new(vec![cpu_dataset])
            .block(
                Block::bordered()
                    .title("CPU usage over time")
                    .bold()
                    .title_alignment(Alignment::Center),
            )
            .x_axis(time_axis)
            .y_axis(percent_axis)
            .legend_position(Some(LegendPosition::TopRight))
            .hidden_legend_constraints((Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)));

        frame.render_widget(cpu_label, layout[0]);
        frame.render_widget(cpu_chart, layout[1]);
    }

    fn render_core_details(&self, frame: &mut Frame, area: Rect) {
        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(&[Constraint::Fill(1); 4])
            .split(area);

        for (i, core) in self.cpu_core_stats.iter().enumerate() {
            let usage = core.usage;
            let usage_bar_color = match usage {
                usage if usage < 50 => Color::Green,
                x if x < 80 => Color::Yellow,
                _ => Color::Red,
            };

            let temp = core.temperature;
            let temp_bar_color = match temp {
                temp if temp < 50 => Color::Green,
                temp if temp < 80 => Color::Yellow,
                _ => Color::Red,
            };
            let bar_group = BarGroup::default()
                .label(format!("Core {}", i).into())
                .bars(&[
                    Bar::default()
                        .label("%".into())
                        .style(Style::default().fg(usage_bar_color))
                        .value(core.usage as u64),
                    Bar::default()
                        .label("f".into())
                        .value(core.frequency as u64),
                    Bar::default()
                        .label("t".into())
                        // FIXME: two character long label throws the label offset from the bar
                        //.label("°C".into())
                        .style(Style::default().fg(temp_bar_color))
                        .value(core.temperature as u64),
                ]);

            let bar_chart = BarChart::default()
                .bar_width(1)
                .bar_gap(1)
                .group_gap(1)
                .data(bar_group)
                .max(100);

            frame.render_widget(bar_chart, layout[i]);
        }
    }

    fn render_memory_details(&self, frame: &mut Frame, area: Rect) {
        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(&[Constraint::Percentage(25), Constraint::Fill(1)])
            .split(area);

        let memory_total_gb = 16;
        let memory_used_gb = 4;
        let memory_free_gb = memory_total_gb - memory_used_gb;
        let swap_total_gb = 4;
        let swap_used_gb = 1;
        let swap_free_gb = swap_total_gb - swap_used_gb;

        // --- Memory + Swap ---
        let mem_block = Block::default()
            .title("Memory / Swap")
            .borders(Borders::NONE);
        let mem_text = format!(
            "Total: {:.1} GB\nUsed: {:.1} GB\nFree: {:.1} GB\nSwap: {:.1} GB\nUsed swap: {:.1} GB\nFree swap: {:.1} GB",
            memory_total_gb, memory_used_gb, memory_free_gb, swap_used_gb, swap_total_gb, swap_free_gb
        );
        let mem_para = Paragraph::new(mem_text).block(mem_block);

        // --- Memory Usage Over Time ---
        let mem_dataset = Dataset::default()
            .name("Memory")
            .marker(Marker::Dot)
            .style(Style::default().magenta())
            .graph_type(GraphType::Scatter)
            .data(&self.memory_usage);
        let swap_dataset = Dataset::default()
            .name("Swap")
            .marker(Marker::Dot)
            .style(Style::default().yellow())
            .graph_type(GraphType::Scatter)
            .data(&self.swap_usage);
        let percent_axis = Axis::default()
            .labels(vec![
                "0".green().bold(),
                "50".yellow().bold(),
                "100".red().bold(),
            ])
            .bounds([0.0, 100.0]);
        let time_axis = Axis::default()
            .gray()
            .labels(vec![
                "1m".gray().bold(),
                "5m".gray().bold(),
                "15m".gray().bold(),
            ])
            // updates coming at every 3 seconds, keep only last 15 minutes
            .bounds([0.0, (15.0 * 60.0) / 3.0]);

        let mem_chart = Chart::new(vec![mem_dataset, swap_dataset])
            .block(
                Block::bordered()
                    .title("Memory & swap usage over time")
                    .title_alignment(Alignment::Center)
                    .bold()
                    .gray(),
            )
            .x_axis(time_axis)
            .y_axis(percent_axis)
            .legend_position(Some(LegendPosition::TopRight))
            .hidden_legend_constraints((Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)));

        frame.render_widget(mem_para, layout[0]);
        frame.render_widget(mem_chart, layout[1]);
    }
}
