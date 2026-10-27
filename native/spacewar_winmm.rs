#![no_std]
#![allow(non_snake_case)]
use core::ffi::c_void;
use core::ptr::{addr_of, addr_of_mut, null_mut};

include!("generated.rs");
core::arch::global_asm!(include_str!("exports.s"));

#[link(name = "kernel32", kind = "raw-dylib")]
unsafe extern "system" {
    fn DisableThreadLibraryCalls(module: *mut c_void) -> i32;
    fn GetModuleFileNameW(module: *mut c_void, buffer: *mut u16, size: u32) -> u32;
    fn SetEnvironmentVariableW(name: *const u16, value: *const u16) -> i32;
    fn GetSystemDirectoryW(buffer: *mut u16, size: u32) -> u32;
    fn LoadLibraryExW(path: *const u16, file: *mut c_void, flags: u32) -> *mut c_void;
    fn GetProcAddress(module: *mut c_void, name: *const u8) -> *mut c_void;
    fn InitOnceExecuteOnce(once: *mut usize, callback: unsafe extern "system" fn(*mut usize, *mut c_void, *mut *mut c_void) -> i32, parameter: *mut c_void, context: *mut *mut c_void) -> i32;
    fn CreateFileW(path: *const u16, access: u32, share: u32, security: *mut c_void, disposition: u32, attributes: u32, template: *mut c_void) -> *mut c_void;
    fn ReadFile(file: *mut c_void, buffer: *mut u8, count: u32, read: *mut u32, overlapped: *mut c_void) -> i32;
    fn WriteFile(file: *mut c_void, buffer: *const u8, count: u32, written: *mut u32, overlapped: *mut c_void) -> i32;
    fn CloseHandle(file: *mut c_void) -> i32;
    fn GetCurrentProcess() -> *mut c_void;
    fn TerminateProcess(process: *mut c_void, status: u32) -> i32;
}

static mut SELF: *mut c_void = null_mut();
static mut ONCE: usize = 0;
static mut REAL: *mut c_void = null_mut();
static mut ROOT: [u16; 1024] = [0; 1024];
static mut ROOT_LEN: usize = 0;
static mut IS_GAME: bool = false;
static mut SETTINGS_APPLIED: bool = false;
static mut APP_ID: [u16; 16] = [0; 16];
static mut APP_ID_ASCII: [u8; 16] = [0; 16];
static mut APP_ID_LEN: usize = 0;
static mut ADDRESSES: [*mut c_void; EXPORT_COUNT] = [null_mut(); EXPORT_COUNT];

