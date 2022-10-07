//! Utilities
use macroquad::prelude::KeyCode;
use std::collections::HashMap;
use std::error::Error;
use std::fmt::Display;

pub const OP_MASK: u16 = 0xf000;
pub const X_MASK: u16 = 0x0f00;
pub const Y_MASK: u16 = 0x00f0;
pub const N_MASK: u16 = 0x000f;
pub const NN_MASK: u16 = 0x00ff;
pub const NNN_MASK: u16 = 0x0fff;

pub const FONT_CHAR_SIZE: usize = 5;

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
    KeymapError,
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
            Self::KeymapError => {
                write!(f, "Error while mapping key from instruction to keycode")
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Chip8Key {
    K0 = 0,
    K1,
    K2,
    K3,
    K4,
    K5,
    K6,
    K7,
    K8,
    K9,
    A,
    B,
    C,
    D,
    E,
    F,
}

impl From<u8> for Chip8Key {
    fn from(byte: u8) -> Self {
        // safe to unwrap, since all variants are covered
        byte.try_into().unwrap()
    }
}

#[rustfmt::skip]
pub fn get_default_keymap() -> HashMap<Chip8Key, KeyCode> {
    use KeyCode as MQ;
    use Chip8Key as C8;
    vec![
        (C8::K1, MQ::Key1), (C8::K2, MQ::Key2), (C8::K3, MQ::Key3), (C8::C, MQ::Key4),
        (C8::K4,    MQ::Q), (C8::K5,    MQ::W), (C8::K6,    MQ::E), (C8::D,    MQ::R),
        (C8::K7,    MQ::A), (C8::K8,    MQ::S), (C8::K9,    MQ::D), (C8::E,    MQ::F),
        (C8::A,     MQ::Z), (C8::K0,    MQ::X), (C8::B,     MQ::C), (C8::F,    MQ::V),
    ]
    .into_iter()
    .collect()
}

// FIXME: remove if not needed
// #[rustfmt::skip]
// pub fn get_default_keymap() -> HashMap<KeyCode, Chip8Key> {
//     use KeyCode as MQ;
//     use Chip8Key as C8;
//     vec![
//         (MQ::Key1, C8::K1), (MQ::Key2, C8::K2), (MQ::Key3, C8::K3), (MQ::Key4, C8::C),
//         (MQ::Q,    C8::K4), (MQ::W,    C8::K5), (MQ::E,    C8::K6), (MQ::R,    C8::D),
//         (MQ::A,    C8::K7), (MQ::S,    C8::K8), (MQ::D,    C8::K9), (MQ::F,    C8::E),
//         (MQ::Z,    C8::A ), (MQ::X,    C8::K0), (MQ::C,    C8::B ), (MQ::V,    C8::F),
//     ]
//     .into_iter()
//     .collect()
// }
