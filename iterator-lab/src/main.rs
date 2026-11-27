mod collections;
mod graphs;
mod heaps;
mod linear;
mod ranges;

fn main() {
    println!("== Linear collections ==");
    linear::run();
    println!("\n== Maps, sets, and ordered trees ==");
    collections::run();
    println!("\n== Heaps ==");
    heaps::run();
    println!("\n== Graph traversal ==");
    graphs::run();
    println!("\n== Range queries ==");
    ranges::run();
}
