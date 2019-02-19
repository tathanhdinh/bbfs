use std::{
    collections::{HashMap, HashSet},
    mem,
};

use lazy_static::lazy_static;
use maplit::hashmap;
use redis::{cmd, Client, Commands, Connection};

// use xedsys as intel;
// use crate::intel;

use xedsys::{
    xed_attribute_enum_t::*, xed_error_enum_t::*, xed_iclass_enum_t::*, xed_iform_enum_t::{self, *},
    xed_operand_enum_t::*, *,
    xed_machine_mode_enum_t, xed_address_width_enum_t,
};

use crate::{args::ExecutionMode, error::Result};

macro_rules! ref_to_raw_pointer {
    ($ref_v:expr) => {
        $ref_v as *const _
    };
}

macro_rules! ref_to_mut_raw_pointer {
    ($ref_v:expr) => {
        $ref_v as *mut _
    };
}

macro_rules! raw_pointer_to_ref {
    ($raw_p:expr) => {
        unsafe { &*$raw_p }
    };
}

lazy_static! {
    static ref xed_nolock_iform_map: HashMap<xed_iform_enum_t, xed_iform_enum_t> = hashmap! {
        XED_IFORM_ADC_LOCK_MEMb_IMMb_80r2 => XED_IFORM_ADC_MEMb_IMMb_80r2,
        XED_IFORM_ADC_LOCK_MEMv_IMMz => XED_IFORM_ADC_MEMv_IMMz,
        XED_IFORM_ADC_LOCK_MEMb_IMMb_82r2 => XED_IFORM_ADC_MEMb_IMMb_82r2,
        XED_IFORM_ADC_LOCK_MEMv_IMMb => XED_IFORM_ADC_MEMv_IMMb,
        XED_IFORM_ADC_LOCK_MEMb_GPR8 => XED_IFORM_ADC_MEMb_GPR8,
        XED_IFORM_ADC_LOCK_MEMv_GPRv => XED_IFORM_ADC_MEMv_GPRv,
        XED_IFORM_DEC_LOCK_MEMb => XED_IFORM_DEC_MEMb,
        XED_IFORM_DEC_LOCK_MEMv => XED_IFORM_DEC_MEMv,
        XED_IFORM_NOT_LOCK_MEMb => XED_IFORM_NOT_MEMb,
        XED_IFORM_NOT_LOCK_MEMv => XED_IFORM_NOT_MEMv,
        XED_IFORM_SUB_LOCK_MEMb_IMMb_80r5 => XED_IFORM_SUB_MEMb_IMMb_80r5,
        XED_IFORM_SUB_LOCK_MEMv_IMMz => XED_IFORM_SUB_MEMv_IMMz,
        XED_IFORM_SUB_LOCK_MEMb_IMMb_82r5 => XED_IFORM_SUB_MEMb_IMMb_82r5,
        XED_IFORM_SUB_LOCK_MEMv_IMMb => XED_IFORM_SUB_MEMv_IMMb,
        XED_IFORM_SUB_LOCK_MEMb_GPR8 => XED_IFORM_SUB_MEMb_GPR8,
        XED_IFORM_SUB_LOCK_MEMv_GPRv => XED_IFORM_SUB_MEMv_GPRv,
        XED_IFORM_BTC_LOCK_MEMv_IMMb => XED_IFORM_BTC_MEMv_IMMb,
        XED_IFORM_BTC_LOCK_MEMv_GPRv => XED_IFORM_BTC_MEMv_GPRv,
        XED_IFORM_AND_LOCK_MEMb_IMMb_80r4 => XED_IFORM_AND_MEMb_IMMb_80r4,
        XED_IFORM_AND_LOCK_MEMv_IMMz => XED_IFORM_AND_MEMv_IMMz,
        XED_IFORM_AND_LOCK_MEMb_IMMb_82r4 => XED_IFORM_AND_MEMb_IMMb_82r4,
        XED_IFORM_AND_LOCK_MEMv_IMMb => XED_IFORM_AND_MEMv_IMMb,
        XED_IFORM_AND_LOCK_MEMb_GPR8 => XED_IFORM_AND_MEMb_GPR8,
        XED_IFORM_AND_LOCK_MEMv_GPRv => XED_IFORM_AND_MEMv_GPRv,
        XED_IFORM_CMPXCHG_LOCK_MEMb_GPR8 => XED_IFORM_CMPXCHG_MEMb_GPR8,
        XED_IFORM_CMPXCHG_LOCK_MEMv_GPRv => XED_IFORM_CMPXCHG_MEMv_GPRv,
        XED_IFORM_INC_LOCK_MEMb => XED_IFORM_INC_MEMb,
        XED_IFORM_INC_LOCK_MEMv => XED_IFORM_INC_MEMv,
        XED_IFORM_OR_LOCK_MEMb_IMMb_80r1 => XED_IFORM_OR_MEMb_IMMb_80r1,
        XED_IFORM_OR_LOCK_MEMv_IMMz => XED_IFORM_OR_MEMv_IMMz,
        XED_IFORM_OR_LOCK_MEMb_IMMb_82r1 => XED_IFORM_OR_MEMb_IMMb_82r1,
        XED_IFORM_OR_LOCK_MEMv_IMMb => XED_IFORM_OR_MEMv_IMMb,
        XED_IFORM_OR_LOCK_MEMb_GPR8 => XED_IFORM_OR_MEMb_GPR8,
        XED_IFORM_OR_LOCK_MEMv_GPRv => XED_IFORM_OR_MEMv_GPRv,
        XED_IFORM_XADD_LOCK_MEMb_GPR8 => XED_IFORM_XADD_MEMb_GPR8,
        XED_IFORM_XADD_LOCK_MEMv_GPRv => XED_IFORM_XADD_MEMv_GPRv,
        XED_IFORM_ADD_LOCK_MEMb_IMMb_80r0 => XED_IFORM_ADD_MEMb_IMMb_80r0,
        XED_IFORM_ADD_LOCK_MEMv_IMMz => XED_IFORM_ADD_MEMv_IMMz,
        XED_IFORM_ADD_LOCK_MEMb_IMMb_82r0 => XED_IFORM_ADD_MEMb_IMMb_82r0,
        XED_IFORM_ADD_LOCK_MEMv_IMMb => XED_IFORM_ADD_MEMv_IMMb,
        XED_IFORM_ADD_LOCK_MEMb_GPR8 => XED_IFORM_ADD_MEMb_GPR8,
        XED_IFORM_ADD_LOCK_MEMv_GPRv => XED_IFORM_ADD_MEMv_GPRv,
        XED_IFORM_SBB_LOCK_MEMb_IMMb_80r3 => XED_IFORM_SBB_MEMb_IMMb_80r3,
        XED_IFORM_SBB_LOCK_MEMv_IMMz => XED_IFORM_SBB_MEMv_IMMz,
        XED_IFORM_SBB_LOCK_MEMb_IMMb_82r3 => XED_IFORM_SBB_MEMb_IMMb_82r3,
        XED_IFORM_SBB_LOCK_MEMv_IMMb => XED_IFORM_SBB_MEMv_IMMb,
        XED_IFORM_SBB_LOCK_MEMb_GPR8 => XED_IFORM_SBB_MEMb_GPR8,
        XED_IFORM_SBB_LOCK_MEMv_GPRv => XED_IFORM_SBB_MEMv_GPRv,
        XED_IFORM_BTS_LOCK_MEMv_IMMb => XED_IFORM_BTS_MEMv_IMMb,
        XED_IFORM_BTS_LOCK_MEMv_GPRv => XED_IFORM_BTS_MEMv_GPRv,
        XED_IFORM_XOR_LOCK_MEMb_IMMb_80r6 => XED_IFORM_XOR_MEMb_IMMb_80r6,
        XED_IFORM_XOR_LOCK_MEMv_IMMz => XED_IFORM_XOR_MEMv_IMMz,
        XED_IFORM_XOR_LOCK_MEMb_IMMb_82r6 => XED_IFORM_XOR_MEMb_IMMb_82r6,
        XED_IFORM_XOR_LOCK_MEMv_IMMb => XED_IFORM_XOR_MEMv_IMMb,
        XED_IFORM_XOR_LOCK_MEMb_GPR8 => XED_IFORM_XOR_MEMb_GPR8,
        XED_IFORM_XOR_LOCK_MEMv_GPRv => XED_IFORM_XOR_MEMv_GPRv,
        XED_IFORM_BTR_LOCK_MEMv_IMMb => XED_IFORM_BTR_MEMv_IMMb,
        XED_IFORM_BTR_LOCK_MEMv_GPRv => XED_IFORM_BTR_MEMv_GPRv,
        XED_IFORM_CMPXCHG8B_LOCK_MEMq => XED_IFORM_CMPXCHG8B_MEMq,
        XED_IFORM_CMPXCHG8B_LOCK_MEMq => XED_IFORM_CMPXCHG8B_MEMq,
        XED_IFORM_CMPXCHG16B_LOCK_MEMdq => XED_IFORM_CMPXCHG16B_MEMdq,
        XED_IFORM_NEG_LOCK_MEMb => XED_IFORM_NEG_MEMb,
        XED_IFORM_NEG_LOCK_MEMv => XED_IFORM_NEG_MEMv,
    };
}

