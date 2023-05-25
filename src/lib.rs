#![feature(is_some_and)]

use std::cmp::Ordering;
use std::collections::{HashMap, VecDeque};
use std::hash::Hash;

use num_traits::bounds::UpperBounded;
use num_traits::{One, Zero};
use petgraph::visit::{IntoNeighborsDirected, IntoNodeIdentifiers};
use petgraph::Direction;

/// Main query function. Each entry of `node_to_keyword` should be sorted.
pub fn semantic_place_skyline<G, K, D>(
    graph: G,
    node_to_keyword: &HashMap<G::NodeId, Vec<K>>,
    keywords: &[K],
) -> Vec<(G::NodeId, Vec<D>)>
where
    G: IntoNodeIdentifiers + IntoNeighborsDirected,
    G::NodeId: Hash + Ord,
    K: Ord,
    D: Copy + Ord + Zero + One + UpperBounded,
{
    // at least one keyword should be provided
    assert!(!keywords.is_empty());
    // initialize keyword distances
    let mut dists: HashMap<_, _> = graph
        .node_identifiers()
        .map(|node| (node, vec![D::max_value(); keywords.len()]))
        .collect();
    // for each keyword, calculate the distance from each node to the nodes containing it
    // implemented by multi-source bfs
    for (keyword_idx, keyword) in keywords.iter().enumerate() {
        let mut queue: VecDeque<_> = graph
            .node_identifiers()
            .filter(|node| {
                node_to_keyword
                    .get(node)
                    .is_some_and(|node_keywords| node_keywords.binary_search(keyword).is_ok())
            })
            .collect();
        for node in &queue {
            *dists.get_mut(node).unwrap().get_mut(keyword_idx).unwrap() = D::zero();
        }
        while let Some(current) = queue.pop_front() {
            let dist = dists[&current][keyword_idx];
            for nbr in graph.neighbors_directed(current, Direction::Incoming) {
                let nbr_dist = dists
                    .get_mut(&nbr)
                    .and_then(|v| v.get_mut(keyword_idx))
                    .unwrap();
                if dist + D::one() < *nbr_dist {
                    *nbr_dist = dist + D::one();
                    queue.push_back(nbr);
                }
            }
        }
    }
    // find the minimal elements in the partially ordered set
    dists
        .iter()
        .filter(|(_, du)| {
            dists
                .values()
                .all(|dv| partial_cmp(dv, du) != Some(Ordering::Less))
        })
        .map(|(u, du)| (*u, du.clone()))
        .collect()
}

fn partial_cmp<D: Ord>(dv1: &[D], dv2: &[D]) -> Option<Ordering> {
    assert_eq!(dv1.len(), dv2.len());
    if dv1.is_empty() {
        return Some(Ordering::Equal);
    }
    let (dv1_first, dv1) = dv1.split_first().unwrap();
    let (dv2_first, dv2) = dv2.split_first().unwrap();
    match dv1_first.cmp(dv2_first) {
        Ordering::Less => dv1
            .iter()
            .zip(dv2)
            .all(|(d1, d2)| d1 <= d2)
            .then_some(Ordering::Less),
        Ordering::Greater => dv1
            .iter()
            .zip(dv2)
            .all(|(d1, d2)| d1 >= d2)
            .then_some(Ordering::Greater),
        Ordering::Equal => partial_cmp(dv1, dv2),
    }
}
