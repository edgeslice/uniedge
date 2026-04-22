use alloc::alloc::{Layout, alloc_zeroed, dealloc};
use alloc::vec;
use core::marker::PhantomData;
use core::ptr::NonNull;

use smoltcp::iface::{Config, Interface, SocketSet, SocketStorage};
use smoltcp::phy::{Device, DeviceCapabilities, Medium, RxToken, TxToken};
use smoltcp::socket::tcp;
use smoltcp::time::Duration;
use smoltcp::wire::{EthernetAddress, HardwareAddress, IpAddress, IpCidr, Ipv4Address};
use virtio_drivers::device::net::{RxBuffer, VirtIONet};
use virtio_drivers::transport::mmio::{MmioTransport, VirtIOHeader};
use virtio_drivers::{BufferDirection, Hal, PhysAddr};

use crate::console::Uart;
use crate::platform::VirtioMmioDevice;
use crate::time;

const GUEST_IP: Ipv4Address = Ipv4Address::new(10, 0, 2, 15);
const HTTP_PORT: u16 = 8080;
const QUEUE_SIZE: usize = 8;
const ETH_BUFFER_LEN: usize = 2048;
const HTTP_KEEP_ALIVE_SECS: u64 = 2;
const HTTP_SOCKET_TIMEOUT_SECS: u64 = 10;
const HTTP_REQUEST_TIMEOUT_MS: i64 = 5_000;
const RESPONSE: &str = concat!(
    "HTTP/1.1 200 OK\r\n",
    "Content-Type: text/plain; charset=utf-8\r\n",
    "Connection: close\r\n",
    "Content-Length: 18\r\n",
    "\r\n",
    "UniEdge is alive!\n",
);

pub fn run(device: VirtioMmioDevice) -> ! {
    let transport = virtio_transport(device);
    let ethernet = VirtIONet::<KernelHal, _, QUEUE_SIZE>::new(transport, ETH_BUFFER_LEN)
        .expect("virtio-net init failed");
    let mac = EthernetAddress(ethernet.mac_address());
    let mut netdev = NetworkDevice::new(ethernet);

    let mut iface = Interface::new(
        Config::new(HardwareAddress::Ethernet(mac)),
        &mut netdev,
        time::now(),
    );
    iface.update_ip_addrs(|ip_addrs| {
        ip_addrs
            .push(IpCidr::new(IpAddress::Ipv4(GUEST_IP), 24))
            .expect("ip address allocation failed");
    });

    let mut tcp_socket = tcp::Socket::new(
        tcp::SocketBuffer::new(vec![0; 1024]),
        tcp::SocketBuffer::new(vec![0; 1024]),
    );
    tcp_socket.set_keep_alive(Some(Duration::from_secs(HTTP_KEEP_ALIVE_SECS)));
    tcp_socket.set_timeout(Some(Duration::from_secs(HTTP_SOCKET_TIMEOUT_SECS)));

    let mut socket_storage = [SocketStorage::EMPTY];
    let mut sockets = SocketSet::new(&mut socket_storage[..]);
    let tcp_handle = sockets.add(tcp_socket);
    let mut request_seen = false;
    let mut announced_connection = false;
    let mut connection_start_ms: Option<i64> = None;

    loop {
        let timestamp = time::now();
        let _ = iface.poll(timestamp, &mut netdev, &mut sockets);

        let socket = sockets.get_mut::<tcp::Socket>(tcp_handle);
        if !socket.is_open() {
            request_seen = false;
            announced_connection = false;
            connection_start_ms = None;
            if let Err(err) = socket.listen(HTTP_PORT) {
                let mut uart = Uart::new();
                let _ = core::fmt::Write::write_fmt(
                    &mut uart,
                    format_args!("http listen error: {err}\n"),
                );
                socket.abort();
            }
        }

        if socket.is_active() && !announced_connection {
            announced_connection = true;
            connection_start_ms = Some(timestamp.total_millis());
            if let Some(endpoint) = socket.remote_endpoint() {
                let mut uart = Uart::new();
                let _ = core::fmt::Write::write_fmt(
                    &mut uart,
                    format_args!("http connection from {}:{}\n", endpoint.addr, endpoint.port),
                );
            }
        }

        if socket.is_active()
            && !request_seen
            && connection_start_ms.is_some_and(|start_ms| {
                timestamp.total_millis().saturating_sub(start_ms) >= HTTP_REQUEST_TIMEOUT_MS
            })
        {
            socket.abort();
            request_seen = false;
            announced_connection = false;
            connection_start_ms = None;

            let mut uart = Uart::new();
            let _ = core::fmt::Write::write_fmt(
                &mut uart,
                format_args!("http request timeout: connection dropped\n"),
            );
            continue;
        }

        if socket.may_recv() {
            match socket.recv(|buffer| (buffer.len(), buffer.len())) {
                Ok(received) if received > 0 => {
                    request_seen = true;
                }
                Ok(_) => {}
                Err(err) => {
                    socket.abort();
                    request_seen = false;
                    announced_connection = false;
                    connection_start_ms = None;

                    let mut uart = Uart::new();
                    let _ = core::fmt::Write::write_fmt(
                        &mut uart,
                        format_args!("http recv error: {err}\n"),
                    );
                    continue;
                }
            }
        }

        if request_seen && socket.can_send() {
            match socket.send_slice(RESPONSE.as_bytes()) {
                Ok(_) => {
                    socket.close();

                    let mut uart = Uart::new();
                    let _ = core::fmt::Write::write_fmt(
                        &mut uart,
                        format_args!("UniEdge is alive! | served http://127.0.0.1:{HTTP_PORT}\n"),
                    );
                }
                Err(err) => {
                    socket.abort();

                    let mut uart = Uart::new();
                    let _ = core::fmt::Write::write_fmt(
                        &mut uart,
                        format_args!("http send error: {err}\n"),
                    );
                }
            }
            request_seen = false;
            announced_connection = false;
            connection_start_ms = None;
        }

        core::hint::spin_loop();
    }
}

