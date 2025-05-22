mod cpu;
pub mod game_boy;
pub mod gbcartridge;
mod instruction;
mod opcode;
mod registers;
mod target;

pub use instruction::Instruction as GbInstruction;
pub use opcode::OpCode as GbOpCode;
