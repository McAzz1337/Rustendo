#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Target {
    A,
    B,
    C,
    D,
    E,
    F,
    L,
    H,
    HL,
    HLP,
    HLM,
    AF,
    BC,
    DE,
    R8,
    R16,
    D8,
    D16,
    A8,
    A16,
    SP,
    SP_R8,
}

impl Target {
    
    pub fn is_16bit(&self) -> bool {

        self == &Target::HL ||
        self == &Target::AF ||
        self == &Target::BC ||
        self == &Target::DE ||
        self == &Target::R16 ||
        self == &Target::D16 ||
        self == &Target::A16 ||
        self == &Target::SP_R8
    }
}