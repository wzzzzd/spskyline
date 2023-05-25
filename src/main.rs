#![feature(iterator_try_collect)]

use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::time::Instant;

use anyhow::{anyhow, Result};
use clap::Parser;
use petgraph::prelude::DiGraphMap;

#[derive(Parser, Debug)]
struct Args {
    /// Path to the edge file
    #[arg(short)]
    edge_file: PathBuf,

    /// Path to the node keyword file
    #[arg(short)]
    node_keyword_file: PathBuf,

    /// List of keyword sets delimited by space. Example: "1,2,3 4,5,6"
    #[arg(value_parser = parse_keywords)]
    queries: Vec<Vec<u32>>,
}

fn parse_keywords(s: &str) -> Result<Vec<u32>> {
    Ok(s.split(',').map(|k| k.parse()).try_collect()?)
}

fn build_graph(
    edge_file_path: &Path,
    node_keyword_file_path: &Path,
) -> Result<(DiGraphMap<u32, ()>, HashMap<u32, Vec<u32>>)> {
    let edge_file = File::open(edge_file_path)?;
    let node_keyword_file = File::open(node_keyword_file_path)?;
    let mut graph = DiGraphMap::new();
    let reader = BufReader::new(edge_file);
    for (line_number, line) in reader.lines().enumerate() {
        let line = line?;
        let (source, targets) = line.split_once(':').ok_or(anyhow!(
            "expect ':' at line {} in {}",
            line_number + 1,
            edge_file_path.display()
        ))?;
        let source = source.parse()?;
        for target in targets
            .trim_matches(|x: char| x.is_whitespace() || x == ',')
            .split(',')
            .map(|t| t.parse::<u32>())
        {
            let target = target?;
            if graph.add_edge(source, target, ()).is_some() {
                return Err(anyhow!(
                    "duplicate edge found: source: {}, target: {}.",
                    source,
                    target
                ));
            }
        }
    }
    let mut node_to_keyword = HashMap::new();
    let reader = BufReader::new(node_keyword_file);
    for (line_number, line) in reader.lines().enumerate() {
        let line = line?;
        let (node, keywords) = line.split_once(':').ok_or(anyhow!(
            "expect ':' at line {} in {}",
            line_number + 1,
            node_keyword_file_path.display()
        ))?;
        let node = node.parse()?;
        let mut keywords: Vec<_> = keywords
            .trim_matches(|x: char| x.is_whitespace() || x == ',')
            .split(',')
            .map(|t| t.parse::<u32>())
            .try_collect()?;
        keywords.sort_unstable();
        if node_to_keyword.insert(node, keywords).is_some() {
            return Err(anyhow!("duplicate node found: {}.", node));
        }
    }
    Ok((graph, node_to_keyword))
}

fn main() -> Result<()> {
    let args = Args::parse();
    let start = Instant::now();
    let (graph, node_to_keyword) = build_graph(&args.edge_file, &args.node_keyword_file)?;
    let building_time = start.elapsed();
    println!("Building graph: {}", building_time.as_secs_f64());

    for keywords in args.queries {
        let start = Instant::now();
        let result =
            skyline::semantic_place_skyline::<_, _, u32>(&graph, &node_to_keyword, &keywords);
        let exec_time = start.elapsed();
        println!("Keywords: {:?}", keywords);
        println!("Execution time: {}", exec_time.as_secs_f64());
        for (root, dist) in result {
            for (k, d) in keywords.iter().zip(dist) {
                println!("{}: {} distance {}", root, k, d);
            }
        }
        println!();
    }
    Ok(())
}
