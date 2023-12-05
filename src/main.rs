#![allow(dead_code)]

use std::env;
use std::os::fd::AsFd;
use mmc_ioc_cmd::MmcIocCmd;
use mmc_ioc_cmd::{COMMAND_FLAGS_CMD56_DATA_IN, COMMAND_FLAGS_CMD56_WRITE, SD_BLOCK_SIZE, mmc_ioc_cmd_rw, SDBlock};
use parsers::{SDParser, LongsysSDParser, SandiskSDParser, SmartDataSDParser};
use nix::errno::Errno;

use std::fs::File;
use std::os::fd::AsRawFd;
use std::process;

mod mmc_ioc_cmd;
mod parsers;


const SD_GEN_CMD: u32 = 56;


fn cmd56_data_in(fdesc: i32, cmd56_arg: u32, lba_block_data: &SDBlock) -> Result<i32, Errno> {
    let mut command: MmcIocCmd = MmcIocCmd::new(0, SD_GEN_CMD, 
        cmd56_arg, COMMAND_FLAGS_CMD56_DATA_IN, lba_block_data);
        
    unsafe {
        let res = mmc_ioc_cmd_rw(fdesc, &mut command/* as *mut _ */);
        dbg!(command);
        if res.is_ok() {
            dump_buf(lba_block_data);
        }
        return res;
    }
}

fn cmd56_write(fdesc: i32, cmd56_arg: u32) -> Result<i32, Errno> {
    let data_out: SDBlock = [0; SD_BLOCK_SIZE];

    let mut command: MmcIocCmd = MmcIocCmd::new(1, SD_GEN_CMD, 
        cmd56_arg, COMMAND_FLAGS_CMD56_WRITE, &data_out);

    unsafe {
        let res = mmc_ioc_cmd_rw(fdesc, &mut command);
        dbg!(command);
        if res.is_ok() {
            dump_buf(&data_out);
        }
        return res;
    }
    // printf("\"idata.response[]\":\"0x%02x 0x%02x 0x%02x 0x%02x\",\n", idata.response[0], idata.response[1], idata.response[2], idata.response[3]);
}


fn dump_buf(buf: &SDBlock) {
    println!("=== Begin buffer dump ===");
    for i in 0..buf.len() {
        print!("{:02X?} ", buf[i]);
        if (i+1) % 16 == 0 {
            println!();
        }
    }
    println!("=== End buffer dump ===");
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Usage: sdmon <device>");
        process::exit(0);
    }

    let device: &String = &args[1];

    dbg!(device);

    let mut _data_in: SDBlock = [0; SD_BLOCK_SIZE];


    let ff = File::options().read(true).write(true).open(device);
    let fl = ff.unwrap();
    let fd = fl.as_fd();
    let rfd = fd.as_raw_fd();

    let cmd56_data_in_res = cmd56_data_in(rfd, 0x00000001, &_data_in);

    match cmd56_data_in_res {
        Ok(res) => print!("Ok: {}", res),
        Err(err) => print!("Error: {}", err),
    }

    if cmd56_data_in_res.is_ok() {
        if LongsysSDParser::check_signature(&_data_in) {
            LongsysSDParser::dump_data(&_data_in);
        }
        if SandiskSDParser::check_signature(&_data_in) {
            SandiskSDParser::dump_data(&_data_in);
        }
        process::exit(0);
    }

    let cmd56_write_res = cmd56_write(rfd, 0x00000010);

    match cmd56_write_res {
        Ok(res) => println!("cmd56_write Ok: {}", res),
        Err(err) => println!("cmd56_write Error: {}", err),
    }

    if cmd56_write_res.is_err() {
        println!("CMD56 1st CALL FAILED: {}", cmd56_write_res.err().unwrap());
    }

    let cmd56_read_smart_data_res = cmd56_data_in(rfd, 0x00000021, &_data_in);

    match cmd56_read_smart_data_res {
        Ok(res) => println!("cmd56_read_smart_data_res Ok: {}", res),
        Err(err) => println!("cmd56_read_smart_data_res Error: {}", err),
    }

    if cmd56_read_smart_data_res.is_err() {
        println!("CMD56 2nd CALL FAILED: {}", cmd56_read_smart_data_res.err().unwrap());
        process::exit(0);
    }
}