struct XedInst<'a> {
    pub data: &'a [u8],
    pub function_name: String,
    pub machine_mode: ExecutionMode,
}

impl<'a> XedInst<'a> {
    pub fn from_instruction_data(data: &'a [u8], mode: ExecutionMode) -> Result<Self> {
        // use self::intel::*;
        unsafe { xed_tables_init() };

        let xed_mode = match mode {
            ExecutionMode::Compat => xed_state_t {
                mmode: xed_machine_mode_enum_t::XED_MACHINE_MODE_LONG_COMPAT_32,
                stack_addr_width: xed_address_width_enum_t::XED_ADDRESS_WIDTH_32b,
            },

            ExecutionMode::Bit64 => xed_state_t {
                mmode: xed_machine_mode_enum_t::XED_MACHINE_MODE_LONG_64,
                stack_addr_width: xed_address_width_enum_t::XED_ADDRESS_WIDTH_64b,
            },
        };

        let mut decoded_inst: xed_decoded_inst_t = unsafe { mem::uninitialized() };
        let decoded_inst_ptr = &mut decoded_inst;

        unsafe {
            xed_decoded_inst_zero_set_mode(
                ref_to_mut_raw_pointer!(decoded_inst_ptr),
                &xed_mode as *const xed_state_t,
            )
        };

        let decoding_error = unsafe {
            xed_decode(
                ref_to_mut_raw_pointer!(decoded_inst_ptr),
                data.as_ptr(),
                data.len() as u32,
            )
        };

        if decoding_error != XED_ERROR_NONE {
            return Err(application_error!(error_str(decoding_error)));
        }

        let remill_function_name = {
            let inst_base = decoded_inst_inst(&decoded_inst).unwrap();
            let iclass = inst_iclass(inst_base);

            format!("{}", iclass_str(iclass))

            // let has_lock = unsafe { xed_operand_values_has_lock_prefix(&decoded_inst) };

            // let iform = {
            //     let iform = inst_iform_enum(inst_base);
            //     if has_lock != 0 {
            //         *xed_nolock_iform_map.get(&iform).unwrap()
            //     } else {
            //         iform
            //     }
            // };

            // let mut func_name = format!("{}", iform_str(iform));

            // let is_scalable = {
            //     let sc = unsafe { xed_inst_get_attribute(inst_base, XED_ATTRIBUTE_SCALABLE) };
            //     sc != 0
            // };

            // if is_scalable {
            //     func_name = format!("{}_{}", func_name, unsafe {
            //         xed_decoded_inst_get_operand_width(&decoded_inst)
            //     });
            // }

            // match iform {
            //     XED_IFORM_MOV_SEG_MEMw
            //     | XED_IFORM_MOV_SEG_GPR16
            //     | XED_IFORM_MOV_CR_CR_GPR32
            //     | XED_IFORM_MOV_CR_CR_GPR64 => format!(
            //         "{}_{}",
            //         func_name,
            //         reg_str(unsafe { xed_decoded_inst_get_reg(&decoded_inst, XED_OPERAND_REG0,) })
            //     ),

            //     _ => func_name,
            // }
        };

        let decoded_byte_count = decoded_inst_get_length(&decoded_inst) as usize;

        Ok(XedInst {
            data: &data[0..decoded_byte_count],
            machine_mode: mode,
            function_name: remill_function_name,
        })
    }

