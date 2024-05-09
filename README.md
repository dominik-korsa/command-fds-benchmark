# Benchmark for `command-fds`
It appears that `Command::spawn` is sometimes orders of magnitute slower when using `command-fds`.
This repo contains a benchmark to test it.

## Benchmark results on my machine
**Meaning of the values**: 10. percentile, **50. percentile**, 90. percentile

| Extra memory | `spawn()` time without command-fds  | `spawn()` time with command-fds         | Note                 |
|--------------|-------------------------------------|-----------------------------------------|----------------------|
| 0 MB         | 94.281µs, **96.76µs**, 125.561µs    | 229.882µs, **239.121µs**, 1.715612ms    | 2,47x times slower   |
| 32 MB        | 94.321µs, **97.281µs**, 129.841µs   | 270.682µs, **282.682µs**, 2.112135ms    | 2,91x times slower   |
| 512 MB       | 94.521µs, **97.08µs**, 125.441µs    | 1.010207ms, **1.257049ms**, 5.857801ms  | 12,95x times slower  |

We can observe that the `spawn()` execution time without `command-fds` does not depend on the amount of RAM used,
but there is a huge difference in execution time when `command-fds` is used.

## Possible cause of the slowdown
- `command-fds` calls `Command::pre_exec` to set the handler that duplicates FDs:
  - https://github.com/google/command-fds/blob/7714f31e5055058b67e94dc5fc23fc9568768a65/src/lib.rs#L103-L129
- This causes the default `Command::spawn` implementation on Linux to use the `fork()` syscall instead of `posix_spawn()`:
  - https://github.com/rust-lang/rust/blob/8c7c151a7a03d92cc5c75c49aa82a658ec1fe4ff/library/std/src/sys/pal/unix/process/process_unix.rs#L455-L463
  - https://github.com/rust-lang/rust/blob/8c7c151a7a03d92cc5c75c49aa82a658ec1fe4ff/library/std/src/sys/pal/unix/process/process_unix.rs#L78-L80
  - https://github.com/rust-lang/rust/blob/8c7c151a7a03d92cc5c75c49aa82a658ec1fe4ff/library/std/src/sys/pal/unix/process/process_unix.rs#L201-L203

The `fork()` syscall is likely much slower than `posix_spawn()`, escpecially when memory usage is high. `fork()` sets all memory pages to copy-on-write,
while `posix_spawn()` uses `clone()` under the hood to share process memory: 
- https://man7.org/linux/man-pages/man2/fork.2.html#NOTES
- https://man7.org/linux/man-pages/man3/posix_spawn.3.html
