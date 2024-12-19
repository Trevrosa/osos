use core::{
    pin::Pin,
    task::{Context, Poll},
};

use conquer_once::spin::OnceCell;
use crossbeam_queue::ArrayQueue;
use futures_util::{task::AtomicWaker, Stream, StreamExt};
use log::{error, warn};
use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1};
use x86_64::instructions::interrupts::without_interrupts;

use crate::{print, vga::WRITER};

static WAKER: AtomicWaker = AtomicWaker::new();

pub struct ScancodeStream {
    _private: (),
}

impl ScancodeStream {
    pub fn new() -> Self {
        if SCANCODE_QUEUE
            .try_init_once(|| ArrayQueue::new(100))
            .is_err()
        {
            error!("scancode queue initialised twice");
        }
        ScancodeStream { _private: () }
    }
}

impl Default for ScancodeStream {
    fn default() -> Self {
        Self::new()
    }
}

static SCANCODE_QUEUE: OnceCell<ArrayQueue<u8>> = OnceCell::uninit();

pub fn add_scancode(scancode: u8) {
    if let Ok(queue) = SCANCODE_QUEUE.try_get() {
        if queue.push(scancode).is_err() {
            warn!("failed to push to scancode queue, full");
        } else {
            WAKER.wake();
        }
    } else {
        warn!("scancode queue not init");
    }
}

pub async fn print_keypresses() {
    let mut scancodes = ScancodeStream::new();
    let mut keyboard = Keyboard::new(
        ScancodeSet1::new(),
        layouts::Us104Key,
        HandleControl::Ignore,
    );

    while let Some(scancode) = scancodes.next().await {
        if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
            if let Some(key) = keyboard.process_keyevent(key_event) {
                match key {
                    // backspace
                    DecodedKey::Unicode(character) if character == 0x08 as char => {
                        without_interrupts(|| {
                            WRITER.lock().backspace();
                        });
                    }
                    DecodedKey::Unicode(character) => print!("{character}"),
                    DecodedKey::RawKey(_) => (),
                }
            }
        }
    }
}

impl Stream for ScancodeStream {
    type Item = u8;

    fn poll_next(self: Pin<&mut Self>, ctx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        #[allow(clippy::expect_used, reason = "cannot continue if queue not init")]
        let queue = SCANCODE_QUEUE.try_get().expect("scancode queue not init");

        if let Some(scancode) = queue.pop() {
            return Poll::Ready(Some(scancode));
        }

        WAKER.register(ctx.waker());
        if let Some(scancode) = queue.pop() {
            WAKER.take();
            Poll::Ready(Some(scancode))
        } else {
            Poll::Pending
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        // capacity is 100
        (0, Some(100))
    }
}
