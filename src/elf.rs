use std::fs;
use std::io::{prelude::*, SeekFrom};

pub struct Elf32Hdr {
    pub e_ident: Vec<u8>,
    pub e_type: u16,
    pub e_machine: u16,
    pub e_version: u32,

    pub e_entry: u32,
    pub e_phoff: u32,
    pub e_shoff: u32,

    pub e_flags: u32,

    pub e_ehsize: u16,

    pub e_phentsize: u16,
    pub e_phnum: u16,

    pub e_shentsize: u16,
    pub e_shnum: u16,

    pub e_shstrndx: u16,
}

#[derive(Copy, Clone)]
pub struct Elf32Phdr {
    pub p_type: u32,

    pub p_offset: u32,

    pub p_vaddr: u32,
    pub p_paddr: u32,

    pub p_filesz: u32,
    pub p_memsz: u32,

    pub p_flags: u32,

    pub p_align: u32,
}

pub struct RawELF {
    pub data: Vec<u8>,

    pub base_addr: u32,
    pub entry_point: u32,
}

#[allow(dead_code)]
fn read_u32(mut reader: impl Read + Seek) -> u32 {
    let mut temp = [0u8;4];
    reader.read_exact(&mut temp).ok();
    return u32::from_be_bytes(temp);
}

#[allow(dead_code)]
fn read_u16(mut reader: impl Read + Seek) -> u16 {
    let mut temp = [0u8;2];
    reader.read_exact(&mut temp).ok();
    return u16::from_be_bytes(temp);
}

#[allow(dead_code)]
fn read_u8s_from_offset(mut reader: impl Read + Seek, size: usize, offset: u64) -> Vec<u8> {
    let mut temp = vec![0u8;size];
    reader.seek(SeekFrom::Start(offset)).ok();
    reader.read_exact(&mut temp).ok();
    return temp;
}

#[allow(dead_code)]
fn read_u8s(mut reader: impl Read + Seek, size: usize) -> Vec<u8> {
    let mut temp = vec![0u8;size];
    reader.read_exact(&mut temp).ok();
    return temp;
}

fn read_elf32_hdr(mut reader: impl Read + Seek) -> Elf32Hdr {
    return Elf32Hdr {
        e_ident:        read_u8s(&mut reader, 16),
        e_type:         read_u16(&mut reader),
        e_machine:      read_u16(&mut reader),
        e_version:      read_u32(&mut reader),

        e_entry:        read_u32(&mut reader),
        e_phoff:        read_u32(&mut reader),
        e_shoff:        read_u32(&mut reader),

        e_flags:        read_u32(&mut reader),

        e_ehsize:       read_u16(&mut reader),

        e_phentsize:    read_u16(&mut reader),
        e_phnum:        read_u16(&mut reader),
        
        e_shentsize:    read_u16(&mut reader),
        e_shnum:        read_u16(&mut reader),

        e_shstrndx:     read_u16(&mut reader)
    }
}

fn read_elf32_prg_hdr(mut reader: impl Read + Seek) -> Elf32Phdr {
    return Elf32Phdr {
        p_type:     read_u32(&mut reader),

        p_offset:   read_u32(&mut reader),

        p_vaddr:    read_u32(&mut reader),
        p_paddr:    read_u32(&mut reader),

        p_filesz:   read_u32(&mut reader),
        p_memsz:    read_u32(&mut reader),

        p_flags:    read_u32(&mut reader),

        p_align:    read_u32(&mut reader),
    }
}

fn verify_elf32_hdr(header: &Elf32Hdr) {
    if  header.e_ident.len() < 16 ||
        header.e_ident[4] != 1 ||
        header.e_ident[6] != 1 ||
        header.e_version != 1 ||
        header.e_type != 2
    {
        panic!("Invalid ELF File!");
    }

    if header.e_machine != 20 {
        panic!("Not PowerPC ELF!");
    }

    if header.e_phnum == 0 || header.e_phoff == 0 {
        panic!("This ELF got nothing!");
    }
}

#[allow(dead_code)]
pub fn turn_elf_to_raw(file_name: &String, image_size: usize, base_addr: u32) -> RawELF {
    let mut file = fs::File::open(&file_name).expect("File either not found or failed to open!!");

    // Read ELF header
    let elf_header = read_elf32_hdr(&file);
    verify_elf32_hdr(&elf_header);

    // Read program headers
    let mut elf_prg_hdr = vec![elf_prg_hdr_default();elf_header.e_phnum as usize];
    file.seek(SeekFrom::Start(elf_header.e_phoff as u64)).ok();

    for i in 0..elf_header.e_phnum as usize {
        elf_prg_hdr[i] = read_elf32_prg_hdr(&file);
    }

    // Copy data to raw
    let mut raw_image = raw_elf_default(image_size);
    for i in 0..elf_header.e_phnum as usize {
        let vaddr = elf_prg_hdr[i].p_vaddr as usize;
        let memsz = elf_prg_hdr[i].p_memsz as usize;
        let filesz = elf_prg_hdr[i].p_filesz as usize;
        let offset = elf_prg_hdr[i].p_offset as usize;

        if memsz != 0 && vaddr != 0 && filesz != 0 && filesz <= memsz {
            file.seek(SeekFrom::Start(offset as u64)).ok();
            let data = read_u8s(&file, memsz);
            raw_image.data[vaddr - base_addr as usize..vaddr - base_addr as usize + memsz].copy_from_slice(&data);
        }
    }

    raw_image.base_addr = base_addr;
    raw_image.entry_point = elf_header.e_entry;

    return raw_image;
}

#[allow(dead_code)]
pub fn raw_elf_default(size: usize) -> RawELF {
    return RawELF {
        base_addr:      0,
        entry_point:    0,

        data:           vec![0;size],
    };
}

pub fn elf_prg_hdr_default() -> Elf32Phdr {
    return Elf32Phdr {
        p_type:     0,

        p_offset:   0,

        p_vaddr:    0,
        p_paddr:    0,

        p_filesz:   0,
        p_memsz:    0,

        p_flags:    0,

        p_align:    0,
    };
}


