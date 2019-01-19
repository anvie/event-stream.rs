use std::{env, sync::Arc, thread::sleep, time::Duration};

extern crate event_stream;

use event_stream::{EventDispatcherBuilder, EventListener};

#[derive(Debug)]
enum Event {
    Ping,
    Hello(String),
}

struct MyListener;

impl MyListener {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {})
    }
}

impl EventListener<Event> for MyListener {
    fn dispatch(&self, event: &Event) {
        println!("{:?} got event: {:?}", self, event);
    }
}

impl std::fmt::Debug for MyListener {
    fn fmt(&self, out: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(out, "<MyListener>")
    }
}

fn main() {
    env::set_var("RUST_LOG", "simple=debug");
    env::set_var("RUST_BACKTRACE", "1");

    let event_dispatcher = EventDispatcherBuilder::new()
        .add_listener(MyListener::new())
        .build();

    sleep(Duration::from_secs(1));

    event_dispatcher.emit(Event::Ping);

    sleep(Duration::from_secs(1));

    event_dispatcher.emit(Event::Hello("world".to_string()));

    sleep(Duration::from_secs(1));

    println!("done.");
}
