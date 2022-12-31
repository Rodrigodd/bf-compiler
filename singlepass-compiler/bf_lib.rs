use std::io::{Read, Write};

#[no_mangle]
pub extern "sysv64" fn bf_write(value: u8) {
    // Writing a non-UTF-8 byte sequence on Windows error out.
    if cfg!(target_os = "windows") && value >= 128 {
        return;
    }

    let mut stdout = std::io::stdout().lock();

    let result = stdout.write_all(&[value]).and_then(|_| stdout.flush());

    if let Err(err) = result {
        eprintln!("IO error: {}", err);
        std::process::exit(1);
    }
}

#[no_mangle]
pub unsafe extern "sysv64" fn bf_read(buf: *mut u8) {
    let mut stdin = std::io::stdin().lock();
    loop {
        let mut value = 0;
        let err = stdin.read_exact(std::slice::from_mut(&mut value));

        if let Err(err) = err {
            if err.kind() != std::io::ErrorKind::UnexpectedEof {
                eprintln!("IO error: {}", err);
                std::process::exit(1);
            }
            value = 0;
        }

        // ignore CR from Window's CRLF
        if cfg!(target_os = "windows") && value == b'\r' {
            continue;
        }

        *buf = value;
        break;
    }
}

#[no_mangle]
pub unsafe extern "sysv64" fn bf_exit() {
    std::process::exit(0);
}
