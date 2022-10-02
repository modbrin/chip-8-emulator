use crate::util::*;
use std::{
    fs::File,
    io::{BufReader, Read, Write},
    ops::Deref,
    path::Path,
    sync::{
        atomic::{AtomicU8, Ordering},
        Arc, Mutex,
    },
    thread,
    time::{Duration, Instant},
};

/// display dimensions
pub const DISPLAY_H: usize = 32;
pub const DISPLAY_W: usize = 64;

/// font is located at 0x050-0x09F
pub const FONT_LOAD_ADDR: usize = 0x50;

/// rom is located at 0x200-*
pub const ROM_LOAD_ADDR: usize = 0x200;

/// chip-8 specifications
pub const DISPLAY_SIZE: usize = DISPLAY_H * DISPLAY_W;
pub const STACK_SIZE: usize = 16;
pub const VREG_SIZE: usize = 16;
pub const RAM_SIZE: usize = 4096;

/// timing, instructions per second
pub const IPS: usize = 700;

/// render timing, frames per second
pub const FPS: usize = 60;

/// timers frequency, 60 Hz
pub const TIMERS_FREQ: usize = 60;

pub const USE_VY_WHEN_SHIFING: bool = false; // TODO: should be a runtime setting
pub const BXNN_JUMP_WITH_OFFSET: bool = false; // TODO: should be a runtime setting

pub struct Chip8 {
    /// 64x32 display, 8-bit depth
    pub display: Arc<Mutex<[u8; DISPLAY_SIZE]>>,
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
            display: Arc::new(Mutex::new([0; DISPLAY_SIZE])),
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
                0x0EE => self.pc = self.stack_pop()?,
                _ => (),
            },
            // jump
            0x1 => {
                self.pc = take_nnn(inst);
            }
            // subroutine call
            0x2 => {
                self.stack_push(self.pc)?;
                self.pc = take_nnn(inst);
            }
            // conditional skip when vx equal nn
            0x3 => {
                if self.vx(inst)? == take_nn(inst) {
                    self.skip_inst();
                }
            }
            // conditional skip when vx not equal nn
            0x4 => {
                if self.vx(inst)? != take_nn(inst) {
                    self.skip_inst();
                }
            }
            // conditional skip when vx equal vy
            0x5 => {
                if take_n(inst) == 0 {
                    if self.vx(inst)? == self.vy(inst)? {
                        self.skip_inst();
                    }
                } else {
                    Self::unknown(inst);
                }
            }
            // set register vx to nn
            0x6 => {
                *self.vx_mut(inst)? = take_nn(inst);
            }
            // add nn to register vx, allow overflow
            0x7 => {
                let (val, _overflow) = self.vx(inst)?.overflowing_add(take_nn(inst));
                *self.vx_mut(inst)? = val;
            }
            // logical and arithmetic operations
            0x8 => {
                match take_n(inst) {
                    // vx = vy
                    0x0 => *self.vx_mut(inst)? = self.vy(inst)?,
                    // vx = vx OR vy
                    0x1 => *self.vx_mut(inst)? = self.vx(inst)? | self.vy(inst)?,
                    // vx = vx AND vy
                    0x2 => *self.vx_mut(inst)? = self.vx(inst)? & self.vy(inst)?,
                    // vx = vx XOR vy
                    0x3 => *self.vx_mut(inst)? = self.vx(inst)? ^ self.vy(inst)?,
                    // vx = vx + vy, set vf on overflow
                    0x4 => {
                        let (val, overflow) = self.vx(inst)?.overflowing_add(self.vy(inst)?);
                        *self.vx_mut(inst)? = val;
                        *self.vf_mut()? = if overflow { 0x1 } else { 0x0 };
                    }
                    // vx = vx - vy, unset vf on overflow
                    0x5 => {
                        let (val, underflow) = self.vx(inst)?.overflowing_sub(self.vy(inst)?);
                        *self.vx_mut(inst)? = val;
                        *self.vf_mut()? = if underflow { 0x0 } else { 0x1 };
                    }
                    // right shift
                    0x6 => {
                        if USE_VY_WHEN_SHIFING {
                            *self.vx_mut(inst)? = self.vy(inst)?;
                        }
                        let shifted_bit = self.vx(inst)? & 0x1;
                        *self.vx_mut(inst)? >>= 1;
                        *self.vf_mut()? = shifted_bit;
                    }
                    // vx = vy - vx, unset vf on overflow
                    0x7 => {
                        let (val, underflow) = self.vy(inst)?.overflowing_sub(self.vx(inst)?);
                        *self.vx_mut(inst)? = val;
                        *self.vf_mut()? = if underflow { 0x0 } else { 0x1 };
                    }
                    // left shift
                    0xe => {
                        if USE_VY_WHEN_SHIFING {
                            *self.vx_mut(inst)? = self.vy(inst)?;
                        }
                        let shifted_bit = self.vx(inst)? & LEFTMOST_BIT >> 7;
                        *self.vx_mut(inst)? <<= 1;
                        *self.vf_mut()? = shifted_bit;
                    }
                    _ => {
                        Self::unknown(inst);
                    }
                }
            }
            // conditional skip when vx not equal vy
            0x9 => {
                if take_n(inst) == 0 {
                    if self.vx(inst)? != self.vy(inst)? {
                        self.skip_inst();
                    }
                } else {
                    Self::unknown(inst);
                }
            }
            // set index register
            0xa => {
                self.ireg = take_nnn(inst);
            }
            // jump with offset
            0xb => {
                let offset = if BXNN_JUMP_WITH_OFFSET {
                    self.vx(inst)?
                } else {
                    self.vreg.get(0).copied().ok_or(EE::VRegOutOfBounds)?
                };
                let jump_to = take_nnn(inst).overflowing_add(offset as u16).0;
                self.pc = jump_to;
            }
            // random
            0xc => {
                *self.vx_mut(inst)? = rand::random::<u8>() & take_nn(inst);
            }
            // draw
            0xd => {
                let height = take_n(inst);
                self.draw_sprite(self.vx(inst)?, self.vy(inst)?, height)?;
            }
            _ => {
                Self::unknown(inst);
            }
        }
        Ok(())
    }

    /// skip one instruction
    fn skip_inst(&mut self) {
        self.pc += 2;
    }

    /// report unknown instruction encounter
    fn unknown(inst: u16) {
        println!("Unknown instruction: {:#06x}", inst);
    }
}

