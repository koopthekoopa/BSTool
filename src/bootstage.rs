use std::fs;
use std::io::{prelude::*, SeekFrom};

// 0 = text
// 1 = data
// TODO: make it read an linkerscript file instead
const LINK_ORDER: [i32; 10] = [
    0,
    1,
    1,
    0,
    1,
    1,
    1,
    1,
    1,
    1
];

const STUB_DEFAULT_ADDR : u32 = 0x81340000;
const STUB_DEFAULT_SIZE : u32 = 0x00010000;

const INIT_MEM_BOUND_START : u32 = 0x81330000;
const UNINIT_MEM_BOUND_START : u32 = 0x81080000;
const MEM_BOUND_END : u32 = 0x816A0000; // AFAIK no existing boot stage exceeds that boundary.

const BS2_PAD : u32 = 0x20;

pub const TEXT_COUNT : usize = 2;
pub const DATA_COUNT : usize = 8;
pub const BSS_COUNT : usize = 3;

pub const HEADER_LENGTH : usize = 0x100;

#[derive(Copy, Clone)]
pub struct BSImageBSS {
    pub addr: u32,
    pub size: u32,
}

pub struct BSImage {
    pub bs1_addr: u32,
    pub bs1_len: u32,
    pub bs1_data: Vec<u8>,

    pub bs2_addr: u32,
    pub bs2_len: u32,
    pub bs2_data: Vec<u8>,

    pub stub_addr: u32,
    pub stub_len: u32,

    pub unk_stuff: Vec<u8>,

    pub bs1_entry: u32,
    pub bs2_entry: u32,

    pub text_addr: Vec<u32>,
    pub text_len: Vec<u32>,

    pub data_addr: Vec<u32>,
    pub data_len: Vec<u32>,

    pub bss_addr: Vec<u32>,
    pub bss_len: Vec<u32>,
}

#[allow(dead_code, unused)]
fn read_u32(mut reader: impl Read + Seek, offset: u64) -> u32 {
    let mut temp = [0u8;4];

    reader.seek(SeekFrom::Start(offset)).ok();
    reader.read_exact(&mut temp).ok();

    return u32::from_be_bytes(temp);
}

#[allow(dead_code)]
fn read_u32_from_buf(buffer: &Vec<u8>, offset: u32) -> u32 {
    let temp = [buffer[offset as usize],
                buffer[offset as usize + 1],
                buffer[offset as usize + 2],
                buffer[offset as usize + 3]];
    return u32::from_be_bytes(temp);
}

#[allow(dead_code)]
fn write_u32_from_buf(buffer: &mut Vec<u8>, offset: u32, value: u32) {
    let temp = u32::to_be_bytes(value);
    buffer[offset as usize] = temp[0];
    buffer[offset as usize + 1] = temp[1];
    buffer[offset as usize + 2] = temp[2];
    buffer[offset as usize + 3] = temp[3];
}

#[allow(dead_code)]
fn find_u32_from_buf(buffer: &Vec<u8>, value: u32, offset: u32) -> u32 {
    let mut done = 0;
    let mut curr_offset = offset as usize;

    while done == 0 {
        let temp = [buffer[curr_offset],
                    buffer[curr_offset + 1],
                    buffer[curr_offset + 2],
                    buffer[curr_offset + 3]];
        if u32::from_be_bytes(temp) == value {
            done = 1;
        }
        else {
            curr_offset += 4;
        }
    }

    return curr_offset as u32;
}

#[allow(dead_code)]
fn find_u32_from_buf_range(buffer: &Vec<u8>, min: u32, max: u32, offset: u32) -> u32 {
    let mut done = 0;
    let mut curr_offset = offset as usize;

    while done == 0 {
        let temp = [buffer[curr_offset],
                    buffer[curr_offset + 1],
                    buffer[curr_offset + 2],
                    buffer[curr_offset + 3]];
        if u32::from_be_bytes(temp) >= min && u32::from_be_bytes(temp) <= max {
            done = 1;
        }
        else {
            curr_offset += 4;
        }
    }

    return curr_offset as u32;
}

