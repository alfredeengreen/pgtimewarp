pub struct Backpressure {
    max_queue: usize,
    current_queue: usize,
}

impl Backpressure {
    pub fn new(max_queue: usize) -> Self {
        Self {
            max_queue,
            current_queue: 0,
        }
    }

    pub fn should_degrade(&self) -> bool {
        self.current_queue >= self.max_queue
    }

    pub fn increment(&mut self) {
        self.current_queue += 1;
    }

    pub fn decrement(&mut self) {
        if self.current_queue > 0 {
            self.current_queue -= 1;
        }
    }
}
