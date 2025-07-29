mod aws;
mod views;
use views::stack_view::Stack;

use aws_sdk_cloudformation::types::StackSummary;
use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Direction, Layout},
    style::{Style, Stylize},
    text::Line,
    widgets::{Block, List, ListState, Widget},
};
use std::sync::Arc;
use tokio::runtime::Builder;

fn main() -> color_eyre::Result<()> {
    let runtime = Builder::new_multi_thread()
        .worker_threads(1)
        .enable_all()
        .build()
        .unwrap();
    let stack_handle = runtime.spawn(aws::get_stacks());
    let stacks = runtime.block_on(stack_handle).unwrap().unwrap();

    color_eyre::install()?;

    // TODO: Tokio and UI running on the same thread. Every call to AWS is blocking! FIX!
    let terminal = ratatui::init();
    let result = App::new(&stacks).run(terminal);
    ratatui::restore();
    result
}

/// List of cloudformation stacks in the users default environment
#[derive(Debug, Clone)]
struct StackList<'a> {
    stacks: &'a Vec<StackSummary>,
    state: ListState,
}

/// The main application which holds the state and logic of the application.
#[derive(Debug)]
pub struct App<'a> {
    running: bool,
    view: View,
    stack_list: StackList<'a>,
    current_stack: Option<&'a StackSummary>,
}

#[derive(Debug, Default, PartialEq)]
enum View {
    #[default]
    Stacks,
    Resources,
}

impl<'a> App<'a> {
    /// Construct a new instance of [`App`].
    pub fn new(stacks: &'a Vec<StackSummary>) -> Self {
        Self {
            running: false,
            view: View::Stacks,
            stack_list: StackList {
                stacks: stacks,
                state: ListState::default(),
            },
            current_stack: None,
        }
    }

    /// Run the application's main loop.
    pub fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        self.running = true;
        while self.running {
            terminal.draw(|frame| self.render(frame))?;
            self.handle_crossterm_events()?;
        }
        Ok(())
    }

    /// Renders the user interface.
    ///
    /// The render_widget function demands the right to destroy / alter memory through state
    /// handling. So we should pass clones instead of references to keep the actual data we grab
    /// from AWS intact (so that you can go back in the UI for example).
    fn render(&mut self, frame: &mut Frame) {
        // Define a layout for the UI and its elements
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Percentage(10), Constraint::Percentage(90)])
            .split(frame.area());

        let title = Line::from("Owl").bold().blue().centered();
        let text = "An integration stack visualiser and configuration UI. \n\n
            Press `Esc`, `Ctrl-C` or `q` to stop running. Press `Backspace` to navigate back.";

        let title_block = Block::new().title(title);
        let tagline = Line::from(text).centered();

        frame.render_widget(&title_block, layout[0]);
        frame.render_widget(tagline, title_block.inner(layout[0]));

        if self.view == View::Resources {
            let stack_view = Stack::new(Arc::new(self.current_stack.unwrap().clone()));
            stack_view.render(layout[1], frame.buffer_mut());
            return;
        }

        // Stack Menu
        frame.render_stateful_widget(
            List::new::<Vec<&str>>(
                self.stack_list
                    .stacks
                    .clone() // Pass a clone to prevent the list from taking ownership of the original AWS data
                    .iter()
                    .map(|stack| stack.stack_name().unwrap())
                    .collect(),
            )
            .block(Block::bordered().title(Line::from("CloudFormation Stacks").centered().bold()))
            .highlight_style(Style::new().blue().italic()),
            layout[1],
            &mut self.stack_list.state,
        );
    }

    /// Event handling using the crossterm backend. This handles all user input.
    fn handle_crossterm_events(&mut self) -> Result<()> {
        match event::read()? {
            // it's important to check KeyEventKind::Press to avoid handling key release events
            Event::Key(key) if key.kind == KeyEventKind::Press => self.on_key_event(key),
            Event::Mouse(_) => {}
            Event::Resize(_, _) => {}
            _ => {}
        }
        Ok(())
    }

    /// Handles the key events and updates the state of [`App`].
    fn on_key_event(&mut self, key: KeyEvent) {
        match (key.modifiers, key.code) {
            (_, KeyCode::Esc | KeyCode::Char('q'))
            | (KeyModifiers::CONTROL, KeyCode::Char('c') | KeyCode::Char('C')) => self.quit(),
            // Add other key handlers here.
            (_, KeyCode::Down) => self.stack_list.state.select_next(),
            (_, KeyCode::Up) => self.stack_list.state.select_previous(),
            (_, KeyCode::Enter) => self.select_stack(),
            (_, KeyCode::Backspace) => self.back(),
            _ => {}
        }
    }

    fn select_stack(&mut self) {
        let index = self
            .stack_list
            .state
            .selected()
            .expect("Unable to find a selected index");

        self.current_stack = Some(&self.stack_list.stacks[index]);
        self.view = View::Resources;
    }

    fn back(&mut self) {
        self.current_stack = None;
        self.view = View::Stacks;
    }

    /// Set running to false to quit the application.
    fn quit(&mut self) {
        self.running = false;
    }
}