#[allow(dead_code)]
fn read_u8s(mut reader: impl Read + Seek, size: usize, offset: u64) -> Vec<u8> {
    let mut temp = vec![0u8;size];

    reader.seek(SeekFrom::Start(offset)).ok();
    reader.read_exact(&mut temp).ok();

    return temp;
}

#[allow(dead_code)]
fn read_u8s_from_buf(buffer: &Vec<u8>, size: usize, offset: u32) -> Vec<u8> {
    let mut temp = vec![0u8;size];

    for i in 0..size {
        temp[i] = buffer[offset as usize + i];
    }

    return temp;
}

fn write_blank(mut writer: impl Write + Seek, size: u32) {
    let temp : Vec<u8> = vec![0;size as usize];
    writer.write(&temp).ok();
}

fn verify_unk_data(image: &BSImage) -> bool {
    if image.unk_stuff.len() < 0x20 {
        return false;
    }

    if image.unk_stuff[0] == 0 || image.unk_stuff[1] == 0 {
        return false;
    }

    return true;
}

#[allow(dead_code)]
pub fn open_file(file_name: &String) -> BSImage {
    let file = fs::File::open(&file_name).expect("File either not found or failed to open!!");

    // Read BS2
    let mut new_image = BSImage {
        bs1_addr:   read_u32(&file, 0x48),
        bs1_len:    read_u32(&file, 0x90) - 4,
        bs1_data:   vec![0], // temp

        bs2_addr:   read_u32(&file, 0x64),
        bs2_len:    read_u32(&file, 0xAC),
        bs2_data:   vec![0], // temp

        stub_addr:  read_u32(&file, 0xD8),
        stub_len:   read_u32(&file, 0xDC),

        unk_stuff:  vec![0],

        bs1_entry:  read_u32(&file, 0xE0),
        bs2_entry:  read_u32(&file, 0x4FC),

        text_addr:  vec![0;TEXT_COUNT],
        text_len:   vec![0;TEXT_COUNT],

        data_addr:  vec![0;DATA_COUNT],
        data_len:   vec![0;DATA_COUNT],

        bss_addr:   vec![0;BSS_COUNT],
        bss_len:    vec![0;BSS_COUNT],
    };

    let bs1_off = read_u32(&file, 0x00);
    let mut bs2_off = read_u32(&file, 0x1C);

    let checker = read_u32(&file, bs2_off as u64);
    let checker2 = read_u32(&file, bs2_off as u64 + 0x08);

    if checker >= INIT_MEM_BOUND_START && checker <= MEM_BOUND_END && checker2 == 0x00000000 {
        new_image.unk_stuff = read_u8s(&file, BS2_PAD as usize, bs2_off as u64);

        bs2_off  += BS2_PAD;
        new_image.bs2_addr += BS2_PAD;
        new_image.bs2_len  -= BS2_PAD;
    }

    new_image.bs1_data = read_u8s(&file, new_image.bs1_len as usize, bs1_off as u64);
    new_image.bs2_data = read_u8s(&file, new_image.bs2_len as usize - 4, bs2_off as u64);

    // Read Section Info
    let rom_offset = find_u32_from_buf(&new_image.bs2_data, INIT_MEM_BOUND_START, 0);
    let mut read_off = rom_offset;
    let mut text_i = 0;
    let mut data_i = 0;
    for i in 0..TEXT_COUNT+DATA_COUNT {
        // Text symbol
        if LINK_ORDER[i] == 0 {
            new_image.text_addr[text_i] = read_u32_from_buf(&new_image.bs2_data, read_off);
            new_image.text_len[text_i] = read_u32_from_buf(&new_image.bs2_data, read_off + 0x08);
            text_i += 1;
        }
        // Data symbol
        else if LINK_ORDER[i] == 1 {
            new_image.data_addr[data_i] = read_u32_from_buf(&new_image.bs2_data, read_off);
            new_image.data_len[data_i] = read_u32_from_buf(&new_image.bs2_data, read_off + 0x08);
            data_i += 1;
        }
        read_off += 0x0C;
    }

    // Read BSS Section Info
    let mut bss_sec = vec![BSImageBSS{addr:0,size:0};BSS_COUNT];
    let bss_offset = find_u32_from_buf_range(&new_image.bs2_data, UNINIT_MEM_BOUND_START, MEM_BOUND_END, read_off);
    read_off = bss_offset;
    for i in 0..BSS_COUNT {
        bss_sec[i].addr = read_u32_from_buf(&new_image.bs2_data, read_off);
        bss_sec[i].size = read_u32_from_buf(&new_image.bs2_data, read_off + 0x04);
        read_off += 0x08;
    }

    // HACK: Order the BSS to fix relocating
    bss_sec.sort_by_key(|x| x.addr);
    read_off = bss_offset;
    if new_image.text_addr[0] >= bss_sec[0].size {
        bss_sec[0].size = new_image.text_addr[0] - bss_sec[0].addr;
    }
    for i in 0..BSS_COUNT {
        write_u32_from_buf(&mut new_image.bs2_data, read_off, bss_sec[i].addr);
        write_u32_from_buf(&mut new_image.bs2_data, read_off + 0x04, bss_sec[i].size);
        read_off += 0x08;
    }

    // Save the BSS
    for i in 0..BSS_COUNT {
        new_image.bss_addr[i] = bss_sec[i].addr;
        new_image.bss_len[i] = bss_sec[i].size;
    }

    return new_image;
}

