use crate::aws::get_stack_resources;
use aws_sdk_cloudformation::types::{StackResource, StackSummary};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    widgets::{Block, Clear, Widget},
};

pub struct Stack<'a> {
    stack_summary: &'a StackSummary,
    resources: Vec<StackResource>,
}

impl<'a> Stack<'a> {
    pub async fn new(stack_summary: &'a StackSummary) -> Self {
        Self {
            stack_summary,
            resources: get_stack_resources(stack_summary).await.unwrap(),
        }
    }
}

impl<'a> Widget for &Stack<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        Clear.render(area, buf);
        Widget::render(
            Block::bordered().title(self.stack_summary.stack_name().unwrap()),
            area,
            buf,
        );
    }
}
