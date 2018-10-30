use lazy_static::lazy_static;
// use std::{path::PathBuf, hash::Hash};
use structopt::StructOpt;
use strum::{AsStaticRef, IntoEnumIterator};
use strum_macros::{AsStaticStr, EnumIter, EnumString};

// use crate::error::Result;

#[derive(EnumString, EnumIter, AsStaticStr, Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub(crate) enum ExecutionMode {
    #[strum(serialize = "compat")]
    Compat,

    #[strum(serialize = "64-bit")]
    Bit64,
}

#[derive(EnumString, EnumIter, AsStaticStr, Debug, PartialEq, Eq, Clone, Copy)]
pub(crate) enum ExecutionPrivilege {
    #[strum(serialize = "user")]
    User,

    #[strum(serialize = "kernel")]
    Kernel,
}

lazy_static! {
    static ref EXECUTION_MODES: Vec<&'static str> =
        { ExecutionMode::iter().map(|e| e.as_static()).collect() };
    static ref EXECUTION_PRIVILEGES: Vec<&'static str> =
        { ExecutionPrivilege::iter().map(|e| e.as_static()).collect() };
}

#[derive(StructOpt, Debug)]
#[structopt(name = "client option")]
pub(crate) enum Opt {
    #[structopt(name = "cache")]
    CacheOpt {},

    #[structopt(name = "show")]
    ShowOpt {
        // #[structopt(
        //     name = "basic_block_file",
        //     help = "basic block file (used as database name)",
        //     parse(from_os_str)
        // )]
        // database: PathBuf,
        // #[structopt(
        //     name = "verbosity",
        //     short = "v",
        //     long = "verbose",
        //     help = "verbosity",
        //     parse(from_occurrences)
        // )]
        // verbosity: u8,
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
            help = "show only basic blocks under the specified execution mode",
            raw(possible_values = "&EXECUTION_MODES", case_insensitive = "false")
        )]
        execution_mode: Option<ExecutionMode>,

        #[structopt(
            name = "ring",
            short = "r",
            long = "ring",
            help = "show only basic blocks under the specified execution privilege",
            raw(possible_values = "&EXECUTION_PRIVILEGES")
        )]
        execution_privilege: Option<ExecutionPrivilege>,

        #[structopt(
            name = "starting index",
            short = "g",
            long = "goto",
            help = "start everything from the basic block of index",
            default_value = "0"
        )]
        starting_index: usize,
        // #[structopt(
        //     name = "cache instruction",
        //     long = "cache",
        //     help = "cache instructions using their function names"
        // )]
        // cache_instruction: bool,
    },
}

// #[derive(StructOpt, Debug)]
// #[structopt(name = "client cache option")]
// struct CacheOpt {}

// #[derive(StructOpt, Debug)]
// #[structopt(name = "client show option")]
// struct ShowOpt {
//     // #[structopt(
//     //     name = "basic_block_file",
//     //     help = "basic block file (used as database name)",
//     //     parse(from_os_str)
//     // )]
//     // database: PathBuf,
//     // #[structopt(
//     //     name = "verbosity",
//     //     short = "v",
//     //     long = "verbose",
//     //     help = "verbosity",
//     //     parse(from_occurrences)
//     // )]
//     // verbosity: u8,

//     #[structopt(
//         name = "instruction pattern",
//         short = "p",
//         long = "pattern",
//         help = "search for instructions containing the pattern"
//     )]
//     instruction_pattern: Option<String>,

//     #[structopt(
//         name = "execution mode",
//         short = "m",
//         long = "exec_mode",
//         help = "show only basic blocks under the specified execution mode",
//         raw(possible_values = "&EXECUTION_MODES", case_insensitive = "false")
//     )]
//     execution_mode: Option<ExecutionMode>,

//     #[structopt(
//         name = "ring",
//         short = "r",
//         long = "ring",
//         help = "show only basic blocks under the specified execution privilege",
//         raw(possible_values = "&EXECUTION_PRIVILEGES")
//     )]
//     execution_privilege: Option<ExecutionPrivilege>,

//     #[structopt(
//         name = "starting index",
//         short = "g",
//         long = "goto",
//         help = "start everything from the basic block of index",
//         default_value = "0"
//     )]
//     starting_index: usize,

//     #[structopt(
//         name = "cache instruction",
//         long = "cache",
//         help = "cache instructions using their function names"
//     )]
//     cache_instruction: bool,
// }

pub(crate) struct ShowingClientOpt {
    // pub database: String,
    pub execution_mode: Option<ExecutionMode>,
    pub execution_privilege: Option<ExecutionPrivilege>,
    pub starting_index: usize,
    pub instruction_pattern: Option<String>,
    // pub cache_instruction: bool,
    // pub verbosity: u8,
}

impl ShowingClientOpt {
    pub fn from(opt: Opt) -> Option<Self> {
        // let opt = Opt::from_args();
        match opt {
            Opt::ShowOpt {
                execution_mode,
                execution_privilege,
                starting_index,
                instruction_pattern,
            } => Some(ShowingClientOpt {
                // database: opt.database,
                execution_mode,
                execution_privilege,
                starting_index,
                instruction_pattern,
                // cache_instruction: opt.cache_instruction,
                // verbosity: opt.verbosity,
            }),
            _ => None,
        }

        // let database = {
        //     let database = opt
        //         .database
        //         .to_str()
        //         .ok_or_else(|| application_error!("bad database name"))?;
        //     database.to_owned()
        // };

        // Ok(BBClientOpt {
        //     database: database,
        //     execution_mode: opt.execution_mode,
        //     starting_index: opt.starting_index,
        //     instruction_pattern: opt.instruction_pattern,
        //     verbosity: opt.verbosity,
        // })
    }
}
