mod aws;

use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Direction, Layout},
    style::Stylize,
    text::Line,
    widgets::{Block, List},
};
use tokio::spawn;

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    let stack_handle = spawn(aws::get_stacks());

    let stacks = stack_handle.await.unwrap().unwrap();
    let stack_names: Vec<&str> = stacks
        .iter()
        .map(|stack| stack.stack_name().unwrap())
        .collect();

    color_eyre::install()?;
    let terminal = ratatui::init();
    let result = App::new(&stack_names).run(terminal);
    ratatui::restore();
    result
}

/// The main application which holds the state and logic of the application.
#[derive(Debug, Default)]
pub struct App<'a> {
    /// Is the application running?
    running: bool,
    stacks: Vec<&'a str>,
}

impl<'a> App<'a> {
    /// Construct a new instance of [`App`].
    pub fn new(stacks: &Vec<&'a str>) -> Self {
        Self {
            running: false,
            stacks: stacks.clone(), // TODO: I dont think this needs to be cloned but dont know
                                    // enough about lifetimes to fix it.
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
        frame.render_widget(
            List::new(self.stacks.clone()).block(
                Block::bordered().title(Line::from("CloudFormation Stacks").centered().bold()),
            ),
            layout[1],
        );
    }

    /// Reads the crossterm events and updates the state of [`App`].
    ///
    /// If your application needs to perform work in between handling events, you can use the
    /// [`event::poll`] function to check if there are any events available with a timeout.
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
            _ => {}
        }
    }

    /// Set running to false to quit the application.
    fn quit(&mut self) {
        self.running = false;
    }
}
