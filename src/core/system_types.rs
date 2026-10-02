#[derive(
    Debug, 
    PartialEq,
    PartialOrd,
    Clone, 
    Copy
)]
pub(crate) struct Byte(
    pub(crate) u8
);

#[derive(
    Debug, 
    PartialEq,
    PartialOrd,
    Clone, 
    Copy
)]
pub(crate) struct Word(
    pub(crate) u16
);

pub(crate) enum SType{
    Byte(Byte),
    Word(Word)
}

impl Byte{
    pub(crate) fn get_high_nibble(&self) -> u8{
        self.0 >> 4
    }

    pub(crate) fn get_low_nibble(&self) -> u8{
        self.0 & 0x0F
    }

    pub(crate) fn get_bit(&self, idx: u8) -> u8{
        ( self.0 & ( 1 << idx ) ) >> idx 
    }
}

impl Word{
    pub(crate) fn from_bytes(hbyte: Byte, lbyte: Byte) -> Self{
        Self (
            ( ( hbyte.0 as u16 ) << 8 ) | lbyte.0 as u16         
        )
    }

    pub(crate) fn to_bytes(&self) -> (Byte, Byte){
        ( 
            Byte(((self.0 & 0xFF00) >> 8) as u8), 
            Byte((self.0 & 0x00FF) as u8)
        )
    }

}

impl SType{
    pub(crate) fn is_byte(&self) -> bool{
        if let SType::Byte(_) = self {true}
        else {false}
    }

    pub(crate) fn is_word(&self) -> bool{
        if let SType::Word(_) = self {true}
        else {false}
    }

    pub(crate) fn get_byte(&self) -> Byte{
        if let SType::Byte(byte) = self { *byte }
        else { panic!("ERROR: Trying to catch a Byte but found a Word") }
    }

    pub(crate) fn get_word(&self) -> Word{
        if let SType::Word(word) = self { *word }
        else { panic!("ERROR: Trying to catch a Word but found a Byte") }
    }
}
