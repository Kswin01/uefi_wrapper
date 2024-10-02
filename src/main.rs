#![no_main]
#![no_std]

#[macro_use]
mod util;

use core::{ffi::CStr, str};
use core::mem::MaybeUninit;
use log::info;
use uefi::{boot::{open_protocol_exclusive, AllocateType, MemoryType, ScopedProtocol}, mem::memory_map::MemoryMap, prelude::*, system, table::boot};
use uefi::{prelude::*, Guid};
use uefi::proto::loaded_image::LoadedImage;
use linked_list_allocator::LockedHeap;
use core::{arch::asm, slice};
use goblin::pe::{
    section_table::{IMAGE_SCN_MEM_EXECUTE, IMAGE_SCN_MEM_READ},
    PE,
};

#[no_mangle]
#[link_section = ".loader"]
#[used]
pub static bytes: [u8; 2535424] = [0; 2535424];

const load_addr_str: &str = "0x100000000";

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

#[entry]
fn main(_image_handle: Handle, system_table: SystemTable<Boot>) -> Status {
    let x = 1;
    let ptr_y = &x;
    uefi::helpers::init().unwrap();

    /* Allocating our own heap due to the absecnce of a heap allocator in no_std. */
    let heap_res = system_table.boot_services().allocate_pages(AllocateType::AnyPages, MemoryType::LOADER_DATA, 10).expect("Failed to alloc");

    unsafe {
        ALLOCATOR
            .lock()
            .init(heap_res as *mut u8, 10 * 4096);
    }

    /* Open the protocol */
    let proto: ScopedProtocol<LoadedImage> =
        open_protocol_exclusive(_image_handle).unwrap();
    info!("Successfully unwrapped protocol. fdsThis path {:?} is this info: {:?}", proto.file_path(), proto.info());

    let image_info: (*const core::ffi::c_void, u64) = proto.info();

    let load_addr: u64 = u64::from_str_radix(load_addr_str.strip_prefix("0x").unwrap(), 16).expect("Load address was not a valid hex string");
    /* Allocate the region that we will load the image into */
    // TODO: What if this conflicts with where UEFI has loaded us (or something else that UEFI is using?)
    // We should:
    //      1. Allocate a random region of memory with size that is big enough to store the image
    //      2. Load the image there
    //      3. Have an assembly routine at the start of the image that relocates it to where it is meant to be executing prior to jumping to main.
    let allocate_res = system_table.boot_services().allocate_pages(AllocateType::Address(load_addr), MemoryType::LOADER_DATA, (bytes.len() / 4096) + 1).expect("Failed to alloc");



    let sl = unsafe { slice::from_raw_parts(image_info.0 as *mut u8, image_info.1 as usize) };

    let pe_parsed: PE<'_> = PE::parse(sl).unwrap();

    for section in pe_parsed.sections {
        let name = unsafe{ CStr::from_ptr(section.name.as_ptr() as *const i8) };
        if name.to_str().unwrap() == ".mloader" {
            info!("BOOM::: {}", name.to_str().unwrap());
            info!("This is the vaddr: {:x}", image_info.0 as u64 + section.virtual_address as u64);
            /* Define the region as a slice so that we can access it */
            let load_region: &mut [u8] = unsafe {slice::from_raw_parts_mut(load_addr as *mut u8, section.virtual_size as usize)};

            let section_data: &mut [u8] = unsafe {slice::from_raw_parts_mut((image_info.0 as u64 + section.virtual_address as u64) as *mut u8, section.virtual_size as usize)};

            load_region.copy_from_slice(&section_data);

            for i in 0..100 {
                info!(" this is byte : {} ---- {}", i, load_region[i]);
            }
        }
    }

    /* Exit boot services  */
    unsafe{system_table.exit_boot_services(MemoryType::LOADER_DATA)};

    /* Copy the image into the correct memory region */

    /* Jump to the loader  */
    let kernel_start: unsafe extern "C" fn() = unsafe { core::mem::transmute(load_addr)};
    unsafe { (kernel_start)() };

    return Status::SUCCESS
}
