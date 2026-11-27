use data_structures::BinaryHeap;
use data_structures::PriorityQueue;
use std::cmp::Reverse;

pub fn run() {
    let heap: BinaryHeap<_> = [4, 9, 2, 7, 1].into_iter().collect();

    // Iteration exposes heap storage order, which is not sorted order.
    println!("heap layout: {:?}", heap.iter().collect::<Vec<_>>());

    // Repeated pop is the right way to consume values by priority.
    let mut heap: BinaryHeap<_> = [4, 9, 2, 7, 1].into_iter().collect();
    let descending: Vec<_> = std::iter::from_fn(|| heap.pop()).collect();
    println!("priority order: {descending:?}");

    let mut triage = PriorityQueue::new();
    triage.push((5, Reverse(1)), "database unavailable");
    triage.push((3, Reverse(2)), "disk pressure");
    triage.push((5, Reverse(3)), "payment latency");
    println!("stable alert priority: {:?}", triage.pop());

    let mut slowest = BinaryHeap::new();
    for latency_ms in [12, 80, 31, 240, 95, 17, 180] {
        slowest.push(Reverse(latency_ms));
        if slowest.len() > 3 {
            slowest.pop();
        }
    }
    let mut top: Vec<_> = std::iter::from_fn(|| slowest.pop())
        .map(|Reverse(latency)| latency)
        .collect();
    top.reverse();
    println!("top three request latencies: {top:?}");
}
