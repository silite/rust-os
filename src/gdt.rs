// GDT是分页模式成为事实标准之前，用于内存分段的遗留结构，但它在64位模式下仍然需要处理一些事情，比如内核态/用户态的配置以及TSS载入
// GDT是包含了程序 段信息 的结构，在分页模式成为标准前，它在旧架构下起到隔离程序执行环境的作用
use lazy_static::lazy_static;
use x86_64::structures::gdt::{Descriptor, GlobalDescriptorTable, SegmentSelector};
use x86_64::structures::tss::TaskStateSegment;
use x86_64::VirtAddr;

pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;

lazy_static! {
    static ref TSS: TaskStateSegment = {
        // TSS用到了分段系统 在GDT中添加一个段描述符，通过ltr 指令加上GDT序号加载我们的TSS
        let mut tss = TaskStateSegment::new();
        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {
            const STACK_SIZE: usize = 4096 * 5;
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];
            let stack_start = VirtAddr::from_ptr(unsafe { &STACK });
            let stack_end = stack_start + STACK_SIZE;
            // x86 的栈内存分配是从高地址到低地址的
            stack_end
        };
        tss
    };
}

struct Selectors {
    code_selector: SegmentSelector,
    tss_selector: SegmentSelector,
}

lazy_static! {
    static ref GDT: (GlobalDescriptorTable, Selectors) = {
        let mut gdt = GlobalDescriptorTable::new();
        let code_selector = gdt.add_entry(Descriptor::kernel_code_segment());
        let tss_selector = gdt.add_entry(Descriptor::tss_segment(&TSS));
        (
            gdt,
            Selectors {
                code_selector,
                tss_selector,
            },
        )
    };
}

pub fn init() {
    // 修改了GDT，需要重载代码段寄存器 cs
    // 加载了包含TSS信息的GDT，还需要告诉CPU使用新的TSS
    // 当TSS加载完毕后，CPU就可以访问到新的IST了，通过修改IDT条目告诉CPU使用新的 double fault 专属栈
    GDT.0.load();

    use x86_64::instructions::segmentation::{Segment, CS};
    use x86_64::instructions::tables::load_tss;
    unsafe {
        CS::set_reg(GDT.1.code_selector);
        load_tss(GDT.1.tss_selector);
    }
}