    pub fn from_basic_block_data(data: &'a [u8], mode: ExecutionMode) -> Vec<Self> {
        let mut xed_insts = vec![];

        let mut decoded_byte_count = 0usize;
        while decoded_byte_count < data.len() {
            if let Ok(xed_inst) = XedInst::from_instruction_data(&data[decoded_byte_count..], mode)
            {
                decoded_byte_count += xed_inst.data.len();
                xed_insts.push(xed_inst);
            } else {
                break;
            }
        }

        xed_insts
    }
}

pub(crate) struct RemillCache<'a> {
    connection: Connection,
    database: &'a str,
    cached_names: HashSet<String>,
}

impl<'a, 'b> RemillCache<'a> {
    pub fn from_args(redis_server_url: &str, instruction_list_name: &'a str) -> Result<Self> {
        let client = Client::open(redis_server_url)?;
        let connection = client.get_connection()?;
        Ok(RemillCache {
            connection,
            database: instruction_list_name,
            cached_names: HashSet::new(),
        })
    }

    pub fn cache_basic_block(&mut self, data: &'b [u8], mode: ExecutionMode) -> Result<()> {
        let numeric_mode: u8 = match mode {
            ExecutionMode::Compat => 0,
            ExecutionMode::Bit64 => 1,
            _ => unreachable!(),
        };
        let xed_insts = XedInst::from_basic_block_data(data, mode);

        for xed_inst in xed_insts {
            if !self.cached_names.contains(&xed_inst.function_name) {
                let mut cache_data = vec![numeric_mode];
                cache_data.extend(xed_inst.data);
                self.connection.rpush(self.database, cache_data)?;

                self.cached_names.insert(xed_inst.function_name);
            }
        }

        Ok(())
    }

    pub fn count(&mut self) -> Result<usize> {
        self.connection.llen(self.database).map_err(From::from)
    }
}
