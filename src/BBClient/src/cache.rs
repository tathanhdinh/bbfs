use redis::{Client, Connection};

use crate::args::ExecutionMode;
use crate::args::ExecutionPrivilege;

struct RawBasicBlock {
    pub program_counter: u64,
    pub execution_mode: u8,
    pub execution_privilege: u8,
    pub loop_count: u64,
    pub data: Vec<u8>,
}

struct BasicBlock {
    pub program_counter: u64,
    pub execution_mode: ExecutionMode,
    pub execution_privilege: ExecutionPrivilege,
    pub loop_count: u64,
    pub data: Vec<u8>,
}

impl From<RawBasicBlock> for BasicBlock {
    fn from(raw: RawBasicBlock) -> Self {
        let execution_mode = match raw.execution_mode {
            0 => {
                ExecutionMode::Compat
            },

            1 => {
                ExecutionMode::Bit64
            },

            _ => unreachable!()
        };

        let execution_privilege = match raw.execution_privilege {
            0 => {
                ExecutionPrivilege::Kernel
            },

            3 => {
                ExecutionPrivilege::User
            }

            _ => unreachable!()
        };

        BasicBlock {
            program_counter: raw.program_counter,
            execution_mode,
            execution_privilege,
            loop_count: raw.loop_count,
            data: raw.data,
        }
    }
}

use crate::error::Result;

struct Cache {
    connection: Connection,
}

impl Cache {
    fn from(redis_server_url: &str, basic_block_list_name: &str) -> Result<Self> {
        // None
        let client = Client::open(redis_server_url)?;
        let connection = client.get_connection()?;
    }
}

impl Iterator for Cache {
    type Item = BasicBlock;
}