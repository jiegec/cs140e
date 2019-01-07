use console::{kprint, kprintln, CONSOLE};
use std::path::{Path, PathBuf};
use pi::timer::{current_time, spin_sleep_ms};
use pi::gpio::Gpio;
use pi::atags::Atags;
#[cfg(not(test))]
use ALLOCATOR;
use FILE_SYSTEM;
use fat32::traits::{Dir, Entry, File, FileSystem, Metadata};
use std::io::Read;
use std::str;
use user::syscall;
use aarch64;

/// Error type for `Command` parse failures.
#[derive(Debug)]
enum Error {
    Empty,
}

const BEL: u8 = 0x07u8;
const BS: u8 = 0x08u8;
const LF: u8 = 0x0au8;
const CR: u8 = 0x0du8;
const ESC: u8 = 0x1bu8;
const DEL: u8 = 0x7fu8;

const BANNER: &str = r#"
                 _ _ ___         
             __-         `-_    
         ___/__        〇   \ 
     - '     _/             /
   '_'             /
 / _- ---            __ -
/`     |          _ / \  \
       |       -       \ |
        \    /          V
          \  |
            \ \
              \"#;

/// A structure representing a single shell command.
struct Command<'a> {
    args: Vec<&'a str>,
}

impl<'a> Command<'a> {
    /// Parse a command from a string `s` using `buf` as storage for the
    /// arguments.
    ///
    /// # Errors
    ///
    /// If `s` contains no arguments, returns `Error::Empty`. If there are more
    /// arguments than `buf` can hold, returns `Error::TooManyArgs`.
    fn parse(s: &str) -> Result<Command, Error> {
        let mut args = Vec::with_capacity(64);
        for arg in s.split(' ').filter(|a| !a.is_empty()) {
            args.push(arg);
        }

        if args.is_empty() {
            return Err(Error::Empty);
        }

        Ok(Command { args })
    }

    /// Returns this command's path. This is equivalent to the first argument.
    fn path(&self) -> &str {
        self.args[0]
    }
}

/// Starts a shell using `prefix` as the prefix for each line. This function
/// returns if the `exit` command is called.
pub fn shell(prefix: &str) {
    spin_sleep_ms(200); // wait a little time for client to attach
    kprintln!("{}", BANNER);

    let mut history = Vec::new();
    let mut cwd = PathBuf::from("/");
    loop {
        kprint!("{}{}", cwd.display(), prefix);

        let line = read_line(&mut history);
        match Command::parse(&line) {
            Ok(command) => {
                let path = command.path();
                match path {
                    "atags" => atags(&command),
                    "brk" => brk(),
                    "bt" => bt(),
                    "cat" => cat(&command, cwd.as_path()),
                    "cd" => cwd = cd(&command, cwd),
                    "current_el" => current_el(),
                    "echo" => echo(&command),
                    "echohex" => echohex(&command),
                    "exit" => return,
                    "gpio" => gpio(&command),
                    "ls" => ls(&command, cwd.as_path()),
                    "memstat" => memstat(&command),
                    "panic" => panic(&command),
                    "pwd" => pwd(&command, cwd.as_path()),
                    "sleep" => sleep(&command),
                    "uptime" => uptime(&command),
                    _ => kprintln!("unknown command: {}", path),
                }
            }
            Err(Error::Empty) => {
                // Ignore
            }
        }
    }
}

