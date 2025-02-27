use core::{
    pin::Pin,
    task::{Context, Poll},
};

use conquer_once::spin::OnceCell;
use crossbeam_queue::ArrayQueue;
use futures_util::{task::AtomicWaker, Stream, StreamExt};
use log::{error, warn};
use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1};

use crate::{print, print_board, vga::WRITER};

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
                        WRITER.lock().backspace();
                    }
                    DecodedKey::Unicode(character) => print!("{character}"),
                    DecodedKey::RawKey(_) => (),
                }
            }
        }
    }
}

pub async fn tic_toe() {
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
                    DecodedKey::Unicode(character) if character == 0x08 as char => {
                        WRITER.lock().backspace();
                    }
                    DecodedKey::Unicode(character) => match character.to_digit(10) {
                        Some(d) => {
                            if d == 0 {
                                *crate::STATE.lock() = true;
                                crate::clear_board();
                            } else if d <= 3 {
                                let mut state = crate::STATE.lock();
                                let mut board = crate::BOARD.lock();
                                if board[0][d as usize - 1].is_none() {
                                    board[0][d as usize - 1] = Some(*state);
                                    *state = !*state;
                                }
                            } else if d <= 6 {
                                let mut state = crate::STATE.lock();
                                let mut board = crate::BOARD.lock();
                                if board[1][d as usize - 3- 1].is_none() {
                                    board[1][d as usize - 3 - 1] = Some(*state);
                                    *state = !*state;
                                }
                            } else if d <= 9 {
                                let mut state = crate::STATE.lock();
                                let mut board = crate::BOARD.lock();
                                if board[2][d as usize - 6 - 1].is_none() {
                                    board[2][d as usize - 6 - 1] = Some(*state);
                                    *state = !*state;
                                }
                            } else {
                                crate::print!("{d}");
                            }
                            print_board();
                        }
                        None => crate::print!("{character}"),
                    },
                    _ => (),
                }
            }
        }
    }
}

impl Stream for ScancodeStream {
    type Item = u8;

    fn poll_next(self: Pin<&mut Self>, ctx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
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
