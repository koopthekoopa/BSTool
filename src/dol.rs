use std::fs;
use std::io::{prelude::*};

pub const TEXT_COUNT : usize = 7;
pub const DATA_COUNT : usize = 11;

pub const HEADER_LENGTH : usize = 0x100;

struct DOLImage {
    pub text_off:    Vec<u32>,
    pub data_off:    Vec<u32>,

    pub text_addr:   Vec<u32>,
    pub data_addr:   Vec<u32>,

    pub text_size:   Vec<u32>,
    pub data_size:   Vec<u32>,

    pub bss_addr:    u32,
    pub bss_size:    u32,

    pub entry_point: u32,
}

fn write_section_info(mut writer: impl Write, for_text: &Vec<u32>, for_data: &Vec<u32>) {
    for i in 0..TEXT_COUNT {
        writer.write(&u32::to_be_bytes(for_text[i])).ok();
    }
    for i in 0..DATA_COUNT {
        writer.write(&u32::to_be_bytes(for_data[i])).ok();
    }
}

fn write_padding(mut writer: impl Write + Seek, stopper: u32) {
    let cur_off = writer.stream_position().unwrap() as usize;
    let size = stopper as usize - cur_off;
    let temp : Vec<u8> = vec![0;size];
    writer.write(&temp).ok();
}

#[allow(dead_code)]
pub fn turn_raw_to_dol(file_name: &String,
                        raw_data: &Vec<u8>,
                        text_addr: &Vec<u32>,
                        text_size: &Vec<u32>,
                        data_addr: &Vec<u32>,
                        data_size: &Vec<u32>,
                        bss_addr: &Vec<u32>,
                        bss_size: &Vec<u32>,
                        entry_point: u32,
                        base_addr: u32,) {
    let mut file = fs::File::create(&file_name).expect("File failed to create!!");
    let mut dol = default();

    for i in 0..text_addr.len() {
        dol.text_off[i] = text_addr[i] - base_addr + HEADER_LENGTH as u32;
        dol.text_addr[i] = text_addr[i];
        dol.text_size[i] = text_size[i];
    }

    for i in 0..data_addr.len() {
        dol.data_off[i] = data_addr[i] - base_addr + HEADER_LENGTH as u32;
        dol.data_addr[i] = data_addr[i];
        dol.data_size[i] = data_size[i];
    }

    let first = *bss_addr.first().unwrap();
    let last = *bss_addr.last().unwrap();
    let last_size = *bss_size.last().unwrap();
    dol.bss_addr = first;
    dol.bss_size = (last + last_size) - first;

    dol.entry_point = entry_point;

    // header time!!

    write_section_info(&file, &dol.text_off, &dol.data_off);
    write_section_info(&file, &dol.text_addr, &dol.data_addr);
    write_section_info(&file, &dol.text_size, &dol.data_size);

    file.write(&u32::to_be_bytes(dol.bss_addr)).ok();
    file.write(&u32::to_be_bytes(dol.bss_size)).ok();
    file.write(&u32::to_be_bytes(dol.entry_point)).ok();

    write_padding(&file, HEADER_LENGTH as u32);

    file.write(&raw_data).ok();
}

#[allow(dead_code)]
fn default() -> DOLImage {
    return DOLImage {
        text_off:    vec![0;TEXT_COUNT],
        data_off:    vec![0;DATA_COUNT],

        text_addr:   vec![0;TEXT_COUNT],
        data_addr:   vec![0;DATA_COUNT],

        text_size:   vec![0;TEXT_COUNT],
        data_size:   vec![0;DATA_COUNT],

        bss_addr:    0,
        bss_size:    0,

        entry_point: 0,
    };
}


