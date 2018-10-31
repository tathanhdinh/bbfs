use std::fmt::{self, Display};

use fasthash::metro;
use lru::LruCache;
use zydis::{Decoder, Formatter};

use crate::{args::ExecutionMode, error::Result};

pub(crate) struct DisasmInst<'a> {
    pub address: u64,
    pub data: &'a [u8],
    pub disasm: String,
}

pub(crate) struct DisasmBasicBlock<'a> {
    instructions: Vec<DisasmInst<'a>>,
}

impl<'a> DisasmBasicBlock<'a> {
    pub fn contain_address_exact(&self, addr: u64) -> bool {
        self.instructions
            .iter()
            .position(|ins| ins.address == addr)
            .is_some()
    }

    pub fn contain_address(&self, addr: u64) -> bool {
        if let (Some(first_ins), Some(last_ins)) =
            (self.instructions.first(), self.instructions.last())
        {
            first_ins.address <= addr && addr <= last_ins.address
        } else {
            unreachable!()
        }
    }

    pub fn contain_instruction_pattern(&self, ins_pat: &str) -> bool {
        self.instructions
            .iter()
            .position(|ins| ins.disasm.contains(ins_pat))
            .is_some()
    }

    // ref: https://stackoverflow.com/questions/35901547/how-can-i-find-a-subsequence-in-a-u8-slice
    pub fn contains_instruction_bytes(&self, ins_bytes: &[u8]) -> bool {
        self.instructions
            .iter()
            .position(|ins| {
                ins.data
                    .windows(ins_bytes.len())
                    .position(|w| w == ins_bytes)
                    .is_some()
            })
            .is_some()
    }
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

struct DisasmInstructionLayout {
    pub address: u64,
    pub end_offset: usize,
    pub disasm: String,
}

struct DisasmBasicBlockLayout {
    instruction_layouts: Vec<DisasmInstructionLayout>,
}

pub(crate) struct Disasm<'a> {
    decoder_32: Decoder,
    decoder_64: Decoder,
    formatter: Formatter<'a>,
    layout_cache: LruCache<(ExecutionMode, u64), DisasmBasicBlockLayout>,
    decoded_buffer: [u8; 200],
}

impl<'a, 'b> Disasm<'a> {
    pub fn from_args() -> Result<Self> {
        use zydis::*;

        let decoder_32 = Decoder::new(MachineMode::LongCompat32, AddressWidth::_32)?;
        let decoder_64 = Decoder::new(MachineMode::Long64, AddressWidth::_64)?;

        let mut formatter = Formatter::new(FormatterStyle::Intel)?;
        formatter.set_property(FormatterProperty::AddressPaddingRelative(Padding::Auto))?;
        formatter.set_property(FormatterProperty::AddressPaddingAbsolute(Padding::Auto))?;
        formatter.set_property(FormatterProperty::AddressSignedness(Signedness::Unsigned))?;

        formatter.set_property(FormatterProperty::DisplacementPadding(Padding::Disabled))?;
        formatter.set_property(FormatterProperty::DisplacementSignedness(
            Signedness::Signed,
        ))?;
        formatter.set_property(FormatterProperty::ImmediatePadding(Padding::Disabled))?;
        formatter.set_property(FormatterProperty::ImmediateSignedness(Signedness::Unsigned))?;

        formatter.set_property(FormatterProperty::HexUppercase(false))?;
        formatter.set_property(FormatterProperty::ForceRelativeRiprel(true))?;

        Ok(Disasm {
            decoder_32,
            decoder_64,
            formatter,
            layout_cache: LruCache::new(16 * 1024),
            decoded_buffer: [0u8; 200],
        })
    }

    pub fn disasm(
        &mut self,
        data: &'b [u8],
        execution_mode: ExecutionMode,
        base_address: Option<u64>,
    ) -> Result<DisasmBasicBlock<'b>> {
        use zydis::*;

        let decoder = match execution_mode {
            ExecutionMode::Compat => &self.decoder_32,
            ExecutionMode::Bit64 => &self.decoder_64,
        };

        let basic_block_hash = (execution_mode, metro::hash64(&data));
        if self.layout_cache.get(&basic_block_hash).is_none() {
            let decoded_insts: Vec<(DecodedInstruction, u64)> =
                decoder.instruction_iterator(data, 0).collect();

            let mut decoded_buffer = OutputBuffer::new(&mut self.decoded_buffer);

            let mut decoded_byte_count = 0usize;
            let mut disasm_inst_layouts = vec![];

            for (ins, ins_addr) in &decoded_insts {
                self.formatter
                    .format_instruction(ins, &mut decoded_buffer, None, None)?;

                let next_decoded_byte_count = decoded_byte_count + ins.length as usize;

                disasm_inst_layouts.push(DisasmInstructionLayout {
                    address: *ins_addr,
                    end_offset: next_decoded_byte_count,
                    disasm: String::from(decoded_buffer.as_str()?),
                });

                decoded_byte_count = next_decoded_byte_count;
            }

            self.layout_cache.put(
                basic_block_hash,
                DisasmBasicBlockLayout {
                    instruction_layouts: disasm_inst_layouts,
                },
            );
        }

        let disasm_basic_block_layout = self.layout_cache.get(&basic_block_hash).unwrap();
        let mut disasm_insts = vec![];
        let mut begin_offset = 0usize;

        let base_address = base_address.unwrap_or(0);

        for DisasmInstructionLayout {
            address,
            end_offset,
            disasm,
        } in &disasm_basic_block_layout.instruction_layouts
        {
            disasm_insts.push(DisasmInst {
                address: *address + base_address,
                data: &data[begin_offset..*end_offset],
                disasm: disasm.to_string(),
            });

            begin_offset = *end_offset;
        }

        Ok(DisasmBasicBlock {
            instructions: disasm_insts,
        })
    }
}
