use crate::util::*;
use std::{
    fs::File,
    io::{BufReader, Read, Write},
    path::Path,
    sync::{atomic::AtomicU8, Arc},
    thread,
    time::{Duration, Instant},
};

/// display dimensions
const DISPLAY_H: usize = 32;
const DISPLAY_W: usize = 64;

/// font is located at 0x050-0x09F
const FONT_LOAD_ADDR: usize = 0x50;

/// rom is located at 0x200-*
const ROM_LOAD_ADDR: usize = 0x200;

/// chip-8 specifications
const DISPLAY_SIZE: usize = DISPLAY_H * DISPLAY_W;
const STACK_SIZE: usize = 16;
const VREG_SIZE: usize = 16;
const RAM_SIZE: usize = 4096;

/// timing, instructions per second
const IPS: usize = 700;

/// render timing, frames per second
const FPS: usize = 60;

/// timers frequency, 60 Hz
const TIMERS_FREQ: usize = 60;

pub struct Chip8 {
    /// 64x32 display, 8-bit depth
    pub display: [u8; DISPLAY_SIZE],
    /// program counter
    pub pc: u16,
    /// index register
    pub ireg: u16,
    /// subroutine stack
    pub stack: [u16; STACK_SIZE],
    /// stack pointer
    pub sp: i8,
    /// variable registers
    pub vreg: [u8; VREG_SIZE],
    /// 4 kb of random access memory
    pub ram: [u8; RAM_SIZE],
    /// delay timer, decrements at 60 Hz rate
    pub delay_timer: Arc<AtomicU8>,
    /// sound timer - beep while non-zero, decrements at 60 Hz rate 
    pub sound_timer: Arc<AtomicU8>,
}

type EE = ExecError;

/// control flow
impl Chip8 {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, ExecError> {
        let mut device = Self {
            display: [0; DISPLAY_SIZE],
            pc: ROM_LOAD_ADDR as u16,
            ireg: 0,
            stack: [0; STACK_SIZE],
            sp: -1,
            vreg: [0; VREG_SIZE],
            ram: [0; RAM_SIZE],
            delay_timer: Arc::new(AtomicU8::new(0)),
            sound_timer: Arc::new(AtomicU8::new(0)),
        };
        let rom = Self::read_rom_from_file(path)?;
        device.load(rom, ROM_LOAD_ADDR)?;
        device.load(get_default_font(), FONT_LOAD_ADDR)?;

        Ok(device)
    }

    pub fn run(&mut self) -> Result<(), ExecError> {
        let time_per_instruction = Duration::from_secs(1) / IPS as u32;
        // start timer threads
        // start exit handler thread
        // optional: start display dimmer thread
        loop {
            let clock = Instant::now();
            // execute instruction cycle
            let inst = self.fetch()?;
            self.decode_and_execute(inst)?;
            // wait to meet timing
            let inst_time = clock.elapsed();
            if let Some(sleep_time) = time_per_instruction.checked_sub(inst_time) {
                thread::sleep(sleep_time);
            } else {
                println!("Instruction took longer than expected: {:#06x}", inst);
            }
        }
    }

    fn read_rom_from_file<P: AsRef<Path>>(path: P) -> Result<Vec<u8>, ExecError> {
        let file = File::open(path.as_ref()).map_err(|_| EE::LoadRomError)?;
        let mut reader = BufReader::new(file);
        let mut buffer = Vec::new();
        reader
            .read_to_end(&mut buffer)
            .map_err(|_| EE::LoadRomError)?;
        Ok(buffer)
    }

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
        // println!("Got instruction: {:#06x}", inst);
        match take_op(inst) {
            0x0 => match take_nnn(inst) {
                // clear screen
                0x0E0 => self.clear_display(),
                _ => {}
            },
            // jump
            0x1 => {
                self.pc = take_nnn(inst);
            }
            // set register vx
            0x6 => {
                *self
                    .vreg
                    .get_mut(take_x(inst) as usize)
                    .ok_or(EE::VRegOutOfBounds)? = take_nn(inst);
            }
            // add to register vx
            0x7 => {
                *self
                    .vreg
                    .get_mut(take_x(inst) as usize)
                    .ok_or(EE::VRegOutOfBounds)? += take_nn(inst);
            }
            // set index register
            0xa => {
                self.ireg = take_nnn(inst);
            }
            0xd => {
                let x_coord = *self
                    .vreg
                    .get(take_x(inst) as usize)
                    .ok_or(EE::VRegOutOfBounds)?;
                let y_coord = *self
                    .vreg
                    .get(take_y(inst) as usize)
                    .ok_or(EE::VRegOutOfBounds)?;
                let height = take_n(inst);
                self.draw_sprite(x_coord, y_coord, height)?;
            }
            _ => {
                println!("Unknown instruction: {:#06x}", inst);
            }
        }
        Ok(())
    }
}

/// memory manipulation
impl Chip8 {
    fn load<B: AsRef<[u8]>>(&mut self, bytes: B, offset: usize) -> Result<(), ExecError> {
        let bytes_ref = bytes.as_ref();
        self.ram
            .get_mut(offset..offset + bytes_ref.len())
            .ok_or(EE::MemoryError)?
            .write(bytes_ref)
            .map_err(|_| EE::RamError)?;
        Ok(())
    }

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
        // position sprite inside display
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

    /// flip state of pixel on screen, doesn't wrap around,
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
