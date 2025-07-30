use crate::aws::get_stack_resources;
use aws_sdk_cloudformation::types::{StackResource, StackSummary};

use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    widgets::{Block, Clear, List, Widget},
};
use std::sync::Arc;
use tokio::runtime::Builder;

pub struct Stack {
    _stack_summary: Arc<StackSummary>,
    resources: Resources,
}

#[derive(Debug, Clone)]
struct Resources {
    lambdas: Vec<StackResource>,
    state_machines: Vec<StackResource>,
    apis: Vec<StackResource>,
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
        // It would actually be WAY better if cached this!
        let resources = runtime.block_on(handle).unwrap().unwrap();

        Self {
            _stack_summary: stack_summary,
            resources: Self::sort(resources),
        }
    }

    fn sort(resources: Vec<StackResource>) -> Resources {
        let mut sorted = Resources {
            lambdas: vec![],
            state_machines: vec![],
            apis: vec![],
        };

        resources.iter().for_each(|resource| -> () {
            match resource.resource_type() {
                Some("AWS::Lambda::Function") => sorted.lambdas.push(resource.clone()),
                Some("AWS::StepFunctions::StateMachine") => {
                    sorted.state_machines.push(resource.clone())
                }
                Some("AWS::ApiGatewayRestApi") => sorted.apis.push(resource.clone()),
                Some(_x) => (),
                None => (),
            }
        });

        return sorted;
    }

    fn render_list(
        &self,
        area: Rect,
        buf: &mut Buffer,
        resources: &Vec<StackResource>,
        title: &str,
    ) -> () {
        Widget::render(
            List::new::<Vec<&str>>(
                resources
                    .clone()
                    .iter()
                    .map(|resource| resource.logical_resource_id().unwrap())
                    .collect(),
            )
            .block(Block::bordered().title(title)),
            area,
            buf,
        );
    }
}

impl Widget for &Stack {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![
                Constraint::Percentage(33),
                Constraint::Percentage(33),
                Constraint::Percentage(33),
            ])
            .split(area);
        Clear.render(area, buf);

        self.render_list(layout[0], buf, &self.resources.lambdas, "Lambda Functions");
        self.render_list(
            layout[1],
            buf,
            &self.resources.state_machines,
            "State Machines",
        );
        self.render_list(
            layout[2],
            buf,
            &self.resources.apis,
            "API Gateway Rest APIs",
        )
    }
}
