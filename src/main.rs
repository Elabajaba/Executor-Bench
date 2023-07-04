use std::sync::Arc;

use exec_test::{get_mesh, Scenarios};

use rand::prelude::*;

// macro_rules! assert_delta {
//     ($x:expr, $y:expr) => {
//         let val = $x;
//         if !((val.length - $y).abs() < 0.001) {
//             assert_eq!(val.length, $y);
//         }
//     };
// }

fn main() {
    let mesh = Arc::new(get_mesh());

    let mut scenarios = Scenarios(Vec::new());

    // Change this to vary the amount of tasks by repeating the scenario multiple times.
    for _ in 0..1 {
        let mut temp = Scenarios::from_file("scenarios/aurora.scen");
        scenarios.0.append(&mut temp.0);
    }

    // Shuffle the pathfinding tasks for a more realistic workload of different task durations, instead of a uniform distribution of fast->slow tasks.
    let mut rng = StdRng::seed_from_u64(0);
    scenarios.0.shuffle(&mut rng);

    let count = scenarios.0.len();
    println!("count: {}", count);

    let future_creation = || {
        let mut array = Vec::with_capacity(count);
        let scenes = scenarios.clone();
        for scen in scenes.0.into_iter() {
            let mesh_clone = mesh.clone();
            array.push(async move { mesh_clone.get_path(scen.from, scen.to).await });
        }
        array
    };

    // Switchyard is slow with this many paths, so disable it for now
    {
        use switchyard::threads;

        let yard = switchyard::Switchyard::new(
            threads::one_to_one(threads::thread_info(), Some("switchyard")),
            || (),
        )
        .unwrap();

        let input = future_creation();

        let now = std::time::Instant::now();
        let handle_vec: Vec<_> = input.into_iter().map(|fut| yard.spawn(0, fut)).collect();

        futures_executor::block_on(async move {
            for handle in handle_vec {
                let _a = handle.await;
                // assert_delta!(a.unwrap(), 1123.2226);
            }
        });

        let after = std::time::Instant::now();
        println!("switchyard: {:?}", after - now);
    }
    // Smolscale
    {
        let input = future_creation();

        let now = std::time::Instant::now();
        let handle_vec: Vec<_> = input.into_iter().map(|fut| smolscale::spawn(fut)).collect();

        futures_executor::block_on(async move {
            for handle in handle_vec {
                let _a = handle.await;
                // assert_delta!(a.unwrap(), 1123.2226);
            }
        });

        let after = std::time::Instant::now();
        println!("smolscale: {:?}", after - now);
    }
    // Bevy_tasks
    {
        let temp = bevy_tasks::TaskPool::new();
        let input = future_creation();

        let now = std::time::Instant::now();
        let handle_vec: Vec<_> = input.into_iter().map(|fut| temp.spawn(fut)).collect();

        futures_executor::block_on(async move {
            for handle in handle_vec {
                let _a = handle.await;
                // assert_delta!(a.unwrap(), 1123.2226);
            }
        });

        let after = std::time::Instant::now();
        println!("bevy-tasks: {:?}", after - now);
    }
    {
        let tokio_runtime = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(std::thread::available_parallelism().unwrap().get())
            .build()
            .unwrap();
        let input = future_creation();

        let now = std::time::Instant::now();
        let handle_vec: Vec<_> = input
            .into_iter()
            .map(|fut| tokio_runtime.spawn(fut))
            .collect();

        futures_executor::block_on(async move {
            for handle in handle_vec {
                let _a = handle.await;
                // assert_delta!(a.unwrap(), 1123.2226);
            }
        });

        let after = std::time::Instant::now();
        println!("tokio: {:?}", after - now);
    }
    // Rayon
    {
        let now = std::time::Instant::now();

        rayon::scope(|s| {
            for scen in scenarios.0.iter() {
                s.spawn(|_| {
                    let _a = mesh.path(scen.from, scen.to);
                    // assert_delta!(a.unwrap(), 1123.2226);
                });
            }
        });

        let after = std::time::Instant::now();
        println!("rayon scope: {:?}", after - now);
    }
    // Single threaded
    {
        let now = std::time::Instant::now();

        for scen in scenarios.0.iter() {
            let _a = mesh.path(scen.from, scen.to);
            // assert_delta!(a.unwrap(), 1123.2226);
        }

        let after = std::time::Instant::now();
        println!("single threaded: {:?}", after - now);
    }
}
