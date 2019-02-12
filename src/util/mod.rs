pub mod event;

pub struct SelectLoop<T> {
    pub items: Vec<T>,
    pub index: usize,
}

impl<T> SelectLoop<T> {
    pub fn new(items: Vec<T>) -> SelectLoop<T> {
        SelectLoop { items, index: 0 }
    }

    pub fn next(&mut self) {
        self.index = (self.index + 1) % self.items.len();
    }

    pub fn previous(&mut self) {
        self.index = match self.index > 0 {
            true => self.index - 1,
            false => self.items.len() - 1,
        }
    }

    pub fn current(&self) -> &T {
        &self.items[self.index]
    }
}
