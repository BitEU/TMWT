use anyhow::Result;
use crossterm::event::{self, Event as CrosstermEvent, KeyEvent, MouseEvent};
use std::{
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};

#[derive(Clone, Debug)]
pub enum Event {
    Tick,
    Key(KeyEvent),
    Mouse(MouseEvent),
    Resize(u16, u16),
}

pub struct EventHandler {
    sender: mpsc::Sender<Event>,
    receiver: mpsc::Receiver<Event>,
    handler: thread::JoinHandle<()>,
}

impl EventHandler {
    pub fn new(tick_rate: Duration) -> Self {
        let (sender, receiver) = mpsc::channel();
        let handler = {
            let sender = sender.clone();
            thread::spawn(move || {
                let mut last_tick = Instant::now();
                loop {
                    // Poll for events
                    let timeout = tick_rate
                        .checked_sub(last_tick.elapsed())
                        .unwrap_or_else(|| Duration::from_secs(0));
                    
                    if event::poll(timeout).expect("Unable to poll for events") {
                        match event::read().expect("Unable to read event") {
                            CrosstermEvent::Key(e) => {
                                if e.kind == event::KeyEventKind::Press {
                                    sender.send(Event::Key(e)).expect("Failed to send key event");
                                }
                            }
                            CrosstermEvent::Mouse(e) => {
                                sender.send(Event::Mouse(e)).expect("Failed to send mouse event");
                            }
                            CrosstermEvent::Resize(w, h) => {
                                sender.send(Event::Resize(w, h)).expect("Failed to send resize event");
                            }
                            _ => {}
                        }
                    }
                    
                    if last_tick.elapsed() >= tick_rate {
                        sender.send(Event::Tick).expect("Failed to send tick event");
                        last_tick = Instant::now();
                    }
                }
            })
        };
        
        Self {
            sender,
            receiver,
            handler,
        }
    }
    
    pub fn next(&self) -> Result<Event> {
        Ok(self.receiver.recv()?)
    }
}