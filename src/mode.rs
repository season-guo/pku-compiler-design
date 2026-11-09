use std::io::Write;

use koopa::{back::KoopaGenerator, ir::Program};

pub enum Mode {
    RiscV,
    Koopa,
}

impl From<String> for Mode {
    fn from(value: String) -> Self {
        match value.as_str() {
            "-riscv" => Mode::RiscV,
            "-koopa" => Mode::Koopa,
            _ => Mode::Koopa,
        }
    }
}

impl Mode {
    pub fn handle(self: &Self, program: Program, output: Option<String>) {
        if let Some(output) = output {
            let file = std::fs::File::create(output).unwrap();
            self.write(program, file);
        } else {
            self.write(program, std::io::stdout());
        }
    }

    pub fn write(self: &Self, program: Program, writer: impl Write) {
        match self {
            Mode::RiscV => {
                crate::risc::risc_gen::RiscVGenerator::new().koopa_to_risc(program, writer)
            }
            Mode::Koopa => KoopaGenerator::new(writer).generate_on(&program).unwrap(),
        }
    }
}
