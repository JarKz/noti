use std::path::Path;

use widgets::widget::WidgetEnum;
mod parser;

mod analysis;
mod ast;
pub mod plugin;
mod runtime_data;

pub struct NotiDsl {}

pub fn parse_layout(path: &Path) -> anyhow::Result<WidgetEnum> {
    let _data = std::fs::read_to_string(path)?;
    todo!()
    // let pairs = parser::parse(&data)?;
    // converter::convert_into_widgets(pairs)
}
