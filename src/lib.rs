#[cfg(windows)]
pub mod win32;

#[cfg(windows)]
pub use win32::key;

pub enum Event {
    Idle,
    Press(usize),
    Release(usize),
}
