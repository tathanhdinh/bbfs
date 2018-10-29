use std::io::{self, Write};
use tabwriter::TabWriter;

#[macro_use]
mod error;
mod args;
mod disasm;
// mod ui;
mod cache;

// use crate::cache::Cache;

use crate::error::Result;

const REDIS_SERVER_LOCATION: &str = "redis://localhost";
const BASIC_BLOCK_LIST_NAME: &str = "basic_block_list";

fn main() -> Result<()> {
    let stdout = io::stdout();
    let mut tw = TabWriter::new(stdout.lock()).padding(4);

    let opt = args::BBClientOpt::new();
    let cache = cache::Cache::from_args(REDIS_SERVER_LOCATION, BASIC_BLOCK_LIST_NAME)?;
    let mut disasm = disasm::Disasm::from_args()?;
    // let mut basic_block_index = opt.starting_index;

    for basic_block in cache
        .basic_blocks()
        .skip(opt.starting_index)
        .filter(|bb| {
            // basic_block_index += 1;
            if let Some(exec_mode) = opt.execution_mode {
                bb.execution_mode == exec_mode
            } else {
                true
            }
        })
        .filter(|bb| {
            if let Some(exec_ring) = opt.execution_privilege {
                bb.execution_privilege == exec_ring
            } else {
                true
            }
        }) {
        // writeln!(tw, "basic block: {} ({})", basic_block_index, basic_block);

        let disasm_basic_block = disasm.disasm(
            &basic_block.data,
            basic_block.execution_mode,
            Some(basic_block.program_counter),
        )?;

        if let Some(ref ins_pattern) = opt.instruction_pattern {
            if !disasm_basic_block.contain_instruction_pattern(&ins_pattern) {
                continue;
            }
        }

        writeln!(tw, "{}", basic_block);
        writeln!(tw, "\n{}\n", disasm_basic_block);
        tw.flush()?;

        // basic_block_index += 1;
    }

    Ok(())
}
