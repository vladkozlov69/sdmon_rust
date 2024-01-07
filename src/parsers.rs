use super::mmc_ioc_cmd::SDBlock;
use std::str;

pub trait SDParser {
    fn check_signature(&self, _block: &SDBlock) -> bool {
        return false;
    }

    fn dump_data(&self, _block: &SDBlock) {

    }
}

fn nb16(val1: u8, val2: u8) -> u16 {
    return ((val1 as u16) << 8) | (val2 as u16);
}

fn nb32(val1: u8, val2: u8, val3: u8, val4:u8) -> u32 {
    return ((val1 as u32) << 24) | 
           ((val2 as u32) << 16) | 
           ((val3 as u32) << 8)  | 
           (val4 as u32);
}

fn nb64(val1: u8, val2: u8, val3: u8, val4:u8, val5: u8, val6: u8, val7: u8, val8:u8) -> u64 {
    return ((val1 as u64) << 56) | 
           ((val2 as u64) << 48) | 
           ((val3 as u64) << 40) | 
           ((val4 as u64) << 32) |
           ((val5 as u64) << 24) | 
           ((val6 as u64) << 16) | 
           ((val7 as u64) << 8)  | 
           (val8 as u64);
}

fn nword_to_u32(block: &SDBlock, offset: usize) -> u32 {
    return nb32(block[offset+3], block[offset+2], block[offset+1], block[offset]);
}

fn nword_to_u64(block: &SDBlock, offset: usize) -> u64 {
    return nb64(block[offset+7], block[offset+6],block[offset+5],block[offset+4], 
                block[offset+3], block[offset+2], block[offset+1], block[offset]);
}

pub struct LongsysSDParser;
pub struct SandiskSDParser;
pub struct MicronSDParser;
pub struct SwissbitSDParser;
pub struct SmartDataSDParser;

impl SDParser for LongsysSDParser {
    fn check_signature(&self, block: &SDBlock) -> bool {
        return block[0] == 0x70 && block[1] == 0x58;
    }

    fn dump_data(&self, block: &SDBlock) {
        println!("Card type: Longsys");
        println!("SMARTVersions: {}",                   nword_to_u32(block, 4));
        println!("sizeOfDevSMART: {}",                  nword_to_u32(block, 12));
        println!("originalBadBlock: {}",                nword_to_u32(block, 16));
        println!("increaseBadBlock: {}",                nword_to_u32(block, 20));
        println!("writeAllSectNum: {} Sector(512Byte)", nword_to_u64(block, 24));
        println!("replaceBlockLeft: {}",                nword_to_u32(block, 32));
        println!("degreOfWear: {} Cycle",               f64::from(nword_to_u32(block, 36))/1000.0);
        println!("sectorTotal: {}",                     nword_to_u32(block, 40));
        println!("remainLifeTime: {}%",                 nword_to_u32(block, 44));
        println!("remainWrGBNum: {}TB",                 f64::from(nword_to_u32(block, 48))/1024.0);
        println!("lifeTimeTotal: {} Cycle",             nword_to_u32(block, 52));
        println!("phyWrGBNum: {}TB",                    f64::from(nword_to_u32(block, 56))/1024.0);
    }
}

impl SDParser for SandiskSDParser {
    fn check_signature(&self, block: &SDBlock) -> bool {
        return block[0] == 0x44 && (block[1] == 0x53 || block[1] == 0x57);
    }

    fn dump_data(&self, block: &SDBlock) {
        let manufacture_yymmdd = str::from_utf8(&block[2..2+6]).unwrap();
        let product_string = str::from_utf8(&block[49..49+32]).unwrap();
        const TAG_SIZE: usize = 431 - 405 + 1;
        let mut tag_bytes: [u8; TAG_SIZE] = [0; TAG_SIZE];
        tag_bytes.clone_from_slice(&block[405..432]);
        for i in 0..TAG_SIZE-1 {
            if tag_bytes[i] < 0x20 || tag_bytes[i] > 0x7F
            {
                tag_bytes[i] = b'_';
            }
        }
        let tag_string = str::from_utf8(&tag_bytes).unwrap();
        
        if block[1] == 0x57 {
            println!("Card type: Western Digital");
        } else {
            println!("Card type: Sandisk");
        }
        
        println!("manufactureYYMMDD: {}", manufacture_yymmdd);
        println!("healthStatusPercentUsed: {}", block[8]);
        println!("featureRevision: {}", block[11]);
        println!("generationIdentifier: {}", block[14]);
        println!("productString: {}", product_string);
        println!("power-on times: {}", nb32(0, 0, block[25], block[26]));
        println!("Tag: {}", tag_string);
        /*
1. SanDisk Industrial, compared to the data manual, adds 26L-24H, data name: power-on times
2. SanDisk Industrial, compared to the data manual, adds 405-424, 20 Bytes, data name: product code, ASCII format
3. SanDisk Industrial, compared to the data manual, adds 426-431, 6 Bytes, data name: product serial number, HEX format
         */
    }
}

impl SDParser for MicronSDParser {
    fn check_signature(&self, block: &SDBlock) -> bool {
        return block[0] == 0x4d && block[1] == 0x45;
    }

    fn dump_data(&self, block: &SDBlock) {
        println!("Card type: Micron");
        println!("Percentange step utilization: {}", block[7]);
        println!("TLC area utilization: {}", block[8]);
        println!("SLC area utilization: {}", block[9]);
    }
}

impl SDParser for SwissbitSDParser {
    fn check_signature(&self, block: &SDBlock) -> bool {
        return block[0] == 0x53 && block[1] == 0x77;
    }

