use crate::aws::get_stack_resources;
use aws_sdk_cloudformation::types::{StackResource, StackSummary};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    widgets::{Block, Clear, List, Widget},
};
use std::sync::Arc;
use tokio::runtime::Builder;

pub struct Stack {
    stack_summary: Arc<StackSummary>,
    resources: Vec<StackResource>,
}

impl Stack {
    pub fn new(stack_summary: Arc<StackSummary>) -> Self {
        let runtime = Builder::new_multi_thread()
            .worker_threads(1)
            .enable_all()
            .build()
            .unwrap();

        let handle = runtime.spawn(get_stack_resources(stack_summary.clone()));
        // TODO: This is still blocking the main UI thread from rendering. Need to figure out a way
        // to render an empty list, and then re-render one the runtime has grabbed what it needs
        // from AWS
        let resources = runtime.block_on(handle).unwrap().unwrap();
        Self {
            stack_summary,
            resources,
        }
    }
}

impl Widget for &Stack {
    fn render(self, area: Rect, buf: &mut Buffer) {
        Clear.render(area, buf);
        Widget::render(
            List::new::<Vec<&str>>(
                self.resources
                    .clone()
                    .iter()
                    .map(|resource| resource.logical_resource_id().unwrap())
                    .collect(),
            )
            .block(Block::bordered().title(self.stack_summary.stack_name().unwrap())),
            area,
            buf,
        );
    }
}
