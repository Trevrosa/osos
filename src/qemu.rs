use x86_64::instructions::port::Port;

#[repr(u32)]
#[allow(unused)]
#[derive(Debug)]
pub enum ExitCode {
    Success = 0x10,
    Failed = 0x11,
}

/// exit qemu
pub fn exit(exit_code: ExitCode) {
    unsafe {
        let mut port = Port::new(0xf4);
        port.write(exit_code as u32);
    }
}
