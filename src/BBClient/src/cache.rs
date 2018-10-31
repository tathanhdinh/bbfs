use std::{
    fmt::{self, Display},
    io::{Cursor, Read},
    marker::PhantomData,
};

use redis::{cmd, Client, Commands, Connection};
use scroll::IOread;
use strum::AsStaticRef;

use crate::args::{ExecutionMode, ExecutionPrivilege};

// struct RawBasicBlock {
//     pub program_counter: u64,
//     pub execution_mode: u8,
//     pub execution_privilege: u8,
//     pub loop_count: u64,
//     pub data: Vec<u8>,
// }

// impl From<&[u8]> for RawBasicBlock {
//     fn from(raw: &[u8]) -> Self {
//         let mut raw = Cursor::new(raw);
//         let program_counter = raw.ioread::<u64>().unwrap();
//         let execution_mode = raw.ioread::<u8>().unwrap();
//         let execution_privilege = raw.ioread::<u8>().unwrap();
//         let loop_count = raw.ioread::<u64>().unwrap();
//         let mut data = Vec::new();
//         raw.read_to_end(&mut data).unwrap();

//         RawBasicBlock {
//             program_counter,
//             execution_mode,
//             execution_privilege,
//             loop_count,
//             data,
//         }
//     }
// }

pub(crate) struct BasicBlock {
    pub program_counter: u64,
    pub execution_mode: ExecutionMode,
    pub execution_privilege: ExecutionPrivilege,
    pub loop_count: u64,
    pub data: Vec<u8>,
}

// impl From<RawBasicBlock> for BasicBlock {
//     fn from(raw: RawBasicBlock) -> Self {
//         let execution_mode = match raw.execution_mode {
//             0 => ExecutionMode::Compat,

//             1 => ExecutionMode::Bit64,

//             _ => unreachable!(),
//         };

//         let execution_privilege = match raw.execution_privilege {
//             0 => ExecutionPrivilege::Kernel,

//             3 => ExecutionPrivilege::User,

//             _ => unreachable!(),
//         };

//         BasicBlock {
//             program_counter: raw.program_counter,
//             execution_mode,
//             execution_privilege,
//             loop_count: raw.loop_count,
//             data: raw.data,
//         }
//     }
// }

impl From<Vec<u8>> for BasicBlock {
    fn from(raw: Vec<u8>) -> Self {
        let mut raw = Cursor::new(raw);
        let program_counter = raw.ioread::<u64>().unwrap();
        let execution_mode = raw.ioread::<u8>().unwrap();
        let execution_privilege = raw.ioread::<u8>().unwrap();
        let loop_count = raw.ioread::<u64>().unwrap();
        let mut data = Vec::new();
        raw.read_to_end(&mut data).unwrap();

        let execution_mode = match execution_mode {
            0 => ExecutionMode::Compat,

            1 => ExecutionMode::Bit64,

            _ => unreachable!(),
        };

        let execution_privilege = match execution_privilege {
            0 => ExecutionPrivilege::Kernel,

            3 => ExecutionPrivilege::User,

            _ => unreachable!(),
        };

        BasicBlock {
            program_counter: program_counter,
            execution_mode,
            execution_privilege,
            loop_count: loop_count,
            data: data,
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
}

pub(crate) struct CachedBasicBlockIter<'a, 'b, T> {
    connection: &'a Connection,
    database: &'b str,
    next_index: usize,
    pub count: usize,
    phantom: PhantomData<T>,
}

// ref: https://users.rust-lang.org/t/problem-with-generics-and-iterator/14863/6
// and: http://bluejekyll.github.io/blog/rust/2017/08/06/type-parameters.html
impl<'a, 'b, T> Iterator for CachedBasicBlockIter<'a, 'b, T>
where
    T: From<Vec<u8>>,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.next_index >= self.count {
            None
        } else {
            let data: std::result::Result<Vec<u8>, _> = self
                .connection
                .lindex(self.database, self.next_index as isize);

            let data = data.unwrap();

            self.next_index += 1;

            Some(From::from(data))
        }
    }
}

impl Cache {
    pub fn from_url(redis_server_url: &str) -> Result<Self> {
        let client = Client::open(redis_server_url)?;
        let connection = client.get_connection()?;

        Ok(Cache { connection })
    }

    pub fn basic_blocks<'a, 'b, T>(
        &'a self,
        database: &'b str,
    ) -> Result<CachedBasicBlockIter<'a, 'b, T>> {
        let database_exists: bool = self.connection.exists(database)?;
        if database_exists {
            let cached_database_type_name: String =
                cmd("TYPE").arg(database).query(&self.connection)?;

            if cached_database_type_name == "list" {
                let count: usize = self.connection.llen(database).unwrap();

                Ok(CachedBasicBlockIter::<T> {
                    connection: &self.connection,
                    database,
                    next_index: 0,
                    count,
                    phantom: PhantomData,
                })
            } else {
                Err(application_error!("cached basic block data is not a list"))
            }
        } else {
            Err(application_error!("cached basic block data does not exist"))
        }
    }
}
