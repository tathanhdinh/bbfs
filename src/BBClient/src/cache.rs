use std::{
    fmt::{self, Display},
    io::{Cursor, Read},
    mem::size_of,
};

use redis::{cmd, Client, Commands, Connection};
use scroll::IOread;
use strum::AsStaticRef;

use crate::args::{ExecutionMode, ExecutionPrivilege};

struct RawBasicBlock {
    pub program_counter: u64,
    pub execution_mode: u8,
    pub execution_privilege: u8,
    pub loop_count: u64,
    pub data: Vec<u8>,
}

impl From<&[u8]> for RawBasicBlock {
    fn from(raw: &[u8]) -> Self {
        let mut raw = Cursor::new(raw);
        let program_counter = raw.ioread::<u64>().unwrap();
        let execution_mode = raw.ioread::<u8>().unwrap();
        let execution_privilege = raw.ioread::<u8>().unwrap();
        let loop_count = raw.ioread::<u64>().unwrap();
        let mut data = Vec::new();
        raw.read_to_end(&mut data).unwrap();

        RawBasicBlock {
            program_counter,
            execution_mode,
            execution_privilege,
            loop_count,
            data,
        }
    }
}

pub(crate) struct BasicBlock {
    pub program_counter: u64,
    pub execution_mode: ExecutionMode,
    pub execution_privilege: ExecutionPrivilege,
    pub loop_count: u64,
    pub data: Vec<u8>,
}

impl From<RawBasicBlock> for BasicBlock {
    fn from(raw: RawBasicBlock) -> Self {
        let execution_mode = match raw.execution_mode {
            0 => ExecutionMode::Compat,

            1 => ExecutionMode::Bit64,

            _ => unreachable!(),
        };

        let execution_privilege = match raw.execution_privilege {
            0 => ExecutionPrivilege::Kernel,

            3 => ExecutionPrivilege::User,

            _ => unreachable!(),
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

impl Display for BasicBlock {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "loop: {}, mode: {}, privilege: {}",
            self.loop_count,
            self.execution_mode.as_static(),
            self.execution_privilege.as_static()
        )
    }
}

use crate::error::Result;

pub(crate) struct Cache {
    connection: Connection,
    database: String,
}

pub(crate) struct CachedBasicBlockIter<'a, 'b> {
    connection: &'a Connection,
    database: &'b str,
    next_index: usize,
    total_count: usize,
}

impl<'a, 'b> Iterator for CachedBasicBlockIter<'a, 'b> {
    type Item = BasicBlock;

    fn next(&mut self) -> Option<Self::Item> {
        if self.next_index >= self.total_count {
            None
        } else {
            let indexed_data: std::result::Result<Vec<u8>, _> = self
                .connection
                .lindex(self.database, self.next_index as isize);

            if indexed_data.is_err() {
                unreachable!()
            }

            let indexed_data = indexed_data.unwrap();
            // println!("cached basic block length: {}", indexed_data.len());

            let min_size = size_of::<u64>() + // program counter
                                size_of::<u8>() + // execution mode
                                size_of::<u8>() + // execution privilege
                                size_of::<u64>(); // loop count

            if indexed_data.len() <= min_size {
                unreachable!()
            }

            self.next_index += 1;

            let raw_basic_block: RawBasicBlock = From::from(indexed_data.as_ref());
            Some(From::from(raw_basic_block))
        }
    }
}

impl Cache {
    pub fn from_args(redis_server_url: &str, basic_block_list_name: &str) -> Result<Self> {
        let client = Client::open(redis_server_url)?;
        let connection = client.get_connection()?;
        let cached_database_type_name: String =
            cmd("TYPE").arg(basic_block_list_name).query(&connection)?;
        if cached_database_type_name == "list" {
            Ok(Cache {
                connection,
                database: String::from(basic_block_list_name),
            })
        } else {
            Err(application_error!("cached basic block data is not a list"))
        }
    }

    pub fn basic_blocks(&self) -> CachedBasicBlockIter {
        let database = &self.database;
        let total_count: usize = self.connection.llen(database).unwrap();

        CachedBasicBlockIter {
            connection: &self.connection,
            database,
            next_index: 0,
            total_count,
        }
    }
}