unsafe fn abort() -> ! {
    TerminateProcess(GetCurrentProcess(), 0x53570400);
    loop { core::hint::spin_loop(); }
}
#[panic_handler]
fn panic(_: &core::panic::PanicInfo<'_>) -> ! { unsafe { abort() } }

// Prebuilt core objects retain Windows exception metadata. This crate never
// unwinds; an unexpected exception in its Rust code terminates the process.
#[unsafe(no_mangle)]
pub static _fltused: i32 = 0;
#[unsafe(no_mangle)]
pub unsafe extern "C" fn __CxxFrameHandler3(_: *mut c_void, _: *mut c_void, _: *mut c_void, _: *mut c_void) -> i32 { abort() }

fn trimmed(mut input: &[u8]) -> &[u8] {
    while input.first().is_some_and(|c| c.is_ascii_whitespace()) { input = &input[1..]; }
    while input.last().is_some_and(|c| c.is_ascii_whitespace()) { input = &input[..input.len()-1]; }
    input
}
fn read_id(bytes: &[u8]) -> u32 {
    let mut steam = false;
    for raw in bytes.split(|c| *c == b'\n') {
        let line = trimmed(raw);
        if line.is_empty() || line[0] == b';' || line[0] == b'#' { continue; }
        if line[0] == b'[' { steam = line.eq_ignore_ascii_case(b"[Steam]"); continue; }
        if !steam { continue; }
        if let Some(equal) = line.iter().position(|c| *c == b'=') {
            if !trimmed(&line[..equal]).eq_ignore_ascii_case(b"AppId") { continue; }
            let value = trimmed(&line[equal+1..]);
            let mut parsed = 0u32;
            if value.is_empty() { return 480; }
            for c in value {
                if !c.is_ascii_digit() { return 480; }
                let Some(next) = parsed.checked_mul(10).and_then(|v| v.checked_add((c-b'0') as u32)) else { return 480; };
                parsed = next;
            }
            return if parsed == 0 { 480 } else { parsed };
        }
    }
    480
}
unsafe fn path_with_suffix(buffer: *mut u16, suffix: &[u16]) -> bool {
    if ROOT_LEN + suffix.len() >= 1024 { return false; }
    core::ptr::copy_nonoverlapping(addr_of!(ROOT).cast::<u16>(), buffer, ROOT_LEN);
    core::ptr::copy_nonoverlapping(suffix.as_ptr(), buffer.add(ROOT_LEN), suffix.len());
    true
}
unsafe fn configure(module: *mut c_void) {
    let length = GetModuleFileNameW(module, addr_of_mut!(ROOT).cast::<u16>(), 1024) as usize;
    if length == 0 || length >= 1024 { return; }
    let mut last = length;
    while last > 0 && ROOT[last-1] != b'\\' as u16 && ROOT[last-1] != b'/' as u16 { last -= 1; }
    ROOT_LEN = last;
    let mut process = [0u16; 260];
    let n = GetModuleFileNameW(null_mut(), process.as_mut_ptr(), 260) as usize;
    if n == 0 || n >= 260 { return; }
    let mut start = n;
    while start > 0 && process[start-1] != b'\\' as u16 && process[start-1] != b'/' as u16 { start -= 1; }
    let wanted = b"riverdrift-win64-shipping.exe";
    if n - start != wanted.len() { return; }
    for i in 0..wanted.len() {
        let c = process[start+i];
        let lower = if c >= 65 && c <= 90 { c + 32 } else { c };
        if lower != wanted[i] as u16 { return; }
    }
    IS_GAME = true;
    let mut path = [0u16; 1024];
    let mut id = 480;
    if path_with_suffix(path.as_mut_ptr(), &INI_FILE) {
        let file = CreateFileW(path.as_ptr(), 0x80000000, 3, null_mut(), 3, 0x80, null_mut());
        if file as isize != -1 {
            let mut bytes = [0u8; 512]; let mut count = 0u32;
            if ReadFile(file, bytes.as_mut_ptr(), 512, &mut count, null_mut()) != 0 { id = read_id(&bytes[..count as usize]); }
            CloseHandle(file);
        }
    }
    let mut digits = [0u8; 10]; let mut count = 0;
    while id != 0 { digits[count] = b'0' + (id % 10) as u8; id /= 10; count += 1; }
    APP_ID_LEN = count;
    for i in 0..count { APP_ID[i] = digits[count-1-i] as u16; APP_ID_ASCII[i] = digits[count-1-i]; }
    APP_ID[count] = 0;
    let value = addr_of!(APP_ID).cast::<u16>();
    SETTINGS_APPLIED = SetEnvironmentVariableW(STEAM_APP_ID.as_ptr(), value) != 0;
    SETTINGS_APPLIED &= SetEnvironmentVariableW(STEAM_GAME_ID.as_ptr(), value) != 0;
    SETTINGS_APPLIED &= SetEnvironmentVariableW(STEAM_OVERLAY_ID.as_ptr(), value) != 0;
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn DllMain(module: *mut c_void, reason: u32, _: *mut c_void) -> i32 {
    if reason == 1 {
        SELF = module;
        DisableThreadLibraryCalls(module);
        configure(module);
    }
    1
}

unsafe fn log_ready() {
    if !IS_GAME { return; }
    let mut path = [0u16; 1024];
    if !path_with_suffix(path.as_mut_ptr(), &LOG_FILE) { return; }
    let file = CreateFileW(path.as_ptr(), 4, 3, null_mut(), 4, 0x80, null_mut());
    if file as isize == -1 { return; }
    let mut written = 0u32;
    let header = if SETTINGS_APPLIED { b"River Drift Spacewar native loader v0.1.0: process-local settings applied; AppId=".as_slice() }
                 else { b"River Drift Spacewar native loader v0.1.0: settings could not be fully applied; AppId=".as_slice() };
    WriteFile(file, header.as_ptr(), header.len() as u32, &mut written, null_mut());
    WriteFile(file, addr_of!(APP_ID_ASCII).cast::<u8>(), APP_ID_LEN as u32, &mut written, null_mut());
    let tail = b"\r\nSystem winmm forwarding initialized. Live Steam identity, overlay and multiplayer require verification.\r\n";
    WriteFile(file, tail.as_ptr(), tail.len() as u32, &mut written, null_mut());
    CloseHandle(file);
}
unsafe extern "system" fn initialize(_: *mut usize, _: *mut c_void, _: *mut *mut c_void) -> i32 {
    // Loading the real system DLL is intentionally deferred until outside DllMain.
    let mut path = [0u16; 260];
    let n = GetSystemDirectoryW(path.as_mut_ptr(), 260) as usize;
    if n == 0 || n + SYSTEM_SUFFIX.len() >= 260 { return 0; }
    core::ptr::copy_nonoverlapping(SYSTEM_SUFFIX.as_ptr(), path.as_mut_ptr().add(n), SYSTEM_SUFFIX.len());
    let module = LoadLibraryExW(path.as_ptr(), null_mut(), 0x800);
    if module.is_null() || module == SELF { return 0; }
    REAL = module;
    for i in 0..EXPORT_COUNT {
        let name = EXPORT_NAMES[i];
        let key = if name.len() <= 1 { EXPORT_ORDINALS[i] as usize as *const u8 } else { name.as_ptr() };
        ADDRESSES[i] = GetProcAddress(module, key);
    }
    log_ready();
    1
}
#[unsafe(no_mangle)]
pub unsafe extern "system" fn sw_resolve_export(index: usize) -> *mut c_void {
    #[cfg(sw_test)]
    {
        if index == EXPORT_COUNT { return test_integer as *mut c_void; }
        if index == EXPORT_COUNT + 1 { return test_float as *mut c_void; }
    }
    if index >= EXPORT_COUNT || InitOnceExecuteOnce(addr_of_mut!(ONCE), initialize, null_mut(), null_mut()) == 0 { abort(); }
    let address = ADDRESSES[index];
    if address.is_null() { abort(); }
    address
}
#[cfg(sw_test)]
unsafe extern "system" fn test_integer(a:u64,b:u64,c:u64,d:u64,e:u64,f:u64,g:u64,h:u64) -> u64 { a + 2*b + 3*c + 4*d + 5*e + 6*f + 7*g + 8*h }
#[cfg(sw_test)]
unsafe extern "system" fn test_float(a:f64,b:f64,c:f64,d:f64,e:f64,f:f64) -> f64 { a + 2.0*b + 3.0*c + 4.0*d + 5.0*e + 6.0*f }

// No C runtime DLL is needed. Volatile loops prevent recursive intrinsic lowering.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn memset(dst:*mut u8, value:i32, count:usize) -> *mut u8 {
    for i in 0..count { core::ptr::write_volatile(dst.add(i), value as u8); } dst
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn memcpy(dst:*mut u8, src:*const u8, count:usize) -> *mut u8 {
    for i in 0..count { core::ptr::write_volatile(dst.add(i), core::ptr::read_volatile(src.add(i))); } dst
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn memmove(dst:*mut u8, src:*const u8, count:usize) -> *mut u8 {
    if dst as usize <= src as usize { memcpy(dst, src, count); }
    else { for i in (0..count).rev() { core::ptr::write_volatile(dst.add(i), core::ptr::read_volatile(src.add(i))); } } dst
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn memcmp(a:*const u8, b:*const u8, count:usize) -> i32 {
    for i in 0..count { let x=core::ptr::read_volatile(a.add(i)); let y=core::ptr::read_volatile(b.add(i)); if x != y { return x as i32-y as i32; } } 0
}