fn read_line(history: &mut Vec<Vec<u8>>) -> String {
    let mut console = CONSOLE.lock();
    let mut cursor = 0;
    let mut line_vec = Vec::with_capacity(512);
    let mut history_index = history.len();
    loop {
        match console.read_byte() {
            BS | DEL => {
                // Backspace
                if cursor > 0 {
                    cursor -= 1;
                    line_vec.remove(cursor);

                    console.write_byte(BS);
                    for byte in &line_vec[cursor..] {
                        console.write_byte(*byte);
                    }
                    console.write_byte(b' ');
                    for _i in cursor..line_vec.len() {
                        console.write_byte(BS);
                    }
                    console.write_byte(BS);
                } else {
                    console.write_byte(BEL);
                }
            }
            CR | LF => {
                // Return
                console.write_byte(CR);
                console.write_byte(LF);
                break;
            }
            ESC => {
                match console.read_byte() {
                    b'[' => {
                        match console.read_byte() {
                            b'D' => {
                                // Left arrow
                                if cursor > 0 {
                                    cursor -= 1;
                                    console.write_byte(ESC);
                                    console.write_byte(b'[');
                                    console.write_byte(b'D');
                                } else {
                                    console.write_byte(BEL);
                                }
                            }
                            b'C' => {
                                // Right arrow
                                if cursor < line_vec.len() {
                                    cursor += 1;
                                    console.write_byte(ESC);
                                    console.write_byte(b'[');
                                    console.write_byte(b'C');
                                } else {
                                    console.write_byte(BEL);
                                }
                            }
                            direction @ b'A' | direction @ b'B' => {
                                if direction == b'A' && history_index > 0 {
                                    // Up arrow
                                    history_index -= 1;
                                } else if direction == b'B' && history.len() > 0 // usize underflow
                                    && history_index < history.len() - 1
                                {
                                    // Down arrow
                                    history_index += 1;
                                } else {
                                    console.write_byte(BEL);
                                    continue;
                                }

                                for _ in 0..line_vec.len() {
                                    console.write_byte(BS);
                                }
                                for _ in 0..line_vec.len() {
                                    console.write_byte(b' ');
                                }
                                for _ in 0..line_vec.len() {
                                    console.write_byte(BS);
                                }
                                line_vec = history[history_index].clone();
                                cursor = line_vec.len();
                                for byte in &line_vec {
                                    console.write_byte(*byte);
                                }
                            }
                            _ => {
                                console.write_byte(BEL);
                            }
                        }
                    }
                    _ => {
                        console.write_byte(BEL);
                    }
                }
            }
            byte if byte.is_ascii_graphic() || byte == b' ' => {
                line_vec.insert(cursor, byte);
                for byte in &line_vec[cursor..] {
                    console.write_byte(*byte);
                }
                cursor += 1;
                for _i in cursor..line_vec.len() {
                    console.write_byte(BS);
                }
            }
            _ => {
                // unrecognized characters
                console.write_byte(BEL);
            }
        }
    }

    history.push(line_vec.clone());
    String::from_utf8(line_vec).unwrap_or_default()
}

fn echo(command: &Command) {
    if command.args.len() > 1 {
        kprint!("{}", command.args[1]);
        if command.args.len() > 2 {
            for arg in &command.args[2..] {
                kprint!(" {}", *arg);
            }
        }
    }

    kprintln!();
}

fn echohex(command: &Command) {
    echo(&command);
    if command.args.len() > 1 {
        for byte in command.args[1].bytes() {
            kprint!("{:02X}", byte);
        }
        if command.args.len() > 2 {
            for arg in &command.args[2..] {
                kprint!(" ");
                for byte in arg.bytes() {
                    kprint!("{:02X}", byte);
                }
            }
        }
    }

    kprintln!();
}

fn uptime(_command: &Command) {
    let time = current_time();
    let sec = time / 1000 / 1000;
    let min = sec / 60;
    let hour = min / 60;
    let day = hour / 24;
    kprintln!(
        "Uptime: {} day(s), {:02}:{:02}:{:02}",
        day,
        hour % 24,
        min % 60,
        sec % 60
    );
}

/*
fn exit(_command: &Command) -> ! {
    kprintln!(
        "Might be dangerous: bootloader might be corrupted, especially when using bin allocator."
    );
    kprintln!("Now exiting to bootloader. Detach and send a new kernel to continue.");
    // in line with bootloader
    const BOOTLOADER_START_ADDR: usize = 0x4000000;
    unsafe {
        asm!("br $0" : : "r"(BOOTLOADER_START_ADDR));
        loop {
            asm!("nop" :::: "volatile")
        }
    }
}
*/

fn panic(_command: &Command) -> ! {
    panic!("You ask me to panic!");
}

fn atags(_command: &Command) {
    for atag in Atags::get() {
        kprintln!("{:?}", atag);
    }
}

fn memstat(_command: &Command) {
    #[cfg(not(test))]
    kprintln!("Allocator: {:?}", ALLOCATOR);
}

fn ls(command: &Command, cwd: &Path) {
    let (all, path) = if command.args.len() > 3 {
        kprintln!("Wrong number of args! Usage: ls [-a] [directory]");
        return;
    } else if command.args.len() == 2 {
        if command.args[1] == "-a" {
            (true, PathBuf::from(cwd))
        } else {
            (false, cwd.join(command.args[1]))
        }
    } else if command.args.len() == 3 {
        if command.args[1] != "-a" {
            kprintln!("Wrong arg {}! Usage: ls [-a] [directory]", command.args[1]);
            return;
        } else {
            (true, cwd.join(command.args[2]))
        }
    } else {
        (false, PathBuf::from(cwd))
    };
    let dir = match FILE_SYSTEM.open_dir(path) {
        Ok(entry) => entry,
        Err(err) => {
            kprintln!("Error opening directory: {}", err);
            return;
        }
    };
    match dir.entries() {
        Ok(entries) => for entry in entries {
            let metadata = entry.metadata();
            if metadata.hidden() && all == false {
                continue;
            }
            kprint!("{}", if metadata.read_only() { 'r' } else { 'w' });
            kprint!("{}", if metadata.hidden() { 'h' } else { 'v' });
            kprint!("{}", if metadata.system() { 's' } else { '-' });
            kprint!("{}", if metadata.volume_id() { 'i' } else { '-' });
            kprint!("{}", if entry.is_dir() { 'd' } else { 'f' });
            kprint!("{}", if metadata.archive() { 'a' } else { '-' });
            kprint!("\t{:?}", metadata.created());
            kprint!("\t{:?}", metadata.modified());
            if entry.is_dir() {
                kprint!("\t0");
                kprintln!("\t{}/", entry.name());
            } else {
                kprint!("\t{}", entry.as_file().unwrap().size());
                kprintln!("\t{}", entry.name());
            }
        },
        Err(err) => kprintln!("Error listing dir: {}", err),
    }
}

