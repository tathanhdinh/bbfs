use lazy_static::lazy_static;
use std::path::PathBuf;
use structopt::StructOpt;
use strum::{AsStaticRef, IntoEnumIterator};
use strum_macros::{AsStaticStr, EnumIter, EnumString};

use crate::error::Result;

#[derive(EnumString, EnumIter, AsStaticStr, Debug)]
pub(crate) enum ExecutionMode {
    #[strum(serialize = "Compat")]
    Compat,

    #[strum(serialize = "64-bit")]
    Bit64,
}

lazy_static! {
    static ref EXECUTION_MODES: Vec<&'static str> =
        { ExecutionMode::iter().map(|e| e.as_static()).collect() };
}

#[derive(StructOpt, Debug)]
#[structopt(name = "Basic block trace client")]
struct Opt {
    #[structopt(
        name = "basic_block_file",
        help = "basic block file (used as database name)",
        parse(from_os_str)
    )]
    database: PathBuf,

    #[structopt(
        name = "verbosity",
        short = "v",
        long = "verbose",
        help = "verbosity",
        parse(from_occurrences)
    )]
    verbosity: u8,

    #[structopt(
        name = "instruction pattern",
        short = "p",
        long = "pattern",
        help = "search for instructions containing the pattern"
    )]
    instruction_pattern: Option<String>,

    #[structopt(
        name = "execution mode",
        short = "m",
        long = "exec_mode",
        help = "show only basic blocks under the execution mode",
        raw(possible_values = "&EXECUTION_MODES", case_insensitive = "false")
    )]
    execution_mode: Option<ExecutionMode>,

    #[structopt(
        name = "starting index",
        short = "g",
        long = "goto",
        help = "start everything from the basic block of index",
        default_value = "0"
    )]
    starting_index: usize,
}

pub(crate) struct BBClientOpt {
    pub database: String,
    pub execution_mode: Option<ExecutionMode>,
    pub starting_index: usize,
    pub instruction_pattern: Option<String>,
    pub verbosity: u8,
}

impl BBClientOpt {
    pub fn new() -> Result<Self> {
        let opt = Opt::from_args();

        let database = {
            let database = opt
                .database
                .to_str()
                .ok_or_else(|| application_error!("bad database name"))?;
            database.to_owned()
        };

        Ok(BBClientOpt {
            database: database,
            execution_mode: opt.execution_mode,
            starting_index: opt.starting_index,
            instruction_pattern: opt.instruction_pattern,
            verbosity: opt.verbosity,
        })
    }
}
