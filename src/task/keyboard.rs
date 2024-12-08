use conquer_once::spin::OnceCell;
use crossbeam_queue::ArrayQueue;
use log::warn;

pub struct ScancodeStream {
    _private: (),
}

impl ScancodeStream {
    pub fn new() -> Self {
        SCANCODE_QUEUE
            .try_init_once(|| ArrayQueue::new(100))
            .expect("should only be called once");
        ScancodeStream { _private: () }
    }
}

static SCANCODE_QUEUE: OnceCell<ArrayQueue<u8>> = OnceCell::uninit();

pub fn add_scancode(scancode: u8) {
    if let Ok(queue) = SCANCODE_QUEUE.try_get() {
        if queue.push(scancode).is_err() {
            warn!("failed to push to scancode queue");
        }
    } else {
        warn!("scancode queue not init");
    }
}
