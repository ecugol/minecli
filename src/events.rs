use crossterm::event::{self, Event as CrosstermEvent, KeyEvent, KeyEventKind, MouseEvent};
use std::time::Duration;

pub enum Event {
    Key(KeyEvent),
    Mouse(MouseEvent),
    Tick,
}

pub struct EventHandler;

impl EventHandler {
    pub fn new() -> Self {
        Self
    }

    pub fn next(&self) -> anyhow::Result<Event> {
        if event::poll(Duration::from_millis(250))? {
            match event::read()? {
                CrosstermEvent::Key(key) => {
                    if key.kind == KeyEventKind::Press {
                        return Ok(Event::Key(key));
                    }
                }
                CrosstermEvent::Mouse(mouse) => {
                    return Ok(Event::Mouse(mouse));
                }
                _ => {}
            }
        }
        Ok(Event::Tick)
    }
}
