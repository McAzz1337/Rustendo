use std::fmt::Display;

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
        self == &Target::HL
            || self == &Target::AF
            || self == &Target::BC
            || self == &Target::DE
            || self == &Target::R16
            || self == &Target::D16
            || self == &Target::A16
            || self == &Target::SP_R8
    }
}

impl Display for Target {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::A => "A",
            Self::B => "B",
            Self::C => "C",
            Self::D => "D",
            Self::E => "E",
            Self::F => "F",
            Self::L => "L",
            Self::H => "H",
            Self::HL => "HL",
            Self::HLP => "HLP",
            Self::HLM => "HLM",
            Self::AF => "AF",
            Self::BC => "BC",
            Self::DE => "DE",
            Self::R8 => "R8",
            Self::R16 => "R16",
            Self::D8 => "D8",
            Self::D16 => "D16",
            Self::A8 => "A8",
            Self::A16 => "A16",
            Self::SP => "SP",
            Self::SP_R8 => "SP_R8",
        };

        write!(f, "{s}")
    }
}
