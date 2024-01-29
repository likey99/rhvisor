pub mod addr;
mod frame;
pub mod heap;
pub fn init_heap() {
    heap::init();
}
pub fn init_frame_allocator() {
    frame::init();
}
