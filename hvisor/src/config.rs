use spin::Mutex;
pub const DTB_ADDR: usize = 0xbfe00000;
// // 定义一个4K对齐的数组类型
// #[repr(align(4096))]
// struct Aligned_dtb1([u8; include_bytes!("../../guests/linux.dtb").len()]);

// #[link_section = ".dtb"]
// /// the guest dtb file
// pub static GUEST2_DTB: Aligned_dtb1 = Aligned_dtb1(*include_bytes!("../../guests/linux.dtb"));
#[link_section = ".dtb1"]
/// the guest dtb file
pub static GUEST1_DTB: [u8; include_bytes!("../../guests/devicetree/linux3.dtb").len()] =
    *include_bytes!("../../guests/devicetree/linux3.dtb");
#[link_section = ".img1"]
/// the guest kernel file
// pub static GUEST1: [u8; include_bytes!("../../guests/img/Image-62U").len()] =
//     *include_bytes!("../../guests/img/Image-62U");
pub static GUEST1: [u8; include_bytes!("../../guests/img/bao-Image").len()] =
    *include_bytes!("../../guests/img/bao-Image");
#[link_section = ".dtb2"]
/// the guest dtb file
pub static GUEST2_DTB: [u8; include_bytes!("../../guests/devicetree/linux.dtb").len()] =
    *include_bytes!("../../guests/devicetree/linux.dtb");
#[link_section = ".img2"]
/// the guest kernel file
pub static GUEST2: [u8; include_bytes!("../../guests/img/Image-62U").len()] =
    *include_bytes!("../../guests/img/Image-62U");

// #[link_section = ".dtb"]
// /// the guest dtb file
// pub static GUEST1_DTB: [u8; include_bytes!("../../guests/linux4.dtb").len()] =
//     *include_bytes!("../../guests/linux4.dtb");
// #[link_section = ".initrd"]
// /// the guest kernel file
// pub static GUEST1: [u8; include_bytes!("../../guests/bao-linux.bin").len()] =
//     *include_bytes!("../../guests/bao-linux.bin");
// pub static GUESTS: [(&'static [u8], &'static [u8]); 1] = [(&GUEST1, &GUEST1_DTB)];
// #[link_section = ".dtb"]
// /// the guest dtb file
// pub static GUEST_DTB: [u8; include_bytes!("../../guests/rCore-Tutorial-v3/rCore-Tutorial-v3.dtb")
//     .len()] = *include_bytes!("../../guests/rCore-Tutorial-v3/rCore-Tutorial-v3.dtb");
// #[link_section = ".initrd"]
// static GUEST: [u8; include_bytes!("../../guests/rCore-Tutorial-v3/rCore-Tutorial-v3.bin").len()] =
//     *include_bytes!("../../guests/rCore-Tutorial-v3/rCore-Tutorial-v3.bin");
// #[link_section = ".dtb"]
// /// the guest dtb file
// pub static GUEST2_DTB: [u8; include_bytes!("../../guests/os_ch5.dtb").len()] =
//     *include_bytes!("../../guests/os_ch5.dtb");
// #[link_section = ".rcore"]
// /// the guest kernel file
// pub static GUEST2: [u8; include_bytes!("../../guests/os_ch5_802.bin").len()] =
//     *include_bytes!("../../guests/os_ch5_802.bin");
// pub static GUESTS: [(&'static [u8], &'static [u8]); 1] = [(&GUEST2, &GUEST2_DTB)];
pub static GUESTS: [(&'static [u8], &'static [u8]); 2] =
    [(&GUEST1, &GUEST1_DTB), (&GUEST2, &GUEST2_DTB)];
