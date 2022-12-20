use std::io::prelude::*;
use std::{env, fs::File};

fn main() {
	let args: Vec<_> = env::args().collect();
	let mut file = File::open(&args[1]).unwrap();
	let mut contents = String::new();
	file.read_to_string(&mut contents).unwrap();

	let mut solver = meowc_sat::dimacs_cnf::parse_dimacs(&contents).unwrap();

	println!(
		r"c nyaa~ :3
c  __  __ ______ ______          _______       _____      _______
c |  \/  |  ____/ __ \ \        / / ____|     / ____|  /\|__   __|
c | \  / | |__ | |  | \ \  /\  / / |   ______| (___   /  \  | |
c | |\/| |  __|| |  | |\ \/  \/ /| |  |______|\___ \ / /\ \ | |
c | |  | | |___| |__| | \  /\  / | |____      ____) / ____ \| |
c |_|  |_|______\____/   \/  \/   \_____|    |_____/_/    \_\_|
c"
	);

	println!("c Solving...");
	let result = solver.solve();
	println!("c -------------------------- Stats ---------------------------");
	solver.print_stats();
	println!("c -------------------------- Result --------------------------");
	println!("s {}", result);
	println!("c ------------------------------------------------------------");
}
