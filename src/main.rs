mod aws;

use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{
    DefaultTerminal, Frame,
    style::Stylize,
    text::Line,
    widgets::{Block, Borders, List, Paragraph},
};
use tokio::spawn;

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    let table_handle = spawn(aws::list_tables());
    let stack_handle = spawn(aws::get_stack());

    println!("Running table getter in a background thread...");
    let tables = table_handle.await.unwrap().unwrap(); // TODO Get rid of these gross ass unwraps!
    let _ = stack_handle.await;
    println!("{:?}", tables);

    Ok(())

    // color_eyre::install()?;
    // let terminal = ratatui::init();
    // let result = App::new(&tables).run(terminal);
    // ratatui::restore();
    // result
}

/// The main application which holds the state and logic of the application.
#[derive(Debug, Default)]
pub struct App {
    /// Is the application running?
    running: bool,
    tables: Vec<String>,
}

impl App {
    /// Construct a new instance of [`App`].
    pub fn new(tables: &Vec<String>) -> Self {
        Self {
            running: false,
            tables: tables.clone(), // TODO: I dont think this needs to be cloned but dont know
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
    ///
    /// This is where you add new widgets. See the following resources for more information:
    ///
    /// - <https://docs.rs/ratatui/latest/ratatui/widgets/index.html>
    /// - <https://github.com/ratatui/ratatui/tree/main/ratatui-widgets/examples>
    fn render(&mut self, frame: &mut Frame) {
        let title = Line::from("Owl").bold().blue().centered();
        let text = "A integration stack visualiser and configuration UI.\n\n\
            Press `Esc`, `Ctrl-C` or `q` to stop running.";
        frame.render_widget(
            List::new(self.tables.clone()).block(Block::bordered().title("DynamoDB Tables")),
            frame.area(),
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
