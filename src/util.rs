//! Utilities

pub const OP_MASK: u16 = 0xf000;
pub const X_MASK: u16 = 0x0f00;
pub const Y_MASK: u16 = 0x00f0;
pub const N_MASK: u16 = 0x000f;
pub const NN_MASK: u16 = 0x00ff;
pub const NNN_MASK: u16 = 0x0fff;

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
