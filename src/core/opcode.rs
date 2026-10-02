use crate::core::{
    opcode_maps::*, system::{RegisterSet, System}
};

#[derive(
    Debug,
    Clone, 
    Copy
)]
pub(crate) enum OperandType{
    Register{reg_set: RegisterSet, is_address: bool},
    U16{is_address: bool},
    U8{is_address: bool},
    I8{is_address: bool}
}



#[derive(
    Debug,
    Clone, 
    Copy
)]
pub(crate) enum OperandSet{
    Nullary,
    Binary(OperandType, OperandType),
    Unary(OperandType)
}

#[derive(
    Debug,
    Clone,
    Copy
)]
pub(crate) struct Opcode{
    pub(crate) mnemonic: &'static str, 
    pub(crate) raw: u8,
    pub(crate) size: u8,
    pub(crate) ncycles: f32,
    pub(crate) operand_type: OperandSet
}

impl OperandType{
    pub(crate) const fn from_bflag(val: u16) -> Self{  
        /* NOTE: This bitmask represent the 
         * max value covered by the registers
         * bitflags.
         */
        if val <= 0b1111111111
        {OperandType::Register(val)}

        else if val == 1<<10 {OperandType::Address}
        else if val == 1<<11 {OperandType::U16}
        else if val == 1<<12 {OperandType::U8}
        else if val == 1<<13 {OperandType::I8}
        else {OperandType::Union}
    }

    pub(crate) const fn to_bflag(&self) -> u16{
        match self{
            OperandType::Register(val) => *val,
            OperandType::Address => 1<<10,
            OperandType::U16 => 1<<11,
            OperandType::U8 => 1<<12,
            OperandType::I8 => 1<<13,
            OperandType::Union => panic!("ERROR: Union operand can't be converted directly to an operand")
        }
    }

    pub(crate) fn split_union(mut union: u16) -> Vec<Self>{
        let mut operands = Vec::new();
        const OPMAP: [OperandType;5] = [
            OperandType::Register(0),
            OperandType::Address,
            OperandType::U16,
            OperandType::U8,
            OperandType::I8
        ];

        while union != 0{
            
            for register in OPMAP {
                if ( 
                    register.to_bflag() & union
                ) == 0 {
                    continue;
                }

                union ^= register.to_bflag(); 

                operands.push(
                    register
                );
            }
        }
        operands
    }


}

#[macro_export]
macro_rules! operand_type_expansion {
    (($arg:expr)) => {
        OperandType::from_bflag(reg!($arg))
    };
    (($arg1:expr, $arg2:expr)) => {
        OperandType::from_bflag(reg!($arg1, $arg2))
    };
    ($op:expr) => {
        $op
    }
}

#[macro_export]
macro_rules! operand_type {
    (
        $( $op:tt ),* $(,)? 
    ) => {
        {
            let mut op_union: u16 = 0;

            $(
                op_union |= OperandType::to_bflag(&operand_type_expansion!($op));
            )*

            op_union
        }
    };
}

impl Opcode{
    pub(crate) fn new_unprefixed(
        system: &System
    ) -> Self{
        let opmap_entry: usize = system.wram[
            system.pc.0 as usize
        ] as usize;

        assert!(opmap_entry < 256);

        (OPCODE_MAP[opmap_entry]).unwrap()
    }

    pub(crate) fn new_prefixed(
        system: &System
    ) -> Self{
        let opmap_entry: usize = system.wram[
            system.pc.0 as usize
        ] as usize;
        assert!(opmap_entry < 256);
        PREFIXED_OPCODE_MAP[opmap_entry]
    }

}