use graph_coloring::{format_benchmark_csv, run_benchmark_suite};

fn main() {
    let rows = run_benchmark_suite();
    print!("{}", format_benchmark_csv(&rows));
}
