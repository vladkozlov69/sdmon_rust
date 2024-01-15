#![allow(dead_code)]

use nix::ioctl_readwrite;
use nix::errno::Errno;
use std::fmt::Display;
use std::fmt::Formatter;
use crate::mmc_ioc_cmd::Cmd56::*;

const MMC_RSP_PRESENT: u32 = 1 << 0;
const MMC_RSP_136: u32 = 1 << 1;    /* 136 bit response */
const MMC_RSP_CRC: u32 = 1 << 2;    /* expect valid crc */
const MMC_RSP_BUSY: u32 = 1 << 3;   /* card may send busy */
const MMC_RSP_OPCODE: u32 = 1 << 4; /* response contains opcode */

const MMC_CMD_AC: u32 = 0 << 5;
pub const MMC_CMD_ADTC: u32 = 1 << 5;

const MMC_RSP_SPI_S1: u32 = 1 << 7;    /* one status byte */
const MMC_RSP_SPI_BUSY: u32 = 1 << 10; /* card may send busy */

const MMC_RSP_SPI_R1: u32 = MMC_RSP_SPI_S1;
const MMC_RSP_SPI_R1B: u32 = MMC_RSP_SPI_S1 | MMC_RSP_SPI_BUSY;

const MMC_RSP_NONE: u32 = 0;
pub const MMC_RSP_R1: u32 = MMC_RSP_PRESENT | MMC_RSP_CRC | MMC_RSP_OPCODE;
const MMC_RSP_R1B: u32 = MMC_RSP_PRESENT | MMC_RSP_CRC | MMC_RSP_OPCODE | MMC_RSP_BUSY;

pub const COMMAND_FLAGS_CMD56_DATA_IN: u32 = MMC_RSP_SPI_R1 | MMC_RSP_R1 | MMC_CMD_ADTC; // 181
pub const COMMAND_FLAGS_CMD56_WRITE: u32 = MMC_RSP_R1 | MMC_CMD_ADTC;

pub const SD_BLOCK_SIZE: usize = 512;

const MMC_BLOCK_MAJOR: u8 = 0xB3;
const SD_GEN_CMD: u32 = 56;

ioctl_readwrite!(mmc_ioc_cmd_rw, MMC_BLOCK_MAJOR, 0, MmcIocCmd);

pub type SDBlock = [u8; SD_BLOCK_SIZE];

pub struct SDB1 {
    data: SDBlock
}

impl SDB1 {
    pub fn new() -> Self {
        return SDB1{data: [0; SD_BLOCK_SIZE]};
    }
    pub fn data(&self) -> &SDBlock {
        return &(self.data);
    }
}

impl Display for SDB1 {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> { 
        _ = writeln!(f, "=== Begin buffer dump ===");
        for i in 0..SD_BLOCK_SIZE {
            _ = write!(f, "{:02X?} ", self.data[i]);
            if (i+1) % 16 == 0 {
                _ = writeln!(f);
            }
        }
        return writeln!(f, "=== End buffer dump ==="); 
    }
}

// #[derive(FromPrimitive)]
#[derive(Copy, Clone, PartialEq)]
pub enum Cmd56 {
    
    Sandisk = 0x00000001, // Sandisk, Longsys
    Micron = 0x110005fb, // Micron
    Swissbit = 0x53420001, // Swissbit 
    Transcend = 0x110005F9, // Transcend
    LongsysM9H = 0x110005FD, // Longsys Industrial M9H, Innodisk
    Atp = 0x11000001,  // ATP Industrial 
    AData = 0x110005f1
}

pub const CMDS56: [Cmd56; 7] = [Sandisk, Micron, Swissbit, Transcend, LongsysM9H, Atp, AData];

pub trait GetInstance<'sdb, T> {
    fn get_instance() -> &'sdb T;
}

impl<'sdb> GetInstance<'sdb, SDBlock> for SDBlock {
    fn get_instance() -> &'sdb SDBlock {
        return &[0; SD_BLOCK_SIZE];
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
struct MmcIocCmd {
    pub write_flag: cty::c_int,
    pub is_acmd: cty::c_int,
    pub opcode: cty::c_uint,
    pub arg: cty::c_uint,
    pub response: [cty::c_uint; 4],
    pub flags: cty::c_uint,
    pub blksz: cty::c_uint,
    pub blocks: cty::c_uint,
    pub postsleep_min_us: cty::c_uint,
    pub postsleep_max_us: cty::c_uint,
    pub data_timeout_ns: cty::c_uint,
    pub cmd_timeout_ms: cty::c_uint,
    pub __pad: cty::c_uint,
    pub data_ptr: cty::c_ulong,
}

impl MmcIocCmd {
    pub fn new(cmd_write_flag: i32, cmd_opcode:u32, cmd_arg: u32, cmd_flags: u32, lba_block_data: &SDBlock) -> Self {
        Self { 
            write_flag : cmd_write_flag, 
            is_acmd : 0, 
            opcode : cmd_opcode, 
            arg : cmd_arg,
            response : [0; 4], 
            flags : cmd_flags, 
            blksz : SD_BLOCK_SIZE as u32, 
            blocks : 1,
            postsleep_min_us : 0, 
            postsleep_max_us : 0,
            data_timeout_ns : 0, 
            cmd_timeout_ms : 0,
            __pad : 0, 
            data_ptr : lba_block_data as *const u8 as u64 }
    }
}

// pub fn dump_buf(buf: &SDBlock) {
//     println!("=== Begin buffer dump ===");
//     for i in 0..buf.len() {
//         print!("{:02X?} ", buf[i]);
//         if (i+1) % 16 == 0 {
//             println!();
//         }
//     }
//     println!("=== End buffer dump ===");
// }

pub fn cmd56_data_in(fdesc: i32, cmd56_arg: u32, lba_block_data: &SDB1, debug: bool) -> Result<i32, Errno> {
    let mut command: MmcIocCmd = MmcIocCmd::new(0, SD_GEN_CMD, 
        cmd56_arg, COMMAND_FLAGS_CMD56_DATA_IN, lba_block_data.data());
        
    unsafe {
        let res = mmc_ioc_cmd_rw(fdesc, &mut command/* as *mut _ */);
        if debug {
            dbg!(command);
            if res.is_ok() {
                println!("{}", lba_block_data);
                // dump_buf(lba_block_data);
            }
        }

        return res;
    }
}

pub fn cmd56_write(fdesc: i32, cmd56_arg: u32, debug: bool) -> Result<i32, Errno> {
    let lba_block_data: SDB1 = SDB1::new();

    let mut command: MmcIocCmd = MmcIocCmd::new(1, SD_GEN_CMD, 
        cmd56_arg, COMMAND_FLAGS_CMD56_WRITE, lba_block_data.data());

    unsafe {
        let res = mmc_ioc_cmd_rw(fdesc, &mut command);
        if debug {
            dbg!(command);
            if res.is_ok() {
                println!("{}", lba_block_data);
            }
        }
        return res;
    }
}