    fn dump_data(&self, block: &SDBlock) {
        println!("Card type: Swissbit Micron");

        println!("fwVersion: [{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}]", block[32], block[33], block[34], block[35], block[36], block[37], block[38], block[39], block[40], block[41], block[42], block[43], block[44], block[45], block[46], block[47]); // show char
        println!("User area rated cycles: {}", nb32(block[48], block[49], block[50], block[51])); 
        println!("User area max cycle cnt: {}", nb32(block[52], block[53], block[54], block[55]));
        println!("User area total cycle cnt: {}", nb32(block[56], block[57], block[58], block[59]));
        println!("User area average cycle cnt: {}", nb32(block[60], block[61], block[62], block[63]));
        println!("System area max cycle cnt: {}", nb32(block[68], block[69], block[70], block[71]));
        println!("System area total cycle cnt: {}", nb32(block[72], block[73], block[74], block[75]));
        println!("System area average cycle cnt: {}", nb32(block[76], block[77], block[78], block[79]));
        println!("Remaining Lifetime Percent: {}%", block[80]);
        match block[86]
        {
            0x00 => println!("Speed mode: Default speed"),
            0x01 => println!("Speed mode: High speed"),
            0x10 => println!("Speed mode: SDR12 speed"),
            0x11 => println!("Speed mode: SDR25 speed"),
            0x12 => println!("Speed mode: SDR50 speed"),
            0x14 => println!("Speed mode: DDR50 speed"),
            0x18 => println!("Speed mode: SDR104 speed"),
            _ => println!("Speed mode: unknown ({})", block[86]),
        }
        match block[87]
        {
            0x00 => println!("Bus width: 1 bit"),
            0x10 => println!("Bus width: 4 bits"),
            _ => println!("Bus width: unknown ({})", block[87]),
        }
        println!("User area spare blocks cnt: {}", nb32(block[88], block[89], block[90], block[91]));
        println!("System area spare blocks cnt: {}", nb32(block[92], block[93], block[94], block[95]));
        println!("User area runtime bad blocks cnt: {}", nb32(block[96], block[97], block[98], block[99]));
        println!("System area runtime bad blocks cnt: {}", nb32(block[100], block[101], block[102], block[103]));
        println!("User area refresh cnt: {}", nb32(block[104], block[105],block[106], block[107]));
        println!("System area refresh cnt: {}", nb32(block[108], block[109],block[110], block[111]));
        println!("Interface crc cnt: {}", nb32(block[112], block[113],block[114], block[115]));
        println!("Power cycle cnt: {}", nb32(block[116], block[117], block[118], block[119]));
    } 
}

impl SDParser for SmartDataSDParser {
    fn check_signature(&self, block: &SDBlock) -> bool {
        return (block[0] != 0x70 || block[1] != 0x58) && (block[0] != 0x44 || (block[1] != 0x53 || block[1] != 0x57));
    }

    fn dump_data(&self, block: &SDBlock) {
        let mut initial_bad_block_count: u16 = 0;
        let mut later_bad_block_count: u16 = 0;

        for i in 32..63 {
            initial_bad_block_count = initial_bad_block_count + block[i] as u16;
        }

        for i in 184..215 {
            later_bad_block_count = later_bad_block_count + block[i] as u16;
        }

        println!("Card type: Generic Smart-capable SD");
        println!("flashId: [{:02X?},{:02X?},{:02X?},{:02X?},{:02X?},{:02X?},{:02X?},{:02X?},{:02X?}]",
            block[0], block[1], block[2], block[3], block[4], block[5], block[6], block[7], block[8]);
        println!("icVersion: [{:02X?},{:02X?}]", block[9], block[10]);
        println!("fwVersion: [{},{}]", block[11], block[12]); // show in decimal
        println!("ceNumber: {:02X?}", block[14]);
        println!("spareBlockCount: {}", nb16(block[17], block[16]));
        println!("initialBadBlockCount: {}", initial_bad_block_count);
        println!("goodBlockRatePercent: {}", nb16(block[64], block[65]) as f32 / 100.0);
        println!("totalEraseCount: {}", nb32(block[80], block[81], block[82], block[83]));
        println!("enduranceRemainLifePercent: {}", nb16(block[97], block[97]) as f32 / 100.0);
        println!("avgEraseCount: {}", nb32(block[104], block[105], block[98], block[99]));
        println!("minEraseCount: {}", nb32(block[106], block[107], block[100], block[101]));
        println!("maxEraseCount: {}", nb32(block[108], block[109], block[102], block[103]));
        println!("powerUpCount: {}", nb32(block[112], block[113], block[114], block[115]));
        println!("abnormalPowerOffCount: {}", nb16(block[128], block[129]));
        println!("totalRefreshCount: {}", nb16(block[160], block[161]));
        println!("productMarker: [{:02X?} {:02X?} {:02X?} {:02X?} {:02X?} {:02X?} {:02X?} {:02X?}]",
            block[176], block[177], block[178], block[179], block[180], block[181], block[182], block[183]);
        println!("laterBadBlockCount: {}", later_bad_block_count);

    }
}

pub fn get_parsers() -> Vec<Box<dyn SDParser>> {
    vec![
        Box::new(LongsysSDParser{}), 
        Box::new(SandiskSDParser{}), 
        Box::new(MicronSDParser{}), 
        Box::new(SwissbitSDParser{})]
}

pub fn get_smartdata_parser() -> Box<dyn SDParser> {
    Box::new(SmartDataSDParser{})
}
