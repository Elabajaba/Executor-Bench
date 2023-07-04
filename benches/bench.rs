use std::sync::Arc;

use criterion::{black_box, criterion_group, criterion_main, BatchSize, Criterion, Throughput};
use exec_test::Scenarios;
use rand::prelude::*;
use switchyard::threads;

pub fn polyanya(c: &mut Criterion) {
    let mut group = c.benchmark_group("polyanya");

    group.sample_size(10);

    let mesh: Arc<polyanya::Mesh> =
        Arc::new(polyanya::PolyanyaFile::from_file("meshes/aurora-merged.mesh").into());

    let mut scenarios = Scenarios::from_file("scenarios/aurora.scen");

    // Shuffle the pathfinding tasks so that they don't uniformly go from fast to slow.
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

    group.throughput(Throughput::Elements(count as u64));
    // Switchyard is much slower
    {
        let yard = switchyard::Switchyard::new(
            threads::one_to_one(threads::thread_info(), Some("switchyard")),
            || (),
        )
        .unwrap();

        group.bench_function("switchyard", |b| {
            b.iter_batched(
                future_creation,
                |input| {
                    let handle_vec: Vec<_> =
                        input.into_iter().map(|fut| yard.spawn(0, fut)).collect();
                    futures_executor::block_on(async move {
                        for handle in handle_vec {
                            handle.await;
                        }
                    })
                },
                BatchSize::SmallInput,
            )
        });
    }

    {
        group.bench_function("smolscale", |b| {
            b.iter_batched(
                future_creation,
                |input| {
                    let handle_vec: Vec<_> =
                        input.into_iter().map(|fut| smolscale::spawn(fut)).collect();
                    futures_executor::block_on(async move {
                        for handle in handle_vec {
                            handle.await;
                        }
                    })
                },
                BatchSize::SmallInput,
            )
        });
    }

    {
        let bevy_taskpool = bevy_tasks::TaskPool::new();

        group.bench_function("bevy", |b| {
            b.iter_batched(
                future_creation,
                |input| {
                    let handle_vec: Vec<_> = input
                        .into_iter()
                        .map(|fut| bevy_taskpool.spawn(fut))
                        .collect();
                    futures_executor::block_on(async move {
                        for handle in handle_vec {
                            handle.await;
                        }
                    })
                },
                BatchSize::SmallInput,
            )
        });
    }

    {
        let tokio_runtime = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(std::thread::available_parallelism().unwrap().get())
            .build()
            .unwrap();

        group.bench_function("tokio", |b| {
            b.iter_batched(
                future_creation,
                |input| {
                    let handle_vec: Vec<_> = input
                        .into_iter()
                        .map(|fut| tokio_runtime.spawn(fut))
                        .collect();
                    futures_executor::block_on(async move {
                        for handle in handle_vec {
                            let _ = handle.await;
                        }
                    })
                },
                BatchSize::SmallInput,
            )
        });
    }

    {
        group.bench_function("rayon", |b| {
            b.iter_batched(
                || (),
                |_| {
                    rayon::scope(|s| {
                        for scen in scenarios.0.iter() {
                            s.spawn(|_| {
                                black_box(mesh.path(scen.from, scen.to));
                            });
                        }
                    });
                },
                BatchSize::SmallInput,
            )
        });
    }

    {
        group.bench_function("single_threaded", |b| {
            b.iter_batched(
                || (),
                |_| {
                    for scen in scenarios.0.iter() {
                        black_box(mesh.path(scen.from, scen.to));
                    }
                },
                BatchSize::SmallInput,
            )
        });
    }
}

criterion_group!(benches, polyanya);
criterion_main!(benches);
