use std::time::{Duration, Instant};

pub const MAX_VERTICES: usize = 64;
pub const BENCHMARK_REPETITIONS: usize = 25;

#[derive(Clone, Debug)]
pub struct Graph {
    adjacency: Vec<u64>,
    edges: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ColoringResult {
    pub colorable: bool,
    pub colors: Option<Vec<usize>>,
    pub recursive_calls: u128,
    pub complete_assignments: u128,
    pub duration: Duration,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GreedyResult {
    pub colors_used: usize,
    pub colors: Vec<usize>,
    pub duration: Duration,
}

#[derive(Clone, Debug)]
pub struct BenchmarkRow {
    pub algorithm: String,
    pub n: usize,
    pub edges: usize,
    pub k: usize,
    pub density: f64,
    pub seed: u64,
    pub colorable: Option<bool>,
    pub colors_used: Option<usize>,
    pub recursive_calls: Option<u128>,
    pub complete_assignments: Option<u128>,
    pub repetitions: usize,
    pub duration: Duration,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct SearchMetrics {
    recursive_calls: u128,
    complete_assignments: u128,
}

impl Graph {
    pub fn new(n: usize) -> Self {
        assert!(
            n <= MAX_VERTICES,
            "this implementation supports at most 64 vertices"
        );
        Self {
            adjacency: vec![0; n],
            edges: 0,
        }
    }

    pub fn complete(n: usize) -> Self {
        let mut graph = Self::new(n);
        for u in 0..n {
            for v in (u + 1)..n {
                graph.add_edge(u, v);
            }
        }
        graph
    }

    pub fn cycle(n: usize) -> Self {
        let mut graph = Self::new(n);
        if n > 1 {
            for u in 0..n {
                graph.add_edge(u, (u + 1) % n);
            }
        }
        graph
    }

    pub fn random(n: usize, density: f64, seed: u64) -> Self {
        assert!((0.0..=1.0).contains(&density), "density must be in [0, 1]");
        let mut graph = Self::new(n);
        let mut rng = SplitMix64::new(seed);
        let threshold = (density * u64::MAX as f64) as u64;

        for u in 0..n {
            for v in (u + 1)..n {
                if rng.next_u64() <= threshold {
                    graph.add_edge(u, v);
                }
            }
        }

        graph
    }

    pub fn vertex_count(&self) -> usize {
        self.adjacency.len()
    }

    pub fn edge_count(&self) -> usize {
        self.edges
    }

    pub fn neighbors_mask(&self, vertex: usize) -> u64 {
        self.adjacency[vertex]
    }

    pub fn degree(&self, vertex: usize) -> usize {
        self.adjacency[vertex].count_ones() as usize
    }

    pub fn add_edge(&mut self, u: usize, v: usize) {
        assert!(
            u < self.vertex_count() && v < self.vertex_count(),
            "invalid vertex"
        );
        assert!(u != v, "loops are not supported");

        let bit_v = 1_u64 << v;
        if self.adjacency[u] & bit_v == 0 {
            self.adjacency[u] |= bit_v;
            self.adjacency[v] |= 1_u64 << u;
            self.edges += 1;
        }
    }
}

pub fn validate_coloring(graph: &Graph, colors: &[usize]) -> bool {
    if colors.len() != graph.vertex_count() {
        return false;
    }

    for u in 0..graph.vertex_count() {
        let mut neighbors = graph.neighbors_mask(u);
        while neighbors != 0 {
            let v = neighbors.trailing_zeros() as usize;
            if u < v && colors[u] == colors[v] {
                return false;
            }
            neighbors &= neighbors - 1;
        }
    }

    true
}

pub fn brute_force_k_coloring(graph: &Graph, k: usize) -> ColoringResult {
    let start = Instant::now();
    let n = graph.vertex_count();
    let mut colors = vec![0; n];
    let mut metrics = SearchMetrics::default();
    let solution = brute_force_rec(graph, k, 0, &mut colors, &mut metrics);

    ColoringResult {
        colorable: solution.is_some(),
        colors: solution,
        recursive_calls: metrics.recursive_calls,
        complete_assignments: metrics.complete_assignments,
        duration: start.elapsed(),
    }
}

fn brute_force_rec(
    graph: &Graph,
    k: usize,
    index: usize,
    colors: &mut [usize],
    metrics: &mut SearchMetrics,
) -> Option<Vec<usize>> {
    metrics.recursive_calls += 1;

    if index == colors.len() {
        metrics.complete_assignments += 1;
        if validate_coloring(graph, colors) {
            return Some(colors.to_vec());
        }
        return None;
    }

    for color in 0..k {
        colors[index] = color;
        if let Some(solution) = brute_force_rec(graph, k, index + 1, colors, metrics) {
            return Some(solution);
        }
    }

    None
}

pub fn welsh_powell_coloring(graph: &Graph) -> GreedyResult {
    let start = Instant::now();
    let n = graph.vertex_count();
    let mut order: Vec<usize> = (0..n).collect();
    order.sort_by_key(|&v| (std::cmp::Reverse(graph.degree(v)), v));

    let mut colors = vec![usize::MAX; n];
    let mut colors_used = 0;

    for &vertex in &order {
        if colors[vertex] != usize::MAX {
            continue;
        }

        colors[vertex] = colors_used;
        for &candidate in &order {
            if colors[candidate] == usize::MAX
                && can_assign_color(graph, &colors, candidate, colors_used)
            {
                colors[candidate] = colors_used;
            }
        }
        colors_used += 1;
    }

    GreedyResult {
        colors_used,
        colors,
        duration: start.elapsed(),
    }
}

pub fn dsatur_k_coloring(graph: &Graph, k: usize) -> ColoringResult {
    let start = Instant::now();
    let n = graph.vertex_count();
    let mut colors = vec![usize::MAX; n];
    let mut forbidden_masks = vec![0_u64; n];
    let mut metrics = SearchMetrics::default();
    let solution = dsatur_rec(graph, k, &mut colors, &mut forbidden_masks, 0, &mut metrics);

    ColoringResult {
        colorable: solution.is_some(),
        colors: solution,
        recursive_calls: metrics.recursive_calls,
        complete_assignments: metrics.complete_assignments,
        duration: start.elapsed(),
    }
}

fn dsatur_rec(
    graph: &Graph,
    k: usize,
    colors: &mut [usize],
    forbidden_masks: &mut [u64],
    colored_count: usize,
    metrics: &mut SearchMetrics,
) -> Option<Vec<usize>> {
    metrics.recursive_calls += 1;

    if colored_count == colors.len() {
        metrics.complete_assignments += 1;
        return Some(colors.to_vec());
    }

    let vertex = select_dsatur_vertex_with_masks(graph, colors, forbidden_masks)?;
    let available = available_colors_mask(forbidden_masks[vertex], k);
    if available == 0 {
        return None;
    }

    for color in 0..k {
        if available & (1_u64 << color) == 0 {
            continue;
        }

        colors[vertex] = color;
        let changed_masks = apply_color_to_neighbors(graph, colors, forbidden_masks, vertex, color);
        if let Some(solution) = dsatur_rec(
            graph,
            k,
            colors,
            forbidden_masks,
            colored_count + 1,
            metrics,
        ) {
            return Some(solution);
        }
        rollback_forbidden_masks(forbidden_masks, changed_masks);
        colors[vertex] = usize::MAX;
    }

    None
}

fn select_dsatur_vertex_with_masks(
    graph: &Graph,
    colors: &[usize],
    forbidden_masks: &[u64],
) -> Option<usize> {
    (0..graph.vertex_count())
        .filter(|&v| colors[v] == usize::MAX)
        .max_by_key(|&v| {
            (
                forbidden_masks[v].count_ones() as usize,
                graph.degree(v),
                std::cmp::Reverse(v),
            )
        })
}

fn available_colors_mask(forbidden_mask: u64, k: usize) -> u64 {
    assert!(k <= 64, "this implementation supports at most 64 colors");
    let all_colors = if k == 64 { u64::MAX } else { (1_u64 << k) - 1 };
    all_colors & !forbidden_mask
}

fn apply_color_to_neighbors(
    graph: &Graph,
    colors: &[usize],
    forbidden_masks: &mut [u64],
    vertex: usize,
    color: usize,
) -> Vec<(usize, u64)> {
    let mut changed_masks = Vec::new();
    let color_bit = 1_u64 << color;
    let mut neighbors = graph.neighbors_mask(vertex);

    while neighbors != 0 {
        let neighbor = neighbors.trailing_zeros() as usize;
        if colors[neighbor] == usize::MAX && forbidden_masks[neighbor] & color_bit == 0 {
            changed_masks.push((neighbor, forbidden_masks[neighbor]));
            forbidden_masks[neighbor] |= color_bit;
        }
        neighbors &= neighbors - 1;
    }

    changed_masks
}

fn rollback_forbidden_masks(forbidden_masks: &mut [u64], changed_masks: Vec<(usize, u64)>) {
    for (vertex, old_mask) in changed_masks {
        forbidden_masks[vertex] = old_mask;
    }
}

fn can_assign_color(graph: &Graph, colors: &[usize], vertex: usize, color: usize) -> bool {
    let mut neighbors = graph.neighbors_mask(vertex);
    while neighbors != 0 {
        let neighbor = neighbors.trailing_zeros() as usize;
        if colors[neighbor] == color {
            return false;
        }
        neighbors &= neighbors - 1;
    }

    true
}

pub fn run_benchmark_suite() -> Vec<BenchmarkRow> {
    let cases = [
        (8, 0.30, 3, 11),
        (10, 0.35, 3, 13),
        (12, 0.40, 3, 17),
        (14, 0.35, 4, 19),
        (18, 0.30, 4, 23),
        (24, 0.25, 4, 29),
    ];

    let mut rows = Vec::new();
    for (n, density, k, seed) in cases {
        let graph = Graph::random(n, density, seed);
        let welsh = repeat_greedy_benchmark(&graph, BENCHMARK_REPETITIONS);
        rows.push(BenchmarkRow {
            algorithm: "welsh_powell".to_string(),
            n,
            edges: graph.edge_count(),
            k,
            density,
            seed,
            colorable: None,
            colors_used: Some(welsh.colors_used),
            recursive_calls: None,
            complete_assignments: None,
            repetitions: BENCHMARK_REPETITIONS,
            duration: welsh.duration,
        });

        let dsatur = repeat_exact_benchmark(&graph, k, BENCHMARK_REPETITIONS, dsatur_k_coloring);
        rows.push(BenchmarkRow {
            algorithm: "dsatur".to_string(),
            n,
            edges: graph.edge_count(),
            k,
            density,
            seed,
            colorable: Some(dsatur.colorable),
            colors_used: dsatur.colors.as_ref().map(|colors| {
                colors
                    .iter()
                    .copied()
                    .max()
                    .map_or(0, |max_color| max_color + 1)
            }),
            recursive_calls: Some(dsatur.recursive_calls),
            complete_assignments: Some(dsatur.complete_assignments),
            repetitions: BENCHMARK_REPETITIONS,
            duration: dsatur.duration,
        });

        if n <= 12 {
            let brute_force =
                repeat_exact_benchmark(&graph, k, BENCHMARK_REPETITIONS, brute_force_k_coloring);
            rows.push(BenchmarkRow {
                algorithm: "bruteforce".to_string(),
                n,
                edges: graph.edge_count(),
                k,
                density,
                seed,
                colorable: Some(brute_force.colorable),
                colors_used: brute_force.colors.as_ref().map(|colors| {
                    colors
                        .iter()
                        .copied()
                        .max()
                        .map_or(0, |max_color| max_color + 1)
                }),
                recursive_calls: Some(brute_force.recursive_calls),
                complete_assignments: Some(brute_force.complete_assignments),
                repetitions: BENCHMARK_REPETITIONS,
                duration: brute_force.duration,
            });
        }
    }

    rows
}

pub fn format_benchmark_csv(rows: &[BenchmarkRow]) -> String {
    let mut output = String::from(
        "algorithm,n,edges,k,density,seed,colorable,colors_used,recursive_calls,complete_assignments,repetitions,avg_duration_ms\n",
    );

    for row in rows {
        output.push_str(&format!(
            "{},{},{},{},{:.2},{},{},{},{},{},{},{:.6}\n",
            row.algorithm,
            row.n,
            row.edges,
            row.k,
            row.density,
            row.seed,
            row.colorable
                .map_or(String::new(), |value| value.to_string()),
            row.colors_used
                .map_or(String::new(), |value| value.to_string()),
            row.recursive_calls
                .map_or(String::new(), |value| value.to_string()),
            row.complete_assignments
                .map_or(String::new(), |value| value.to_string()),
            row.repetitions,
            row.duration.as_secs_f64() * 1000.0
        ));
    }

    output
}

fn repeat_exact_benchmark(
    graph: &Graph,
    k: usize,
    repetitions: usize,
    algorithm: fn(&Graph, usize) -> ColoringResult,
) -> ColoringResult {
    assert!(repetitions > 0, "repetitions must be positive");
    let mut result = algorithm(graph, k);
    let mut total_duration = result.duration;

    for _ in 1..repetitions {
        let next = algorithm(graph, k);
        assert_eq!(result.colorable, next.colorable);
        assert_eq!(result.recursive_calls, next.recursive_calls);
        assert_eq!(result.complete_assignments, next.complete_assignments);
        total_duration += next.duration;
    }

    result.duration = average_duration(total_duration, repetitions);
    result
}

fn repeat_greedy_benchmark(graph: &Graph, repetitions: usize) -> GreedyResult {
    assert!(repetitions > 0, "repetitions must be positive");
    let mut result = welsh_powell_coloring(graph);
    let mut total_duration = result.duration;

    for _ in 1..repetitions {
        let next = welsh_powell_coloring(graph);
        assert_eq!(result.colors_used, next.colors_used);
        total_duration += next.duration;
    }

    result.duration = average_duration(total_duration, repetitions);
    result
}

fn average_duration(total_duration: Duration, repetitions: usize) -> Duration {
    Duration::from_secs_f64(total_duration.as_secs_f64() / repetitions as f64)
}

#[derive(Clone, Debug)]
struct SplitMix64 {
    state: u64,
}

impl SplitMix64 {
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_graph_is_one_colorable() {
        let graph = Graph::new(5);
        let result = dsatur_k_coloring(&graph, 1);

        assert!(result.colorable);
        assert!(validate_coloring(&graph, result.colors.as_ref().unwrap()));
    }

    #[test]
    fn even_cycle_is_two_colorable() {
        let graph = Graph::cycle(6);
        let result = dsatur_k_coloring(&graph, 2);

        assert!(result.colorable);
        assert!(validate_coloring(&graph, result.colors.as_ref().unwrap()));
    }

    #[test]
    fn odd_cycle_is_not_two_colorable() {
        let graph = Graph::cycle(5);
        let result = dsatur_k_coloring(&graph, 2);

        assert!(!result.colorable);
    }

    #[test]
    fn k4_is_not_three_colorable() {
        let graph = Graph::complete(4);
        let result = dsatur_k_coloring(&graph, 3);

        assert!(!result.colorable);
    }

    #[test]
    fn brute_force_and_dsatur_agree_on_random_graphs() {
        for seed in 1..20 {
            let graph = Graph::random(9, 0.35, seed);
            let brute = brute_force_k_coloring(&graph, 3);
            let dsatur = dsatur_k_coloring(&graph, 3);

            assert_eq!(brute.colorable, dsatur.colorable);
            if let Some(colors) = dsatur.colors {
                assert!(validate_coloring(&graph, &colors));
            }
        }
    }

    #[test]
    fn validator_rejects_adjacent_equal_colors() {
        let graph = Graph::complete(3);
        assert!(!validate_coloring(&graph, &[0, 1, 1]));
        assert!(validate_coloring(&graph, &[0, 1, 2]));
    }

    #[test]
    fn zero_colors_only_color_empty_graph() {
        let empty = Graph::new(0);
        assert!(brute_force_k_coloring(&empty, 0).colorable);
        assert!(dsatur_k_coloring(&empty, 0).colorable);

        let non_empty = Graph::new(1);
        assert!(!brute_force_k_coloring(&non_empty, 0).colorable);
        assert!(!dsatur_k_coloring(&non_empty, 0).colorable);
    }

    #[test]
    fn welsh_powell_returns_valid_colorings() {
        for seed in 1..10 {
            let graph = Graph::random(12, 0.30, seed);
            let result = welsh_powell_coloring(&graph);

            assert!(validate_coloring(&graph, &result.colors));
            assert!(result.colors_used <= graph.vertex_count());
        }
    }
}
