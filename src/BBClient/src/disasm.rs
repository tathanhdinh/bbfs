use std::fmt::{self, Display};

use zydis::{Decoder, Formatter};

use crate::{args::ExecutionMode, error::Result};

struct DisasmInst<'a> {
    pub address: u64,
    pub data: &'a [u8],
    pub disasm: String,
}

struct DisasmBasicBlock<'a> {
    instructions: Vec<DisasmInst<'a>>,
}

impl<'a> Display for DisasmBasicBlock<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut formatted_insts = Vec::new();
        for inst in &self.instructions {
            let formatted_inst = {
                let inst_data_str = &inst
                    .data
                    .iter()
                    .map(|b| format!("{:02x}", b))
                    .collect::<Vec<_>>()
                    .join(" ")[..];
                format!(
                    "0x{:016x}\t{:45}\t{}",
                    inst.address, inst_data_str, &inst.disasm
                )
            };
            formatted_insts.push(formatted_inst);
        }
        write!(f, "{}", formatted_insts.join("\n"))
    }
}

struct Disasm<'a> {
    decoder_32: Decoder,
    decoder_64: Decoder,
    formatter: Formatter<'a>,
}

impl<'a> Disasm<'a> {
    fn new() -> Result<Self> {
        use zydis::*;

        let decoder_32 = Decoder::new(MachineMode::LongCompat32, AddressWidth::_32)?;
        let decoder_64 = Decoder::new(MachineMode::Long64, AddressWidth::_64)?;

        let mut formatter = Formatter::new(FormatterStyle::Intel)?;
        formatter.set_property(FormatterProperty::AddressPaddingRelative(Padding::Disabled))?;
        formatter.set_property(FormatterProperty::AddressPaddingAbsolute(Padding::Disabled))?;
        formatter.set_property(FormatterProperty::AddressSignedness(Signedness::Signed))?;
        formatter.set_property(FormatterProperty::DisplacementPadding(Padding::Disabled))?;
        formatter.set_property(FormatterProperty::ImmediatePadding(Padding::Disabled))?;
        formatter.set_property(FormatterProperty::HexUppercase(false))?;
        formatter.set_property(FormatterProperty::DisplacementSignedness(
            Signedness::Signed,
        ))?;

        Ok(Disasm {
            decoder_32,
            decoder_64,
            formatter,
        })
    }

    fn disasm(
        &self,
        data: &'a [u8],
        execution_mode: ExecutionMode,
        base_address: Option<u64>,
    ) -> Result<DisasmBasicBlock<'a>> {
        use zydis::*;

        let decoder = match execution_mode {
            ExecutionMode::Compat => &self.decoder_32,
            ExecutionMode::Bit64 => &self.decoder_64,
        };

        let base_address = base_address.unwrap_or(0);

        let decoded_insts: Vec<(DecodedInstruction, u64)> =
            decoder.instruction_iterator(data, base_address).collect();

        let mut decoded_buffer = [0u8, 200];
        let mut decoded_buffer = { OutputBuffer::new(&mut decoded_buffer) };

        let mut decoded_byte_count = 0usize;
        let mut disasm_insts = vec![];

        for (ins, ins_addr) in &decoded_insts {
            self.formatter
                .format_instruction(ins, &mut decoded_buffer, None, None)?;

            let next_decoded_byte_count = decoded_byte_count + ins.length as usize;

            let ins_disasm = String::from(decoded_buffer.as_str()?);
            disasm_insts.push(DisasmInst {
                address: *ins_addr,
                data: &data[decoded_byte_count..next_decoded_byte_count],
                disasm: ins_disasm,
            });

            decoded_byte_count = next_decoded_byte_count;
        }

        Ok(DisasmBasicBlock {
            instructions: disasm_insts,
        })
    }
}
