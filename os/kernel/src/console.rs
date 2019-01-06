use std::io;
use std::fmt;

use pi::uart;
use pi::uart::MiniUart;

use mutex::Mutex;

/// A global singleton allowing read/write access to the console.
pub struct Console {
    inner: Option<MiniUart>,
}

impl Console {
    /// Creates a new instance of `Console`.
    const fn new() -> Console {
        Console { inner: None }
    }

    /// Initializes the console if it's not already initialized.
    #[inline]
    fn initialize(&mut self) {
        self.inner.get_or_insert_with(&MiniUart::new);
    }

    /// Returns a mutable borrow to the inner `MiniUart`, initializing it as
    /// needed.
    fn inner(&mut self) -> &mut MiniUart {
        self.initialize();
        self.inner.as_mut().unwrap()
    }

    /// Reads a byte from the UART device, blocking until a byte is available.
    pub fn read_byte(&mut self) -> u8 {
        self.inner().read_byte()
    }

    /// Writes the byte `byte` to the UART device.
    pub fn write_byte(&mut self, byte: u8) {
        self.inner().write_byte(byte)
    }
}

impl io::Read for Console {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.inner().read(buf)
    }
}

impl io::Write for Console {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
       self.inner().write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl fmt::Write for Console {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.inner().write_str(s)
    }
}

/// Global `Console` singleton.
pub static CONSOLE: Mutex<Console> = Mutex::new(Console::new());

/// Internal function called by the `kprint[ln]!` macros.
#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    #[cfg(not(test))]
    {
        use std::fmt::Write;
        let mut console = CONSOLE.lock();
        console.write_fmt(args).unwrap();
    }

    #[cfg(test)]
    { print!("{}", args); }
}

/// Like `println!`, but for kernel-space.
pub macro kprintln {
    () => (kprint!("\n")),
    ($fmt:expr) => (kprint!(concat!($fmt, "\n"))),
    ($fmt:expr, $($arg:tt)*) => (kprint!(concat!($fmt, "\n"), $($arg)*))
}

/// Like `print!`, but for kernel-space.
pub macro kprint($($arg:tt)*) {
    _print(format_args!($($arg)*))
}

/// Internal function called by the `noblock_kprint[ln]!` macros.
#[doc(hidden)]
pub fn _noblock_print(args: fmt::Arguments) {
    use std::fmt::Write;
    let mut console = Console::new();
    console.write_fmt(args).unwrap();
}

/// Like `println!`, but non-blocking and for kernel-space.
pub macro noblock_kprintln {
    () => (noblock_kprint!("\n")),
    ($fmt:expr) => (noblock_kprint!(concat!($fmt, "\n"))),
    ($fmt:expr, $($arg:tt)*) => (noblock_kprint!(concat!($fmt, "\n"), $($arg)*))
}

/// Like `print!`, but non-blocking and for kernel-space.
pub macro noblock_kprint($($arg:tt)*) {
    _noblock_print(format_args!($($arg)*))
}
