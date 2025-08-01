mod aws;

use aws_sdk_cloudformation::types::StackSummary;
use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use open;
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Direction, Layout},
    style::{Style, Stylize},
    text::Line,
    widgets::{Block, List, ListState},
};
use tokio::spawn;

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    let stack_handle = spawn(aws::get_stacks());
    let stacks = stack_handle.await.unwrap().unwrap();

    color_eyre::install()?;
    let terminal = ratatui::init();
    let result = App::new(&stacks).run(terminal);
    ratatui::restore();
    result
}

/// The main application which holds the state and logic of the application.
#[derive(Debug)]
pub struct App<'a> {
    running: bool,
    stack_list: StackList<'a>,
}

/// List of cloudformation stacks in the users default environment
#[derive(Debug, Clone)]
struct StackList<'a> {
    stacks: &'a Vec<StackSummary>,
    state: ListState,
}

impl<'a> App<'a> {
    /// Construct a new instance of [`App`].
    pub fn new(stacks: &'a Vec<StackSummary>) -> Self {
        Self {
            running: false,
            stack_list: StackList {
                stacks: stacks,
                state: ListState::default(),
            },
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
        let text = "An integration stack visualiser and configuration UI. \n\n\
            Press `Esc`, `Ctrl-C` or `q` to stop running.";

        let title_block = Block::new().title(title);
        let tagline = Line::from(text).centered();

        frame.render_widget(&title_block, layout[0]);
        frame.render_widget(tagline, title_block.inner(layout[0]));

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
            (_, KeyCode::Enter) => self.go_to_stack_link(),
            _ => {}
        }
    }

    /// Open a link to the selected stack in AWS Console
    fn go_to_stack_link(&mut self) {
        let index = self
            .stack_list
            .state
            .selected()
            .expect("Unable to find a selected index");

        let stack_arn = self.stack_list.stacks[index].stack_id().unwrap();

        let _ = open::that(format!(
            "https://console.aws.amazon.com/go/view?arn={}",
            stack_arn
        ));
    }

    /// Set running to false to quit the application.
    fn quit(&mut self) {
        self.running = false;
    }
}
