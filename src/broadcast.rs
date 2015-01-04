use std::sync::{Mutex,Arc};
use std::sync::mpsc::{channel,Sender,Receiver};

#[derive(Clone)]
pub struct BroadcastStation<T: Send> {
    clients: Arc<Mutex<Vec<Sender<T>>>>
}

unsafe impl<T: Send> Sync for BroadcastStation<T> {}

impl<T: Send+Clone> BroadcastStation<T> {
    pub fn new() -> BroadcastStation<T> {
        BroadcastStation::<T> {
            clients: Arc::new(Mutex::new(Vec::new()))
        }
    }
    
    pub fn signup(&mut self) -> Receiver<T> {
        let (cast_tx, cast_rx) = channel();
        let mut clients = self.clients.lock().unwrap();
        clients.push(cast_tx);
        cast_rx
    }

    pub fn send(&mut self, msg: T) {
        let mut clients = self.clients.lock().unwrap();
        clients.retain(|client|
            client.send(msg.clone()).is_ok()
        );
    }

    #[allow(dead_code)]
    pub fn disconnect_all(&mut self) {
        let mut clients = self.clients.lock().unwrap();
        *clients = Vec::new();
    }
}