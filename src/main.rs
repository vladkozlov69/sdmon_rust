use std::env;
use std::os::fd::AsFd;
use mmc_ioc_cmd::{
    cmd56_data_in,
    cmd56_write,
    dump_buf,
    SDBlock,
    GetInstance
};
use parsers::{SDParser, get_parsers, get_smartdata_parser};


use std::fs::File;
use std::os::fd::AsRawFd;
use std::process;

mod mmc_ioc_cmd;
mod parsers;



fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Usage: sdmon <device>");
        process::exit(0);
    }

    let mut device: String = "".to_string();
    let mut debug_flag = false;

    for arg in args {
        if arg.starts_with('/')
        {
            device.clone_from(&arg);
        }
        if arg == "-d" {
            debug_flag = true;
        }
    }

    dbg!(&device);

    let ff = File::options().read(true).write(true).open(&device);

    if ff.is_err() {
        println!("Device {} opening error: {}", &device, ff.err().unwrap());
        process::exit(1);
    }

    let fl = ff.unwrap(); // TODO check result here
    let fd = fl.as_fd();
    let rfd = fd.as_raw_fd();

    let cmds: [u32; 6] = [
        0x00000001, // Sandisk, Longsys
        0x110005fb, // Micron
        0x53420001, // Swissbit 
        0x110005F9, // ADATA
        0x110005FD, // Longsys Industrial M9H
        0x11000001  // ATP Industrial 
    ];

    let mut _data_in: SDBlock = SDBlock::get_instance();

    for cmd  in cmds {

        let cmd56_data_in_res = cmd56_data_in(rfd, cmd, &_data_in, debug_flag);

        if cmd56_data_in_res.is_ok() {
            let parsers_vec: Vec<Box<dyn SDParser>> = get_parsers();

            for parser in parsers_vec {
                if parser.check_signature(&_data_in)
                {
                    parser.dump_data(&_data_in);
                    process::exit(0);
                }
            }

            println!("Command {:010X?} succeeded but no parser available", cmd);
            dump_buf(&_data_in);
        }
        else {
            println!("Command {:010X?} failed", cmd);
        }
    }

    let cmd56_write_res = cmd56_write(rfd, 0x00000010, debug_flag);

    if cmd56_write_res.is_err() {
        println!("CMD56 1st CALL FAILED: {}", cmd56_write_res.err().unwrap());
    }

    let cmd56_read_smart_data_res = cmd56_data_in(rfd, 0x00000021, &_data_in, debug_flag);

    if cmd56_read_smart_data_res.is_err() {
        println!("CMD56 2nd CALL FAILED: {}", cmd56_read_smart_data_res.err().unwrap());
        process::exit(0);
    }
    else {
        get_smartdata_parser().dump_data(&_data_in)
    }
}