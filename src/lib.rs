#[macro_use]
extern crate crossbeam_channel as cc;

use cc::{unbounded, Sender, Receiver};

use std::{
    thread,
    sync::Arc
};

pub trait EventListener<T> 
where 
    T: Sync + Send + 'static
{
    fn dispatch(&self, event: &T);
}

type EL<T> = dyn EventListener<T> + Send + Sync;

pub struct EventDispatcher<T> {
    tx: Sender<T>,
    rx: Receiver<T>,
    listeners: Vec<Arc<EL<T>>>
}

pub struct EventDispatcherBuilder<T> {
    tx: Sender<T>,
    rx: Receiver<T>,
    listeners: Vec<Arc<EL<T>>>
}

impl<T> EventDispatcherBuilder<T>
where 
    T: Sync + Send + 'static
{
    pub fn new() -> Self {
        let (tx, rx) = unbounded::<T>();
        EventDispatcherBuilder { 
            tx, rx,
            listeners: vec![] 
        }
    }

    pub fn add_listener(&mut self, listener: Arc<EL<T>>) -> &mut Self {
        self.listeners.push(listener);
        self
    }

    pub fn build(&self) -> Arc<EventDispatcher<T>> {
        Arc::new(EventDispatcher {
            tx: self.tx.clone(), 
            rx: self.rx.clone(),
            listeners: self.listeners.clone()
        }.start())
    }

}

impl<T> EventDispatcher<T> 
where
    T: Sync + Send + 'static
{
    pub fn emit(&self, event: T){
        match self.tx.send(event){
            Err(e) => println!("error: {}", e),
            _ => ()
        }
    }

    pub fn start(self) -> Self {
        let rx_thread = self.rx.clone();
        let listeners = self.listeners.clone();
        thread::spawn(move || {
            loop {
                select! {
                    recv(rx_thread) -> msg => {
                        for listener in &listeners {
                            if let Ok(event) = &msg {
                                listener.dispatch(&event);
                            }
                        }
                    }
                }
            }
        });
        self
    }
}
