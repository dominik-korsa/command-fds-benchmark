use std::fs::File;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};
use command_fds::{CommandFdExt, FdMapping};

const REPEAT_COUNT: usize = 10_000;

fn main() {
    // 0 MB
    test(false, 0);
    test(true, 0);

    // 32 MB
    test(false, 32);
    test(true, 32);

    // 512 MB
    test(false, 512);
    test(true, 512);

    // 16 GB
    // test(false, 16 * 1024);
    // test(true, 16 * 1024);
}

fn test(register_fd_3: bool, allocate_mbs: usize) {
    let large_vec: Option<Vec<u8>> = (allocate_mbs > 0).then(|| {
        println!("Allocating memory...");
        let size = allocate_mbs * 1024 * 1024;

        // Allocating a big vector with zeroes does not affect spawn time.
        // Maybe the memory is only marked as reserved but not actually written to?
        // vec![0; size];

        (0..size).map(|i| (i % 256) as u8).collect()
    });

    println!("Running tests...");

    let mut command = Command::new("true");
    command
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .stdout(Stdio::null());

    if register_fd_3 {
        let null_file = File::open("/dev/null").unwrap();
        command.fd_mappings(vec![
            FdMapping {
                parent_fd: null_file.into(),
                child_fd: 3,
            },
        ]).unwrap();
    }

    let mut durations: Vec<Duration> = (0..REPEAT_COUNT).map(|_| {
        let start_time = Instant::now();
        let mut child = command.spawn().unwrap();
        let end_time = Instant::now();

        // wait is required to release OS resources
        child.kill().unwrap();
        child.wait().unwrap();

        end_time.checked_duration_since(start_time).expect("Duration should be positive")
    }).collect();
    durations.sort();

    println!(
        "spawn() wall-clock time {} fd_mappings and with {} MB extra memory:\n10. percentile: {:?}, 50. percentile {:?}, 90. percentile: {:?}\n",
        if register_fd_3 { "with" } else { "without" },
        allocate_mbs,
        durations[durations.len() * 10 / 100],
        durations[durations.len() * 50 / 100],
        durations[durations.len() * 90 / 100],
    );


    drop(large_vec);
}