/// memory manipulation
impl Chip8 {
    /// copy `bytes` to ram given start memory offset
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

    /// shortcut for taking vx value
    fn vx(&self, inst: u16) -> Result<u8, ExecError> {
        self.vreg
            .get(take_x(inst) as usize)
            .copied()
            .ok_or(EE::VRegOutOfBounds)
    }

    /// shortcut for taking vy value
    fn vy(&self, inst: u16) -> Result<u8, ExecError> {
        self.vreg
            .get(take_y(inst) as usize)
            .copied()
            .ok_or(EE::VRegOutOfBounds)
    }

    // shortcut for taking vf value
    fn vf(&self) -> Result<u8, ExecError> {
        self.vreg
            .get(VF_REG_FLAG)
            .copied()
            .ok_or(EE::VRegOutOfBounds)
    }

    /// shortcut for taking vx mutable reference
    fn vx_mut(&mut self, inst: u16) -> Result<&mut u8, ExecError> {
        self.vreg
            .get_mut(take_x(inst) as usize)
            .ok_or(EE::VRegOutOfBounds)
    }

    /// shortcut for taking vy mutable reference
    fn vy_mut(&mut self, inst: u16) -> Result<&mut u8, ExecError> {
        self.vreg
            .get_mut(take_y(inst) as usize)
            .ok_or(EE::VRegOutOfBounds)
    }

    /// shortcut for taking vf mutable reference
    fn vf_mut(&mut self) -> Result<&mut u8, ExecError> {
        self.vreg.get_mut(VF_REG_FLAG).ok_or(EE::VRegOutOfBounds)
    }
}

// value for pixel being on, i.e. white
const PIXEL_ON: u8 = 0xff;
// value for pixel being just turned off, will be dimmed with time
const PIXEL_PRE_OFF: u8 = 0xfe;
// value for pixel being completely off, i.e. black
const PIXEL_OFF: u8 = 0x00;
// VF register address which is treated as flag
const VF_REG_FLAG: usize = 0x0f;
const LEFTMOST_BIT: u8 = 0b1000_0000;

// translate XY location to 1-dim array index
#[inline]
pub const fn loc_to_idx(x: usize, y: usize) -> usize {
    y * DISPLAY_W + x
}

#[inline]
pub const fn is_pixel_on(pixel: u8) -> bool {
    pixel == PIXEL_ON
}

/// display management
impl Chip8 {
    fn clear_display(&mut self) {
        self.display
            .lock()
            .unwrap()
            .iter_mut()
            .for_each(|pixel| *pixel = PIXEL_OFF);
    }

    fn get_pixel_value(&mut self, x: usize, y: usize) -> u8 {
        *self
            .display
            .lock()
            .unwrap()
            .get(loc_to_idx(x, y))
            .unwrap_or_else(|| &PIXEL_OFF)
    }

    fn draw_sprite(&mut self, x: u8, y: u8, h: u8) -> Result<(), ExecError> {
        *self.vf_mut()? = 0x00;
        // position sprite inside display
        let x = x as usize % DISPLAY_W;
        let y = y as usize % DISPLAY_H;
        // sprite is located at `ireg` memory address
        for line_i in 0..h as usize {
            let addr = self.ireg + line_i as u16;
            let line = *self.ram.get(addr as usize).ok_or(EE::MemoryError)?;
            for bit_i in 0..8usize {
                if (LEFTMOST_BIT >> bit_i) & line != 0 {
                    if self.flip_pixel(x + bit_i, y + line_i) {
                        *self.vf_mut()? = 0x01;
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
            .lock()
            .unwrap()
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

pub fn decrement_timers_routine(timers: Vec<Arc<AtomicU8>>) {
    let time_per_cycle = Duration::from_secs(1) / TIMERS_FREQ as u32;
    loop {
        let clock = Instant::now();
        // check and decrement timers
        for timer in timers.iter() {
            let mut old_t = timer.load(Ordering::Relaxed);
            loop {
                if old_t == 0 {
                    break;
                }
                match timer.compare_exchange_weak(
                    old_t,
                    old_t - 1,
                    Ordering::SeqCst,
                    Ordering::Relaxed,
                ) {
                    Ok(_) => break,
                    Err(x) => old_t = x,
                }
            }
        }
        // wait to meet timing
        let inst_time = clock.elapsed();
        if let Some(sleep_time) = time_per_cycle.checked_sub(inst_time) {
            thread::sleep(sleep_time);
        } else {
            println!("Timer doesn't meet timing");
        }
    }
}
