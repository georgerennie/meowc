use criterion::{black_box, criterion_group, criterion_main, Criterion};
use meowc_sat::dimacs_cnf::parse_dimacs;
use std::{fs, time::Duration};

pub fn criterion_benchmark(c: &mut Criterion) {
	let mut group = c.benchmark_group("cnfs");
	group
		.sample_size(40)
		.significance_level(0.08)
		.noise_threshold(0.05)
		.warm_up_time(Duration::from_millis(500))
		.measurement_time(Duration::from_secs(3));

	const CNFS_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/cnfs");
	for file in fs::read_dir(CNFS_PATH).unwrap() {
		let file = file.unwrap();

		let dimacs = fs::read_to_string(file.path()).unwrap();
		let solver = parse_dimacs(&dimacs).unwrap();

		group.bench_function(file.file_name().to_str().unwrap(), |b| {
			b.iter(|| black_box(solver.clone()).solve())
		});
	}

	group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
