use bootloader::bootinfo::{MemoryMap, MemoryRegionType};
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

/// 从bootloader的MemoryMap中返回可用的 frames。
pub struct BootInfoFrameAllocator {
    memory_map: &'static MemoryMap,
    next: usize,
}
impl BootInfoFrameAllocator {
    /// 从传递的内存 map 中创建一个FrameAllocator。
    /// 这个函数是不安全的，因为调用者必须保证传递的内存 map 是有效的。
    /// 主要的要求是，所有在其中被标记为 "可用 "的帧都是真正未使用的。
    pub unsafe fn init(memory_map: &'static MemoryMap) -> Self {
        BootInfoFrameAllocator {
            memory_map,
            // 每次分配帧时增加，以避免两次返回相同的帧
            next: 0,
        }
    }
    /// 返回内存映射中指定的可用页帧的迭代器
    fn usable_frames(&self) -> impl Iterator<Item = PhysFrame> {
        let regions = self.memory_map.iter();
        // 获取可用的内存区域
        let usable_regions = regions.filter(|r| r.region_type == MemoryRegionType::Usable);

        // 将每个区域映射到其地址范围
        let addr_ranges = usable_regions.map(|r| r.range.start_addr()..r.range.end_addr());
        // 转化为一个页帧起始地址的迭代器
        let frame_addresses = addr_ranges.flat_map(|r| r.step_by(4096));
        // 从起始地址创建 `PhysFrame`  类型
        frame_addresses.map(|addr| PhysFrame::containing_address(PhysAddr::new(addr)))
    }
}

unsafe impl FrameAllocator<Size4KiB> for BootInfoFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        let frame = self.usable_frames().nth(self.next);
        self.next += 1;
        frame
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
