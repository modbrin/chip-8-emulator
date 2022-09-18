//! Utilities
use std::error::Error;
use std::fmt::Display;

pub const OP_MASK: u16 = 0xf000;
pub const X_MASK: u16 = 0x0f00;
pub const Y_MASK: u16 = 0x00f0;
pub const N_MASK: u16 = 0x000f;
pub const NN_MASK: u16 = 0x00ff;
pub const NNN_MASK: u16 = 0x0fff;

#[inline]
pub const fn take_op(inst: u16) -> u8 {
    ((inst & OP_MASK) >> 12) as u8
}

#[inline]
pub const fn take_x(inst: u16) -> u8 {
    ((inst & X_MASK) >> 8) as u8
}

#[inline]
pub const fn take_y(inst: u16) -> u8 {
    ((inst & Y_MASK) >> 4) as u8
}

#[inline]
pub const fn take_n(inst: u16) -> u8 {
    (inst & N_MASK) as u8
}

#[inline]
pub const fn take_nn(inst: u16) -> u8 {
    (inst & NN_MASK) as u8
}

#[inline]
pub const fn take_nnn(inst: u16) -> u16 {
    inst & NNN_MASK
}

#[derive(Clone, Copy, Debug)]
pub enum ExecError {
    VRegOutOfBounds,
    StackOverflow,
    StackUnderflow,
    MemoryError,
    DisplayOutOfBounds,
    LoadRomError,
    RamError,
}

impl Error for ExecError {}

impl Display for ExecError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::VRegOutOfBounds => {
                write!(f, "Variable register out of bounds")
            }
            Self::StackOverflow => {
                write!(f, "Stack overflow")
            }
            Self::StackUnderflow => {
                write!(f, "Stack underflow")
            }
            Self::MemoryError => {
                write!(f, "RAM access out of bounds")
            }
            Self::DisplayOutOfBounds => {
                write!(f, "Screen access/draw out of bounds")
            }
            Self::LoadRomError => {
                write!(f, "Error while loading ROM from file")
            }
            Self::RamError => {
                write!(f, "Error while writing data to RAM")
            }
        }
    }
}

pub fn get_default_font() -> Vec<u8> {
    vec![
        0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
        0x20, 0x60, 0x20, 0x20, 0x70, // 1
        0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
        0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
        0x90, 0x90, 0xF0, 0x10, 0x10, // 4
        0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
        0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
        0xF0, 0x10, 0x20, 0x40, 0x40, // 7
        0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
        0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
        0xF0, 0x90, 0xF0, 0x90, 0x90, // A
        0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
        0xF0, 0x80, 0x80, 0x80, 0xF0, // C
        0xE0, 0x90, 0x90, 0x90, 0xE0, // D
        0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
        0xF0, 0x80, 0xF0, 0x80, 0x80, // F
    ]
}
