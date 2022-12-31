#[no_mangle]
pub extern "sysv64" fn my_write(address: *const u8, len: usize) {
    let string = unsafe { std::slice::from_raw_parts(address, len) };
    let string = std::str::from_utf8(string).unwrap();
    print!("{}", string);
}
