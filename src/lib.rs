#[macro_use]
extern crate crossbeam_channel;

use crossbeam_channel::{unbounded, Receiver, Sender};

use std::{sync::Arc, thread};

pub trait EventListener<T>
where
    T: Sync + Send,
{
    fn dispatch(&self, _event: &T) {}
}

pub struct EventDispatcher<T> {
    tx: Sender<T>,
    rx: Receiver<T>,
    listeners: Vec<Arc<dyn EventListener<T> + Send + Sync>>,
}

pub struct EventDispatcherBuilder<T> {
    tx: Sender<T>,
    rx: Receiver<T>,
    listeners: Vec<Arc<dyn EventListener<T> + Send + Sync>>,
}

impl<T> EventDispatcherBuilder<T>
where
    T: Sync + Send + Sized,
{
    pub fn new() -> Self {
        let (tx, rx) = unbounded::<T>();
        EventDispatcherBuilder {
            tx,
            rx,
            listeners: vec![],
        }
    }

    pub fn add_listener(mut self, listener: Arc<dyn EventListener<T> + Send + Sync>) -> Self {
        self.listeners.push(listener);
        self
    }

    pub fn build(self) -> EventDispatcher<T> {
        EventDispatcher {
            tx: self.tx.clone(),
            rx: self.rx.clone(),
            listeners: self.listeners,
        }
    }
}

impl<T> EventDispatcher<T>
where
    T: Sized + Send + Sync + 'static,
{
    pub fn emit(&self, event: T) {
        let _ = self.tx.send(event).expect("cannot send via channel");
    }

    pub fn start(&self) {
        let rx_thread = self.rx.clone();
        let listeners = self.listeners.clone();
        let mut tried = 0;

        thread::spawn(move || loop {
            let rx_thread_c = rx_thread.clone();
            let listeners_c = listeners.clone();
            let h = thread::spawn(move || loop {
                select! {
                    recv(rx_thread_c) -> msg => {
                        for listener in listeners_c.iter() {
                            if let Ok(event) = &msg {
                                listener.dispatch(&event);
                            }
                        }
                    }
                }
            });

            let r = h.join();
            match r {
                Ok(r) => eprintln!("event dispatcher thread ended! {:?}", r),
                Err(e) => {
                    eprintln!("event dispatcher thread died! {:?}", e);
                }
            }

            eprintln!("Starting new event dispatcher thread again...");
            tried = tried + 1;
            if tried >= 100 {
                eprintln!(
                    "event dispatcher thread cannot be started after {} attempts!",
                    tried
                );
                break;
            }
        });
    }

    pub fn get_channel(&self) -> &Sender<T> {
        &self.tx
    }
}
