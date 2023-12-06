#![allow(dead_code)]

use super::mmc_ioc_cmd::SDBlock;
use std::str;

pub trait SDParser {
    fn check_signature(_block: &SDBlock) -> bool {
        return false;
    }

    fn dump_data(_block: &SDBlock) {

    }
}

fn nib16(block: &SDBlock, offset: usize, shift: i8) -> u16 {
    return (block[offset] as u16) << shift;
}

fn nib32(block: &SDBlock, offset: usize, nr: usize, shift: i8) -> u32 {
    return (block[offset+nr] as u32) << shift;
}

fn nib64(block: &SDBlock, offset: usize, nr: usize, shift: i8) -> u64 {
    return (block[offset+nr] as u64) << shift;
}

fn nword_to_u32(block: &SDBlock, offset: usize) -> u32 {
    nib32(block, offset, 3, 24) |
    nib32(block, offset, 2, 16) |
    nib32(block, offset, 1, 8)  |
    nib32(block, offset, 0, 0)
}

fn nword_to_u64(block: &SDBlock, offset: usize) -> u64 {
    nib64(block, offset, 7, 56) |
    nib64(block, offset, 6, 48) |
    nib64(block, offset, 5, 40) |
    nib64(block, offset, 4, 32) |
    nib64(block, offset, 3, 24) |
    nib64(block, offset, 2, 16) |
    nib64(block, offset, 1, 8)  |
    nib64(block, offset, 0, 0)
}

pub struct LongsysSDParser;
pub struct SandiskSDParser;
pub struct SmartDataSDParser;

impl SDParser for LongsysSDParser {
    fn check_signature(block: &SDBlock) -> bool {
        return block[0] == 0x70 && block[1] == 0x58;
    }

    fn dump_data(block: &SDBlock) {
        println!("Longsys");
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
    fn check_signature(block: &SDBlock) -> bool {
        return block[0] == 0x44 && block[1] == 0x53;
    }

    fn dump_data(block: &SDBlock) {
        println!("Sandisk");

        //let manufacture_yymmdd: Vec<u8> = vec![block[2..8]];
        // let mm = &block[2..8];
        let manufacture_yymmdd = str::from_utf8(&block[2..2+6]).unwrap();
        /*
        strncpy(tmpstr, (char *)&data_in[2], 6);
        tmpstr[6] = 0;
        // printf("\"manufactureYYMMDD\": \"%s\",\n", tmpstr);
        */
        println!("manufactureYYMMDD: {}", manufacture_yymmdd);
        println!("healthStatusPercentUsed: {}", block[8]);
        println!("featureRevision: {}", block[11]);
        println!("generationIdentifier: {}", block[14]);

        let product_string = str::from_utf8(&block[49..49+32]).unwrap();
        println!("productString: {}", product_string);
        /*
        strncpy(tmpstr, (char *)&data_in[49], 32);
        tmpstr[32] = 0;
        printf("\"productString\": \"%s\",\n", tmpstr);
        */
    }
}

impl SDParser for SmartDataSDParser {
    fn check_signature(block: &SDBlock) -> bool {
        return (block[0] != 0x70 || block[1] != 0x58) && (block[0] != 0x44 || block[0] != 0x53);
    }

    fn dump_data(block: &SDBlock) {
        println!("Generic Smart-capable SD");
        /*
        printf("\"flashId\": "
        "[\"0x%02x\",\"0x%02x\",\"0x%02x\",\"0x%02x\",\"0x%02x\",\"0x%02x\","
        "\"0x%02x\",\"0x%02x\",\"0x%02x\"],\n",
        data_in[0], data_in[1], data_in[2], data_in[3], data_in[4], data_in[5], data_in[6], data_in[7], data_in[8]);
 printf("\"icVersion\": [\"0x%02x\",\"0x%02x\"],\n", data_in[9], data_in[10]);
 printf("\"fwVersion\": [%02d,%02d],\n", data_in[11],
        data_in[12]); // show in decimal
 printf("\"ceNumber\": \"0x%02x\",\n", data_in[14]);
*/
        println!("spareBlockCount: {}", nib16(block, 16, 8) + nib16(block, 17, 0));

 /* 
 // sum up to get initial bad block count
 sum = 0;
 for (i = 32; i < 64; i++)
   sum += data_in[i];
 printf("\"initialBadBlockCount\": %ld,\n", sum);

 printf("\"goodBlockRatePercent\": %2.2f,\n", (float)((float)((int)((data_in[64] << 8) + data_in[65])) / 100));

 printf("\"totalEraseCount\": %ld,\n", (long)((data_in[80] << 24) + (data_in[81] << 16) + (data_in[82] << 8) + data_in[83]));

 printf("\"enduranceRemainLifePercent\": %2.2f,\n", (float)((float)((int)((data_in[96] << 8) + data_in[97])) / 100));

 printf("\"avgEraseCount\": %ld,\n", (long)((data_in[104] << 24) + (data_in[105] << 16) + (data_in[98] << 8) + data_in[99]));
 printf("\"minEraseCount\": %ld,\n", (long)((data_in[106] << 24) + (data_in[107] << 16) + (data_in[100] << 8) + data_in[101]));
 printf("\"maxEraseCount\": %ld,\n", (long)((data_in[108] << 24) + (data_in[109] << 16) + (data_in[102] << 8) + data_in[103]));

 printf("\"powerUpCount\": %ld,\n", (long)((data_in[112] << 24) + (data_in[113] << 16) + (data_in[114] << 8) + data_in[115]));
 printf("\"abnormalPowerOffCount\": %d,\n", (int)((data_in[128] << 8) + data_in[129]));
 printf("\"totalRefreshCount\": %d,\n", (int)((data_in[160] << 8) + data_in[161]));
 printf("\"productMarker\": "
        "[\"0x%02x\",\"0x%02x\",\"0x%02x\",\"0x%02x\",\"0x%02x\",\"0x%02x\","
        "\"0x%02x\",\"0x%02x\"],\n",
        data_in[176], data_in[177], data_in[178], data_in[179], data_in[180], data_in[181], data_in[182], data_in[183]);

 sum = 0;
 for (i = 184; i < 216; i++)
   sum += data_in[i];
 printf("\"laterBadBlockCount\": %ld,\n", sum);
*/
    }
}

