use crossterm::event::{self, Event};
use std::time::Duration;

pub(crate) fn poll_event() -> Result<Option<Event>, Box<dyn std::error::Error>> {
    if !event::poll(Duration::from_millis(50))? {
        return Ok(None);
    }
    Ok(Some(event::read()?))
}
