use core::ptr::read_volatile;
use fdt::Fdt;

pub const DTB_ADDR: usize = 0x4000_0000;
const VIRTIO_MMIO_START: usize = 0x0a00_0000;
const VIRTIO_MMIO_END: usize = 0x0a00_4000;
const VIRTIO_MMIO_STRIDE: usize = 0x200;
const VIRTIO_MAGIC: u32 = 0x7472_6976;
const VIRTIO_DEVICE_ID_NET: u32 = 1;

#[derive(Copy, Clone)]
pub struct VirtioMmioDevice {
    pub base: usize,
    pub size: usize,
}

pub fn first_virtio_mmio() -> Option<VirtioMmioDevice> {
    let dtb = unsafe { core::slice::from_raw_parts(DTB_ADDR as *const u8, 64 * 1024) };
    if let Ok(fdt) = Fdt::new(dtb) {
        if let Some(node) = fdt.find_compatible(&["virtio,mmio"]) {
            if let Some(region) = node.reg().and_then(|mut reg| reg.next()) {
                let device = VirtioMmioDevice {
                    base: region.starting_address as usize,
                    size: region.size.unwrap_or(VIRTIO_MMIO_STRIDE),
                };
                if is_virtio_net(device.base) {
                    return Some(device);
                }
            }
        }
    }

    scan_virtio_mmio()
}

fn scan_virtio_mmio() -> Option<VirtioMmioDevice> {
    let mut base = VIRTIO_MMIO_START;
    while base < VIRTIO_MMIO_END {
        if is_virtio_net(base) {
            return Some(VirtioMmioDevice {
                base,
                size: VIRTIO_MMIO_STRIDE,
            });
        }

        base += VIRTIO_MMIO_STRIDE;
    }

    None
}

fn mmio_read(addr: usize) -> u32 {
    unsafe { read_volatile(addr as *const u32) }
}

fn is_virtio_net(base: usize) -> bool {
    let magic = mmio_read(base);
    let version = mmio_read(base + 0x004);
    let device_id = mmio_read(base + 0x008);

    magic == VIRTIO_MAGIC && version >= 1 && device_id == VIRTIO_DEVICE_ID_NET
}
