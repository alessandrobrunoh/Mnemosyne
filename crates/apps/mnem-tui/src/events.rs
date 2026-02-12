use mnem_core::AppResult;
use crossterm::event::{self, Event, KeyEvent, MouseEvent};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

pub enum AppEvent {
    Input(KeyEvent),
    Mouse(MouseEvent),
    Tick,
}

pub struct EventHandler {
    rx: mpsc::Receiver<AppEvent>,
    _handle: thread::JoinHandle<()>,
}

impl EventHandler {
    pub fn new(tick_rate: Duration) -> Self {
        let (tx, rx) = mpsc::channel();
        let handle = thread::spawn(move || loop {
            match event::poll(tick_rate) {
                Ok(true) => match event::read() {
                    Ok(Event::Key(key)) => {
                        if tx.send(AppEvent::Input(key)).is_err() {
                            break;
                        }
                    }
                    Ok(Event::Mouse(mouse)) => {
                        if tx.send(AppEvent::Mouse(mouse)).is_err() {
                            break;
                        }
                    }
                    _ => {}
                },
                Ok(false) => {}
                Err(_) => {
                    // Prevent busy loop on persistent poll errors
                    thread::sleep(tick_rate);
                }
            }
            if tx.send(AppEvent::Tick).is_err() {
                break;
            }
        });
        Self {
            rx,
            _handle: handle,
        }
    }

    pub fn next(&self) -> AppResult<AppEvent> {
        self.rx
            .recv()
            .map_err(|_| mnem_core::AppError::Internal("Event channel closed".into()))
    }
}
