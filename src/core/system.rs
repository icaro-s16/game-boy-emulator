const MEMORY_SIZE: u16 = 8_192;
const DISPLAY_HEIGHT: u16 = 144;
const DISPLAY_WIDGHT: u16 = 160;

pub struct GbSystem{
    memory: [u8; MEMORY_SIZE],
    display: [[u8; DISPLAY_WIDGHT]; DISPLAY_HEIGHT],
    AF: u16,
    BC: u16,
    DE: u16,
    HL: u16,
    SP: u16,
    PC: u16
}

struct Opcode{
    mnemonic: String, 
    raw_val: u8,
    operand: Option<u8>
}

impl Opcode{
    fn new(pc: u16) -> Self;
    fn exec(self, system: &mut GbSystem);
}
