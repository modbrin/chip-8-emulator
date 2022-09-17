use crate::util::*;
use std::error::Error;
use std::fmt::Display;
use std::sync::{atomic::AtomicU8, Arc};

const DISPLAY_H: usize = 32;
const DISPLAY_W: usize = 64;

pub struct Chip8 {
    /// 64x32 display, 8-bit depth
    pub display: [u8; DISPLAY_H * DISPLAY_W],
    /// program counter
    pub pc: u16,
    /// index register
    pub ireg: u16,
    /// subroutine stack
    pub stack: [u16; 16],
    /// stack pointer
    pub sp: i8,
    /// variable registers
    pub vreg: [u8; 16],
    /// 4 kb of random access memory
    pub ram: [u8; 4096],
    /// delay timer
    pub delay_timer: Arc<AtomicU8>,
    /// sound timer
    pub sound_timer: Arc<AtomicU8>,
}

#[derive(Clone, Debug)]
pub enum ExecError {
    VRegOutOfBounds,
    StackOverflow,
    StackUnderflow,
    MemoryError,
    DisplayOutOfBounds,
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
        }
    }
}

type EE = ExecError;

/// control flow
impl Chip8 {
    pub fn initialize() {}

    pub fn load_rom() {}

    pub fn run() {}

    fn fetch(&mut self) -> Result<u16, ExecError> {
        let a = *self.ram.get(self.pc as usize).ok_or(EE::MemoryError)?;
        let b = *self
            .ram
            .get((self.pc + 1) as usize)
            .ok_or(EE::MemoryError)?;
        self.pc += 2;
        Ok(((a as u16) << 8) | (b as u16))
    }

    fn decode_and_execute(&mut self, inst: u16) -> Result<(), ExecError> {
        match inst & OP_MASK {
            0x_0_000 => match inst & NNN_MASK {
                // clear screen
                0x0_0E0_ => self.clear_display(),
                _ => {}
            },
            // jump
            0x_1_000 => {
                self.pc = inst & NNN_MASK;
            }
            // set register vx
            0x_6_000 => {
                *self
                    .vreg
                    .get_mut(take_x(inst) as usize)
                    .ok_or(EE::VRegOutOfBounds)? = take_nn(inst);
            }
            // add to register vx
            0x_7_000 => {
                *self
                    .vreg
                    .get_mut(take_x(inst) as usize)
                    .ok_or(EE::VRegOutOfBounds)? += take_nn(inst);
            }
            // set index register
            0x_a_000 => {
                self.ireg = take_nnn(inst);
            }
            0x_d_000 => {
                let x_coord = *self
                    .vreg
                    .get(take_x(inst) as usize)
                    .ok_or(EE::VRegOutOfBounds)?;
                let y_coord = *self
                    .vreg
                    .get(take_y(inst) as usize)
                    .ok_or(EE::VRegOutOfBounds)?;
                let height = take_n(inst);
                self.draw_sprite(x_coord, y_coord, height);
            }
            _ => {}
        }
        Ok(())
    }
}

/// memory manipulation
impl Chip8 {
    fn stack_push(&mut self, val: u16) -> Result<(), ExecError> {
        *self
            .stack
            .get_mut((self.sp + 1) as usize)
            .ok_or(EE::StackOverflow)? = val;
        self.sp += 1;
        Ok(())
    }

    fn stack_pop(&mut self) -> Result<u16, ExecError> {
        if self.sp >= 0 {
            let val = *self.stack.get(self.sp as usize).ok_or(EE::StackOverflow)?;
            self.sp -= 1;
            Ok(val)
        } else {
            Err(EE::StackUnderflow)
        }
    }
}

const PIXEL_ON: u8 = 0xff;
const PIXEL_OFF: u8 = 0x00;
const VF_REG_FLAG: usize = 0x0f;

// translate XY location to 1-dim array index
#[inline]
pub const fn loc_to_idx(x: usize, y: usize) -> usize {
    y * DISPLAY_W + x
}

/// display management
impl Chip8 {
    fn clear_display(&mut self) {
        self.display.iter_mut().for_each(|pixel| *pixel = PIXEL_OFF);
    }

    fn get_pixel_value(&mut self, x: usize, y: usize) -> u8 {
        *self
            .display
            .get(loc_to_idx(x, y))
            .unwrap_or_else(|| &PIXEL_OFF)
    }

    fn set_vf(&mut self, val: u8) -> Result<(), ExecError> {
        *self.vreg.get_mut(VF_REG_FLAG).ok_or(EE::VRegOutOfBounds)? = val;
        Ok(())
    }

    fn draw_sprite(&mut self, x: u8, y: u8, h: u8) -> Result<(), ExecError> {
        self.set_vf(0x00)?;
        // position sprite inside window
        let x = x as usize % DISPLAY_W;
        let y = y as usize % DISPLAY_H;
        // sprite is located at `ireg` memory address
        for line_i in 0..h as usize {
            let addr = self.ireg + line_i as u16;
            let line = *self.ram.get(addr as usize).ok_or(EE::MemoryError)?;
            for bit_i in (0..8usize).rev() {
                if (0x01 << bit_i) & line > 0 {
                    if self.flip_pixel(x + bit_i, y + line_i) {
                        self.set_vf(0x01)?;
                    }
                }
            }
        }
        Ok(())
    }

    /// flip state of pixel on screen, doesn't wrap around
    /// return `true` if pixel was turned off
    fn flip_pixel(&mut self, x: usize, y: usize) -> bool {
        self.display
            .get_mut(loc_to_idx(x, y))
            .map(|p| {
                let was_on = *p == PIXEL_ON;
                if was_on {
                    *p = PIXEL_OFF;
                } else {
                    *p = PIXEL_ON;
                }
                was_on
            })
            .unwrap_or_else(|| false)
    }
}