fn virtio_transport(device: VirtioMmioDevice) -> MmioTransport<'static> {
    let header = NonNull::new(device.base as *mut VirtIOHeader).expect("invalid virtio header");
    unsafe { MmioTransport::new(header, device.size) }.expect("virtio mmio transport init failed")
}

struct NetworkDevice {
    inner: VirtIONet<KernelHal, MmioTransport<'static>, QUEUE_SIZE>,
}

impl NetworkDevice {
    fn new(inner: VirtIONet<KernelHal, MmioTransport<'static>, QUEUE_SIZE>) -> Self {
        Self { inner }
    }
}

impl Device for NetworkDevice {
    type RxToken<'a>
        = NetworkRxToken<'a>
    where
        Self: 'a;
    type TxToken<'a>
        = NetworkTxToken<'a>
    where
        Self: 'a;

    fn receive(
        &mut self,
        _timestamp: smoltcp::time::Instant,
    ) -> Option<(Self::RxToken<'_>, Self::TxToken<'_>)> {
        let rx = self.inner.receive().ok()?;
        let device = NonNull::from(&mut self.inner);
        Some((
            NetworkRxToken {
                device,
                buffer: Some(rx),
                _lifetime: PhantomData,
            },
            NetworkTxToken {
                device,
                _lifetime: PhantomData,
            },
        ))
    }

    fn transmit(&mut self, _timestamp: smoltcp::time::Instant) -> Option<Self::TxToken<'_>> {
        let device = NonNull::from(&mut self.inner);
        Some(NetworkTxToken {
            device,
            _lifetime: PhantomData,
        })
    }

    fn capabilities(&self) -> DeviceCapabilities {
        let mut caps = DeviceCapabilities::default();
        caps.medium = Medium::Ethernet;
        caps.max_transmission_unit = 1500;
        caps.max_burst_size = Some(1);
        caps
    }
}

struct NetworkRxToken<'a> {
    device: NonNull<VirtIONet<KernelHal, MmioTransport<'static>, QUEUE_SIZE>>,
    buffer: Option<RxBuffer>,
    _lifetime: PhantomData<&'a mut NetworkDevice>,
}

impl RxToken for NetworkRxToken<'_> {
    fn consume<R, F>(mut self, f: F) -> R
    where
        F: FnOnce(&[u8]) -> R,
    {
        let buffer = self.buffer.take().expect("missing rx buffer");
        let result = f(buffer.packet());
        unsafe { self.device.as_mut() }
            .recycle_rx_buffer(buffer)
            .expect("failed to recycle rx buffer");
        result
    }
}

struct NetworkTxToken<'a> {
    device: NonNull<VirtIONet<KernelHal, MmioTransport<'static>, QUEUE_SIZE>>,
    _lifetime: PhantomData<&'a mut NetworkDevice>,
}

impl TxToken for NetworkTxToken<'_> {
    fn consume<R, F>(self, len: usize, f: F) -> R
    where
        F: FnOnce(&mut [u8]) -> R,
    {
        let device = unsafe { &mut *self.device.as_ptr() };
        let mut tx = device.new_tx_buffer(len);
        let result = f(tx.packet_mut());
        device.send(tx).expect("failed to transmit packet");
        result
    }
}

struct KernelHal;

unsafe impl Hal for KernelHal {
    fn dma_alloc(pages: usize, _direction: BufferDirection) -> (PhysAddr, NonNull<u8>) {
        let layout = Layout::from_size_align(pages * 4096, 4096).expect("invalid dma layout");
        let ptr = unsafe { alloc_zeroed(layout) };
        let vaddr = NonNull::new(ptr).expect("dma alloc failed");
        (vaddr.as_ptr() as u64, vaddr)
    }

    unsafe fn dma_dealloc(_paddr: PhysAddr, vaddr: NonNull<u8>, pages: usize) -> i32 {
        let layout = Layout::from_size_align(pages * 4096, 4096).expect("invalid dma layout");
        unsafe {
            dealloc(vaddr.as_ptr(), layout);
        }
        0
    }

    unsafe fn mmio_phys_to_virt(paddr: PhysAddr, _size: usize) -> NonNull<u8> {
        NonNull::new(paddr as *mut u8).expect("invalid mmio address")
    }

    unsafe fn share(buffer: NonNull<[u8]>, _direction: BufferDirection) -> PhysAddr {
        buffer.as_ptr() as *mut u8 as u64
    }

    unsafe fn unshare(_paddr: PhysAddr, _buffer: NonNull<[u8]>, _direction: BufferDirection) {}
}
