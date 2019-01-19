#[macro_use]
extern crate crossbeam_channel;

use crossbeam_channel::{unbounded, Receiver, Sender};

use std::{sync::Arc, thread};

pub trait EventListener<T>
where
    T: Sync + Send,
{
    fn dispatch(&self, event: &T);
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
        match self.tx.send(event) {
            Err(e) => println!("error: {}", e),
            _ => (),
        }
    }

    pub fn start(&self) {
        let rx_thread = self.rx.clone();
        let listeners = self.listeners.clone();

        thread::spawn(move || loop {
            select! {
                recv(rx_thread) -> msg => {
                    for listener in listeners.iter() {
                        if let Ok(event) = &msg {
                            listener.dispatch(&event);
                        }
                    }
                }
            }
        });
    }

    pub fn get_channel(&self) -> &Sender<T> {
        &self.tx
    }
}
