#[allow(dead_code, unused)]
use argp::{FromArgs};

pub mod bootstage;
pub mod dol;
pub mod elf;

/// Tool for IPL BootStage files
#[derive(FromArgs, PartialEq, Debug)]
struct ProcessArg {
    #[argp(subcommand)]
    processes: ProcessEnum,
}

#[derive(FromArgs, PartialEq, Debug)]
#[argp(subcommand)]
enum ProcessEnum {
    DTK(DTKArgs),
    CONVERT(ConvertArgs),
}

/// Convert BootStage to DOL file for DTK.
#[derive(FromArgs, PartialEq, Debug)]
#[argp(subcommand, name = "dtk")]
struct DTKArgs {
    /// Input BootStage file.
    #[argp(option, short = 'i')]
    in_file: String,
    
    /// Output DOL file.
    #[argp(option, short = 'o')]
    out_file: String,
}

/// Convert ELF to BootStage.
#[derive(FromArgs, PartialEq, Debug)]
#[argp(subcommand, name = "convert")]
struct ConvertArgs {
    /// Input ELF file for BS2.
    #[argp(option, short = 'i')]
    in_file: String,
    
    /// Base Bootstage file. (For meta data and BS1)
    #[argp(option, short = 'b')]
    base_file: String,

    /// Image Size of the BootStage
    #[argp(option, short = 's')]
    image_size: Option<usize>,

    /// Base address of the BootStage
    #[argp(option, short = 'a')]
    base_addr: Option<u32>,
    
    /// Output DOL file.
    #[argp(option, short = 'o')]
    out_file: String,
}

fn main() -> std::io::Result<()> {
    let args: ProcessArg = argp::parse_args_or_exit(argp::DEFAULT);
    match args.processes {
        ProcessEnum::DTK(le_args)     => bs_to_dtk(le_args.in_file, le_args.out_file)?,
        ProcessEnum::CONVERT(le_args) => {
            let image_size : usize = if let Some(v) = le_args.image_size { v } else { 0xFFFFFFFF };
            let base_addr  : u32   = if let Some(v) = le_args.base_addr  { v } else { 0xFFFFFFFF };
            elf_to_bs(le_args.base_file, le_args.in_file, le_args.out_file, image_size, base_addr)?
        },
    }
    Ok(())
}

fn bs_to_dtk(in_file: String, out_file: String) -> std::io::Result<()> {
    let image = bootstage::open_file(&in_file);
    dol::turn_raw_to_dol(&out_file,
                        &image.bs2_data,
                        &image.text_addr,
                        &image.text_len,
                        &image.data_addr,
                        &image.data_len,
                        &image.bss_addr,
                        &image.bss_len,
                         image.bs2_entry,
                         image.bs2_addr);
    Ok(())
}

fn elf_to_bs(base_file: String, in_file: String, out_file: String, image_size: usize, base_addr: u32) -> std::io::Result<()> {
    let base_image = bootstage::open_file(&base_file);

    let bs2_image_size = if image_size == 0xFFFFFFFF { base_image.bs2_len as usize } else { image_size };
    let bs2_base_addr = if base_addr == 0xFFFFFFFF { base_image.bs2_addr } else { base_addr };
    //println!("bs2_image_size: {:#08X}", bs2_image_size);
    //println!("bs2_base_addr: {:#08X}", bs2_base_addr);

    let mut output_image = base_image;

    let raw_elf_data = elf::turn_elf_to_raw(&in_file, bs2_image_size, bs2_base_addr);

    output_image.bs2_data   = raw_elf_data.data;
    output_image.bs2_addr   = bs2_base_addr;
    output_image.bs2_len    = bs2_image_size as u32;
    output_image.bs2_entry  = raw_elf_data.entry_point;

    bootstage::create_file(&out_file, &output_image);

    Ok(())
}


