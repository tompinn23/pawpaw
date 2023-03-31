use tokio::sync::watch::Receiver;

pub struct Cached<T> where T: Clone {
    value: T,
    updater: Receiver<T>
}

impl<T> Cached<T> where T: Clone {

    pub fn get(&mut self) -> &T {
        if self.updater.has_changed() {
            self.value = self.updater.borrow_and_update().clone();
        }
        &self.value
    }
}