#[allow(dead_code)]
pub fn create_file(file_name: &String, image: &BSImage) {
    let bs1_off = HEADER_LENGTH as u32;
    let bs2_off = HEADER_LENGTH as u32 + image.bs1_len + 4;
    let mut bs2_addr = image.bs2_addr;
    let mut bs2_len = image.bs2_len;

    if verify_unk_data(&image) {
        bs2_addr -= 0x20;
        bs2_len += 0x20;
    }

    let mut file = fs::File::create(&file_name).expect("File failed to create!!");

    // Offset
    file.write(&u32::to_be_bytes(bs1_off)).ok();
    write_blank(&file, 0x18);
    file.write(&u32::to_be_bytes(bs2_off)).ok();
    write_blank(&file, 0x28);

    // Address
    file.write(&u32::to_be_bytes(image.bs1_addr)).ok();
    write_blank(&file, 0x18);
    file.write(&u32::to_be_bytes(bs2_addr)).ok();
    write_blank(&file, 0x28);
    
    // Length
    file.write(&u32::to_be_bytes(image.bs1_len + 4)).ok();
    write_blank(&file, 0x18);
    file.write(&u32::to_be_bytes(bs2_len)).ok();
    write_blank(&file, 0x28);

    // Other stuff
    file.write(&u32::to_be_bytes(image.stub_addr)).ok();
    file.write(&u32::to_be_bytes(image.stub_len)).ok();

    file.write(&u32::to_be_bytes(image.bs1_entry)).ok();
    write_blank(&file, 0x1C);

    // BS1 (with entry point)
    file.write(&image.bs1_data).ok();
    file.write(&u32::to_be_bytes(image.bs2_entry)).ok();

    // BS2 (with entry point)
    if verify_unk_data(&image) {
        file.write(&image.unk_stuff).ok();
    }
    file.write(&image.bs2_data).ok();
}

#[allow(dead_code)]
pub fn default() -> BSImage {
    return BSImage {
        bs1_addr:   0,
        bs1_len:    0,
        bs1_data:   vec![0],

        bs2_addr:   0,
        bs2_len:    0,
        bs2_data:   vec![0],

        stub_addr:  STUB_DEFAULT_ADDR,
        stub_len:   STUB_DEFAULT_SIZE,

        unk_stuff:  vec![0],

        bs1_entry:  0,
        bs2_entry:  0,

        text_addr:  vec![0;TEXT_COUNT],
        text_len:   vec![0;TEXT_COUNT],

        data_addr:  vec![0;DATA_COUNT],
        data_len:   vec![0;DATA_COUNT],

        bss_addr:   vec![0;BSS_COUNT],
        bss_len:    vec![0;BSS_COUNT],
    };
}