fn cd(command: &Command, cwd: PathBuf) -> PathBuf {
    if command.args.len() != 2 {
        kprintln!("Wrong number of args for cd");
        return cwd;
    }
    FILE_SYSTEM
        .canonicalize(cwd.join(command.args[1]))
        .unwrap_or_else(|err| {
            kprintln!("Error: {}", err);
            cwd
        })
}

fn pwd(_command: &Command, cwd: &Path) {
    kprintln!("{}", cwd.display());
}

fn cat(command: &Command, cwd: &Path) {
    for files in &command.args[1..] {
        let file_path = cwd.join(files);
        let mut file = match FILE_SYSTEM.open_file(file_path) {
            Ok(entry) => entry,
            Err(err) => {
                kprintln!("Error: {}", err);
                continue;
            }
        };
        let mut contents = Vec::new();
        match file.read_to_end(&mut contents) {
            Ok(_) => kprintln!(
                "{}",
                str::from_utf8(&contents).unwrap_or("Error: file contain invalid UTF-8")
            ),
            Err(err) => kprintln!("Error reading file: {}", err),
        }
    }
}

fn gpio(command: &Command) {
    if command.args.len() != 3 {
        kprintln!("Usage: gpio [pin] [set|clear|level]");
        return;
    }

    let pin_num = match command.args[1].parse() {
        Ok(num) => num,
        Err(err) => {
            kprintln!("Unsupported pin number: {}", err);
            return;
        }
    };

    match command.args[2] {
        "set" => {
            let mut pin = Gpio::new(pin_num).into_output();
            pin.set();
            kprintln!("Pin {} set", pin_num);
        }
        "clear" => {
            let mut pin = Gpio::new(pin_num).into_output();
            pin.clear();
            kprintln!("Pin {} cleared", pin_num);
        }
        "level" => {
            let mut pin = Gpio::new(pin_num).into_input();
            kprintln!("Pin {} level: {}", pin_num, pin.level());
        }
        op => kprintln!("Unsupported operation {}", op),
    };
}

fn brk() {
    #[cfg(not(test))]
    unsafe {
        asm!("brk 2" :::: "volatile");
    }
}

fn current_el() {
    // Does not work if CurrentEL is 0
    // kprintln!("Current EL is {}", unsafe { aarch64::current_el() });
}

fn sleep(command: &Command) {
    if command.args.len() > 2 {
        kprintln!("Usage: sleep [time] ");
    }
    let time = if command.args.len() == 1 {
        1000
    } else {
        command.args[1].parse().unwrap_or(1000)
    };
    let (error, time_elapsed) = syscall::sleep(time);
    if error != 0 {
        kprintln!("Failed with {}", error);
    } else {
        kprintln!("Slept for {} msec", time_elapsed);
    }
}

extern "C" {
    fn __text_start();
    fn __text_end();
}

fn bt() {
    #[cfg(not(test))]
    unsafe {
        let mut current_pc = aarch64::pc();
        let mut current_fp = aarch64::fp();
        let mut stack_num = 0;
        kprintln!("#{} {:#018X}", stack_num, current_pc);
        current_pc = aarch64::lr();
        while current_pc >= __text_start as usize && current_pc <= __text_end as usize && current_fp as usize != 0 {
            stack_num = stack_num + 1;
            kprintln!("#{} {:#018X}", stack_num, current_pc);
            current_pc = *(current_fp as *const usize).offset(1);
            current_fp = *(current_fp as *const usize);
        }
    }
}

pub fn timer() {
    let (error, time_elapsed) = syscall::sleep(10 * 1000);
    if error != 0 {
        kprintln!("Failed with {}", error);
    } else {
        kprintln!("Slept for {} msec from thread 2", time_elapsed);
    }
}