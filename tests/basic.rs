use event_stream::{EventDispatcherBuilder, EventListener};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use std::{thread, time::Duration};

#[derive(Debug, Clone, PartialEq)]
enum Event {
    Ping,
    Hello(String),
}

#[derive(Debug)]
struct MyListener {
    pub count: AtomicUsize,
}

impl MyListener {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            count: AtomicUsize::new(0),
        })
    }
}

impl EventListener<Event> for MyListener {
    fn dispatch(&self, event: &Event) {
        println!("{:?} oh event: {:?}", self, event);
        self.count.fetch_add(1, Ordering::Relaxed);
    }
}

#[test]
fn test_basic() {
    let my_listener = MyListener::new();
    let event_dispatcher = EventDispatcherBuilder::new()
        .add_listener(my_listener.clone())
        .build();

    event_dispatcher.start();

    // give some time
    thread::sleep(Duration::from_millis(10));

    event_dispatcher.emit(Event::Ping);
    event_dispatcher.emit(Event::Hello("test".to_owned()));

    thread::sleep(Duration::from_millis(1000));

    assert_eq!(my_listener.count.load(Ordering::Relaxed), 2);
}
