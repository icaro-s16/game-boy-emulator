const MEMORY_SIZE: u16 = 1024 * 8;
const DISPLAY_HEIGHT: u16 = 144;
const DISPLAY_WIDGHT: u16 = 160;

pub struct GbSystem{
    wram: [u8; MEMORY_SIZE],
    vram: [u8; MEMORY_SIZE],
    display: [[u8; DISPLAY_WIDGHT]; DISPLAY_HEIGHT],
    A: u8,
    F: u8,
    B: u8,
    C: u8,
    D: u8,
    E: u8,
    H: u8,
    L: u8,
    SP: u16,
    PC: u16
}

struct Opcode{
    mnemonic: String, 
    raw_val: u16,
    operand: Option<u8>
}

impl Opcode{
    fn new(pc: u16) -> Self{
        match(pc){

        }
    }
    fn exec(self, system: &mut GbSystem);
}
