use linked_list_allocator::LockedHeap;

const HEAP_SIZE: usize = 1024 * 1024;

#[repr(align(4096))]
struct HeapSpace([u8; HEAP_SIZE]);

static mut HEAP_SPACE: HeapSpace = HeapSpace([0; HEAP_SIZE]);

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

pub fn init() {
    unsafe {
        let heap_start = core::ptr::addr_of_mut!(HEAP_SPACE.0) as *mut u8;
        ALLOCATOR.lock().init(heap_start, HEAP_SIZE);
    }
}
