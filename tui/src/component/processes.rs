use core::model::{process_list_from_json, ProcessList};

use humansize::{BaseUnit, FormatSize, FormatSizeOptions};
use ratatui::{
    layout::{Alignment, Constraint, Flex},
    widgets::{Block, Cell, Row, Table},
};
use tuirealm::{
    command::{Cmd, CmdResult},
    ratatui::prelude::Rect,
    AttrValue, Attribute, Component, Event, Frame, MockComponent, NoUserEvent, Props, State,
};

use crate::Message;

/// Component for displaying process list in a table style.
#[derive(Default)]
pub struct Processes {
    properties: Props,

    list: ProcessList,
}

impl MockComponent for Processes {
    fn attr(&mut self, attr: Attribute, value: AttrValue) {
        if matches!(attr, Attribute::Value) {
            if let Some(json) = value.as_string() {
                if let Ok(process_list) = process_list_from_json(json) {
                    self.list = process_list;
                }
            }
        } else {
            self.properties.set(attr, value);
        }
    }

    fn perform(&mut self, _cmd: Cmd) -> CmdResult {
        CmdResult::None
    }

    fn query(&self, attribute: Attribute) -> Option<AttrValue> {
        self.properties.get(attribute)
    }

    fn state(&self) -> State {
        State::None
    }

    fn view(&mut self, frame: &mut Frame, area: Rect) {
        let opts = FormatSizeOptions::default()
            .base_unit(BaseUnit::Byte)
            .decimal_places(1)
            .decimal_zeroes(0)
            .kilo(humansize::Kilo::Binary)
            .long_units(false)
            .space_after_value(false);

        let header = Row::new(vec![
            Cell::from("pid"),
            Cell::from("name"),
            Cell::from("mem"),
            Cell::from("virtmem"),
            Cell::from("cpu"),
            Cell::from("cputime"),
            Cell::from("user"),
            Cell::from("runtime"),
            Cell::from("command"),
        ]);

        let rows: Vec<Row<'_>> = self
            .list
            .iter()
            .map(|process| {
                let cells = vec![
                    Cell::from(process.pid.to_string()),
                    Cell::from(process.name.clone()),
                    Cell::from(process.memory.format_size(opts)),
                    Cell::from(process.virtual_memory.format_size(opts)),
                    Cell::from(format!("{}%", process.cpu_usage.to_string())),
                    Cell::from(process.cpu_time.to_string()),
                    Cell::from(process.username.clone()),
                    Cell::from(process.running_time.to_string()),
                    Cell::from(process.command.clone()),
                ];
                Row::new(cells)
            })
            .collect();

        let table = Table::default()
            .block(
                Block::bordered()
                    .title("Processes")
                    .title_alignment(Alignment::Center),
            )
            .widths([
                Constraint::Length(6),
                Constraint::Fill(1),
                Constraint::Length(8),
                Constraint::Length(8),
                Constraint::Length(8),
                Constraint::Length(8),
                Constraint::Fill(1),
                Constraint::Length(8),
                Constraint::Fill(1),
            ])
            .header(header)
            .flex(Flex::Center)
            .rows(rows);

        frame.render_widget(table, area);
    }
}

impl Component<Message, NoUserEvent> for Processes {
    fn on(&mut self, _event: Event<NoUserEvent>) -> Option<Message> {
        None
    }
}
