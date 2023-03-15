use x86_64::{
    registers::control::Cr3,
    structures::paging::{
        page_table::FrameError, FrameAllocator, Mapper, OffsetPageTable, Page, PageTable,
        PhysFrame, Size4KiB,
    },
    PhysAddr, VirtAddr,
};
/// 返回一个对活动的4级表的可变引用。
unsafe fn active_level_4_table(physical_memory_offset: VirtAddr) -> &'static mut PageTable {
    let (level_4_table_frame, _) = Cr3::read();
    let phys = level_4_table_frame.start_address();
    let virt = physical_memory_offset + phys.as_u64();
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();

    &mut *page_table_ptr
}

pub unsafe fn init(physical_memory_offset: VirtAddr) -> OffsetPageTable<'static> {
    let level_4_table = active_level_4_table(physical_memory_offset);
    OffsetPageTable::new(level_4_table, physical_memory_offset)
}

// bootloader在虚拟地址空间的第一兆字节内加载自己，这意味着这个区域的所有页面都存在一个有效的1级表
// 就无需创造新页表，免去创建映射时页表还未创建的麻烦
pub struct EmptyFrameAllocator;

unsafe impl FrameAllocator<Size4KiB> for EmptyFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        None
    }
}

/// 为给定的页面创建一个实例映射到框架`0xb8000`
pub fn create_example_mapping(
    page: Page,
    mapper: &mut OffsetPageTable,
    // 映射给定的页面可能需要创建额外的页表，而页表需要未使用的框架作为后备存储
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) {
    use x86_64::structures::paging::PageTableFlags as Flags;

    let frame = PhysFrame::containing_address(PhysAddr::new(0xb8000));
    let flags = Flags::PRESENT | Flags::WRITABLE;

    let map_to_result = unsafe {
        // FIXME: 这并不安全，我们这样做只是为了测试。
        mapper.map_to(page, frame, flags, frame_allocator)
    };
    // MapperFlush.flush() 从翻译查找（TLB）重刷新映射的页面
    map_to_result.expect("map_to failed").flush();
}
