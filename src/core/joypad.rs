use sdl2::{
    EventPump,
    event::Event, 
    keyboard::Keycode
};

use crate::core::{
    cpu::{
        IF_ADDR,
        JOYPAD_INTERRUPT_BIT
    }, 
    memory::Memory
};


const P1_ADDR: usize = 0xFF00;
const SELECTB_BIT: u8 = 0x5;
const SELECTD_BIT: u8 = 0x4;

const A_BIT: u8 = 0x0;
const B_BIT: u8 = 0x1;
const SELECT_BIT: u8 = 0x2;
const START_BIT: u8 = 0x3;

const RIGHT_BIT: u8 = 0x0;
const LEFT_BIT: u8 = 0x1;
const UP_BIT: u8 = 0x2;
const DOWN_BIT: u8 = 0x3;


pub struct KeyMap{
    start: Keycode,
    select: Keycode,
    b: Keycode,
    a: Keycode,
    down: Keycode,
    up: Keycode,
    left: Keycode,
    right: Keycode
}

pub struct JoyPad{
    bkeys: u8,
    dkeys: u8,
    kmap: KeyMap,
    event_pump: EventPump
}

impl KeyMap {
    pub fn new (
        start: Keycode,
        select: Keycode,
        b: Keycode,
        a: Keycode,
        down: Keycode,
        up: Keycode,
        left: Keycode,
        right: Keycode
    ) -> Self{
        Self { 
            start, 
            select, 
            b, 
            a, 
            down, 
            up, 
            left, 
            right 
        }
    }
}

impl Default for KeyMap{
    fn default() -> Self {
        Self { 
            start: Keycode::Q, 
            select: Keycode::W, 
            b: Keycode::B, 
            a: Keycode::A, 
            down: Keycode::DOWN, 
            up: Keycode::UP, 
            left: Keycode::LEFT, 
            right:Keycode::RIGHT 
        }
    }
}


impl JoyPad {
    pub fn new(kmap: KeyMap, event_pump: EventPump) -> Self{
        Self{
            dkeys: 0xF,
            bkeys: 0xF,
            kmap,
            event_pump
        }
    }

    pub fn send_interrupt(
        keys: u8,
        memory: &mut Memory
    ) {
        let last_keys =
            memory[P1_ADDR] & 0xF;

        if 
            ((last_keys & 1) != 0 &&
            (keys & 1) == 0) ||
            ((last_keys & (1 << 1)) != 0 &&
            (keys & (1 << 1)) == 0) ||
            ((last_keys & (1 << 2)) != 0 &&
            (keys & (1 << 2)) == 0) ||
            ((last_keys & (1 << 3)) != 0 &&
            (keys & (1 << 3)) == 0) 
        {
            memory[IF_ADDR] |= 1 << JOYPAD_INTERRUPT_BIT;
        } 
    }

    pub fn update(
        &mut self, 
        memory: &mut Memory
    ) -> bool{
        let event = 
            if let Some(event) = self.event_pump.poll_event(){
                event
            }
            else {
                return false;
            };

        match event {
            Event::Quit { .. } => return true,

            Event::KeyUp{ keycode: Some(key), ..} => {
                self.dkeys = 0xF;
                self.bkeys = 0xF;

                if key == self.kmap.select {
                    self.bkeys &= !(1 << SELECT_BIT);
                }
                if key == self.kmap.start {
                    self.bkeys &= !(1 << START_BIT);
                }
                if key == self.kmap.a {
                    self.bkeys &= !(1 << A_BIT);
                }
                if key == self.kmap.b {
                    self.bkeys &= !(1 << B_BIT);
                }
                if key == self.kmap.right {
                    self.dkeys &= !(1 << RIGHT_BIT);
                }
                if key == self.kmap.left {
                    self.dkeys &= !(1 << LEFT_BIT);
                }
                if key == self.kmap.up {
                    self.dkeys &= !(1 << UP_BIT);
                }
                if key == self.kmap.down {
                    self.dkeys &= !(1 << DOWN_BIT);
                }

                if 
                    (memory[P1_ADDR] & (1 << SELECTB_BIT)) == 0 || 
                    (memory[P1_ADDR] & (1 << SELECTD_BIT)) == 0
                {   
                    let mut keys = 0xF;
                    if (memory[P1_ADDR] & (1 << SELECTB_BIT)) == 0 {
                        keys &= self.bkeys;
                    }
                    if (memory[P1_ADDR] & (1 << SELECTD_BIT)) == 0 {
                        keys &= self.dkeys;
                    }
                    JoyPad::send_interrupt(keys, memory);
                    memory[P1_ADDR] |= keys; 
                }
            
            },
            Event::KeyDown{ keycode: Some(key), ..} => {
                self.bkeys = 0;
                self.dkeys = 0;

                if key == self.kmap.select {
                    self.bkeys |= 1 << SELECT_BIT;
                }
                if key == self.kmap.start {
                    self.bkeys |= 1 << START_BIT;
                }
                if key == self.kmap.a {
                    self.bkeys |= 1 << A_BIT;
                }
                if key == self.kmap.b {
                    self.bkeys |= 1 << B_BIT;
                }
                if key == self.kmap.right {
                    self.dkeys |= 1 << RIGHT_BIT;
                }
                if key == self.kmap.left {
                    self.dkeys |= 1 << LEFT_BIT;
                }
                if key == self.kmap.up {
                    self.dkeys |= 1 << UP_BIT;
                }
                if key == self.kmap.down {
                    self.dkeys |= 1 << DOWN_BIT;
                }

                if 
                    (memory[P1_ADDR] & (1 << SELECTB_BIT)) == 0 || 
                    (memory[P1_ADDR] & (1 << SELECTD_BIT)) == 0
                {   
                    let mut keys = 0xF;

                    if (memory[P1_ADDR] & (1 << SELECTB_BIT)) == 0 {
                        keys &= self.bkeys;
                    }
                    if (memory[P1_ADDR] & (1 << SELECTD_BIT)) == 0 {
                        keys &= self.dkeys;
                    }

                    memory[P1_ADDR] |= keys; 
                }
            },
            _ => {}
        }
        

        false
    }    
}