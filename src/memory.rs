use bootloader::bootinfo::{MemoryMap, MemoryRegionType};
use x86_64::{
    structures::paging::{
        FrameAllocator, OffsetPageTable, PageTable, PhysFrame, Size4KiB, UnusedPhysFrame,
    },
    PhysAddr, VirtAddr,
};

pub struct BootInfoFrameAllocator<I: Iterator<Item = UnusedPhysFrame>> {
    usable_frames: I,
}

/// Create a FrameAllocator from the passed memory map.
/// # Safety
/// - All frames marked as USABLE in `memory_map` are really unused
pub unsafe fn init_allocator(memory_map: &'static MemoryMap) -> BootInfoFrameAllocator<impl Iterator<Item = UnusedPhysFrame>> {
	BootInfoFrameAllocator {
		usable_frames: usable_frames(memory_map)
	}
}
fn usable_frames(memory_map: &'static MemoryMap) -> impl Iterator<Item = UnusedPhysFrame> {
	//get usable regions from memory map
	let regions = memory_map.iter();
	let usable_regions = regions.filter(|r| r.region_type == MemoryRegionType::Usable);
	//map each region to its address range
	let addr_ranges = usable_regions.map(|r| r.range.start_addr()..r.range.end_addr());
	//transform to an iterator of frame start addresses
	let frame_addresses = addr_ranges.flat_map(|r| r.step_by(4096));
	//create PhysFrame types from the start addresses
	let frames = frame_addresses.map(|addr| PhysFrame::containing_address(PhysAddr::new(addr)));
	frames.map(|f| unsafe { UnusedPhysFrame::new(f) })
}

unsafe impl<I: Iterator<Item = UnusedPhysFrame>> FrameAllocator<Size4KiB> for BootInfoFrameAllocator<I> {
    fn allocate_frame(&mut self) -> Option<UnusedPhysFrame> {
        self.usable_frames.next()
    }
}

/// Gets a mut reference to the active level 4 table
/// # Safety
/// - The complete physical memory is mapped to virtual memory at the passed `physical_memory_offset`
unsafe fn active_level_4_table(physical_memory_offset: VirtAddr) -> &'static mut PageTable {
    use x86_64::registers::control::Cr3;

    let (level_4_table_frame, _) = Cr3::read();

    //get the frame address of the CR3 register
    let phys = level_4_table_frame.start_address();
    //convert that to a virtual address
    let virt = physical_memory_offset + phys.as_u64();
    //get the page table through a virtual memory read
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();

    &mut *page_table_ptr
}

/// Initialize the memory
/// # Safety
/// - The complete physical memory must be mapped to virtual memory at the passed in `physical_memory_offset`
/// - This method must only be called once, to avoid aliasing &mut references
pub unsafe fn init(physical_memory_offset: VirtAddr) -> OffsetPageTable<'static> {
    let level_4_table = active_level_4_table(physical_memory_offset);
    OffsetPageTable::new(level_4_table, physical_memory_offset)
}
