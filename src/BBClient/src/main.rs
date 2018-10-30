use indicatif::ProgressBar;
use std::io::{self, Write};
use structopt::StructOpt;
use tabwriter::TabWriter;

#[macro_use]
mod error;
mod args;
mod disasm;
// mod ui;
mod cache;
mod iname;

// use crate::cache::Cache;

use crate::error::Result;

const REDIS_SERVER_LOCATION: &str = "redis://localhost";
const RAW_BASIC_BLOCK_LIST_NAME: &str = "raw_basic_block_list";
const BASIC_BLOCK_LIST_NAME: &str = "basic_block_list";
const INSTRUCTION_LIST_NAME: &str = "instruction_list";

fn main() -> Result<()> {
    let opt = args::Opt::from_args();

    let cache = cache::Cache::from_args(
        REDIS_SERVER_LOCATION,
        BASIC_BLOCK_LIST_NAME,
        RAW_BASIC_BLOCK_LIST_NAME,
    )?;

    let basic_blocks = cache.basic_blocks()?;

    if let Some(opt) = args::ShowingClientOpt::from(opt) {
        let stdout = io::stdout();
        let mut tw = TabWriter::new(stdout.lock()).padding(4);

        let mut disasm = disasm::Disasm::from_args()?;

        for (basic_block_index, basic_block) in basic_blocks
            .skip(opt.starting_index)
            .enumerate()
            .filter(|(_, bb)| {
                if let Some(exec_mode) = opt.execution_mode {
                    bb.execution_mode == exec_mode
                } else {
                    true
                }
            })
            .filter(|(_, bb)| {
                if let Some(exec_ring) = opt.execution_privilege {
                    bb.execution_privilege == exec_ring
                } else {
                    true
                }
            }) {
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

            writeln!(
                tw,
                "basic block: {} ({})",
                basic_block_index + opt.starting_index,
                basic_block
            );
            writeln!(tw, "\n{}\n", disasm_basic_block);
            tw.flush()?;
        }
    } else {
        let mut instruction_cache =
            iname::RemillCache::from_args(REDIS_SERVER_LOCATION, INSTRUCTION_LIST_NAME)?;

        // let progress_bar = ProgressBar::new(basic_blocks.count as u64);

        // for basic_block in basic_blocks {
        //     instruction_cache.cache_basic_block(&basic_block.data, basic_block.execution_mode)?;
        //     progress_bar.inc(1);
        // }

        // progress_bar.finish_with_message(&format!(
        //     "{} instruction cached",
        //     instruction_cache.count()?
        // ));
        println!("{} instruction cached", instruction_cache.count()?);
    }

    Ok(())
}
