use std::collections::DList;
use std::sync::{Mutex,Arc};

#[deriving(Clone)]
pub struct BroadcastStation<T: Send> {
    clients: Arc<Mutex<DList<Sender<T>>>>
}

unsafe impl<T: Send> Sync for BroadcastStation<T> {}

impl<T: Send+Clone> BroadcastStation<T> {
    pub fn new() -> BroadcastStation<T> {
        BroadcastStation::<T> {
            clients: Arc::new(Mutex::new(DList::new()))
        }
    }
    
    pub fn signup(&mut self) -> Receiver<T> {
        let (cast_tx, cast_rx) = channel();
        let mut clients = self.clients.lock().unwrap();
        clients.push_back(cast_tx);
        cast_rx
    }

    pub fn send(&mut self, msg: T) {
        // This implementation is not satisfactory.
        // I'd rather avoid "client.clone()" and
        // just move the clients into the new list.
        let mut clients = self.clients.lock().unwrap();
        *clients = clients.iter().flat_map(|client| {
            client
                .send_opt(msg.clone())
                .ok()
                .map(move |_| client.clone())
                .into_iter()
        }).collect();
    }

    pub fn disconnect_all(&mut self) {
        let mut clients = self.clients.lock().unwrap();
        *clients = DList::new();
    }
}