This is a comparison a few different rust async executors against each other to see how they handle many (relatively uniform) tasks.

The tasks are 2990 unique pathfinding tasks using [polyanya](https://github.com/vleue/polyanya).

System tested on: Windows 11, AMD 5900x:

| Executor                                                                     | runtime    |
|------------------------------------------------------------------------------|------------|
| [switchyard](https://github.com/bve-reborn/switchyard)                       | 3699.8 ms  |
| [smolscale](https://github.com/geph-official/smolscale)                      | 231.77 ms  |
| [bevy_tasks](https://github.com/bevyengine/bevy/tree/main/crates/bevy_tasks) | 406.98 ms  |
| [tokio](https://github.com/tokio-rs/tokio)                                   | 300.87 ms  |
| [rayon](https://github.com/rayon-rs/rayon)                                   | 205.48 ms  |
| singlethreaded                                                               | 3111.5 ms  |

## Instructions

Run `cargo --bench` to get criterion benchmarks.

`cargo run --release` is faster, but much less accurate and doesn't keep track of previous runs.
