use graph_coloring::{
    Graph, brute_force_k_coloring, dsatur_k_coloring, validate_coloring, welsh_powell_coloring,
};
use std::env;
use std::process;

#[derive(Debug)]
struct Args {
    algorithm: String,
    n: usize,
    k: usize,
    density: f64,
    seed: u64,
}

fn main() {
    let args = parse_args().unwrap_or_else(|message| {
        eprintln!("{message}");
        print_usage();
        process::exit(2);
    });

    let graph = Graph::random(args.n, args.density, args.seed);
    println!(
        "graph: n={}, edges={}, density={:.2}, seed={}",
        graph.vertex_count(),
        graph.edge_count(),
        args.density,
        args.seed
    );

    match args.algorithm.as_str() {
        "bruteforce" => {
            let result = brute_force_k_coloring(&graph, args.k);
            print_exact_result("bruteforce", args.k, &graph, result);
        }
        "dsatur" => {
            let result = dsatur_k_coloring(&graph, args.k);
            print_exact_result("dsatur", args.k, &graph, result);
        }
        "welsh-powell" | "welsh_powell" => {
            let result = welsh_powell_coloring(&graph);
            println!("algorithm: welsh_powell");
            println!("colors_used: {}", result.colors_used);
            println!("valid: {}", validate_coloring(&graph, &result.colors));
            println!("duration_ms: {:.6}", result.duration.as_secs_f64() * 1000.0);
            println!("colors: {:?}", result.colors);
        }
        _ => {
            eprintln!("unknown algorithm: {}", args.algorithm);
            print_usage();
            process::exit(2);
        }
    }
}

fn print_exact_result(
    algorithm: &str,
    k: usize,
    graph: &Graph,
    result: graph_coloring::ColoringResult,
) {
    println!("algorithm: {algorithm}");
    println!("k: {k}");
    println!("colorable: {}", result.colorable);
    println!("recursive_calls: {}", result.recursive_calls);
    println!("complete_assignments: {}", result.complete_assignments);
    println!("duration_ms: {:.6}", result.duration.as_secs_f64() * 1000.0);
    if let Some(colors) = result.colors {
        println!("valid: {}", validate_coloring(graph, &colors));
        println!("colors: {:?}", colors);
    }
}

fn parse_args() -> Result<Args, String> {
    let mut args = Args {
        algorithm: "dsatur".to_string(),
        n: 12,
        k: 3,
        density: 0.35,
        seed: 1,
    };

    let mut iter = env::args().skip(1);
    while let Some(flag) = iter.next() {
        let value = iter
            .next()
            .ok_or_else(|| format!("missing value for argument {flag}"))?;

        match flag.as_str() {
            "--algorithm" | "-a" => args.algorithm = value,
            "--n" => args.n = parse_value(&flag, &value)?,
            "--k" => args.k = parse_value(&flag, &value)?,
            "--density" | "-d" => args.density = parse_value(&flag, &value)?,
            "--seed" | "-s" => args.seed = parse_value(&flag, &value)?,
            _ => return Err(format!("unknown argument: {flag}")),
        }
    }

    if args.k > 64 {
        return Err("k must be at most 64".to_string());
    }
    if args.n > graph_coloring::MAX_VERTICES {
        return Err(format!(
            "n must be at most {}",
            graph_coloring::MAX_VERTICES
        ));
    }
    if !(0.0..=1.0).contains(&args.density) {
        return Err("density must be in [0, 1]".to_string());
    }

    Ok(args)
}

fn parse_value<T>(flag: &str, value: &str) -> Result<T, String>
where
    T: std::str::FromStr,
{
    value
        .parse()
        .map_err(|_| format!("invalid value for {flag}: {value}"))
}

fn print_usage() {
    eprintln!(
        "usage: cargo run --bin coloring -- --algorithm dsatur --n 12 --k 3 --density 0.35 --seed 1"
    );
    eprintln!("algorithms: dsatur, bruteforce, welsh_powell");
}
