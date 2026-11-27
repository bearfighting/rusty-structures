use data_structures::{Deque, FixedCapacityQueue, Queue, Stack, Vector};

pub fn run() {
    // Adapters transform borrowed values; collect builds another Vector.
    let scores: Vector<i32> = (1..=8).collect();
    let adjusted: Vector<_> = scores
        .iter()
        .copied()
        .filter(|score| score % 2 == 0)
        .map(|score| score * 10)
        .collect();
    println!("even scores scaled: {adjusted:?}");

    // Mutable iteration changes values without changing the structure.
    let mut temperatures: Vector<i32> = [18, 20, 22].into_iter().collect();
    for temperature in &mut temperatures {
        *temperature += 1;
    }
    println!("adjusted temperatures: {temperatures:?}");

    // Each structure's iteration order follows its abstract behavior.
    let stack: Stack<_> = ["parse", "compile", "link"].into_iter().collect();
    println!(
        "stack top to bottom: {:?}",
        stack.iter().copied().collect::<Vec<_>>()
    );

    let queue: Queue<_> = ["first", "second", "third"].into_iter().collect();
    println!("queue FIFO: {:?}", queue.into_iter().collect::<Vec<_>>());

    let mut deque = Deque::new();
    deque.push_back("middle");
    deque.push_front("front");
    deque.push_back("back");
    let mut ends = deque.iter();
    println!(
        "deque from both ends: {:?}, {:?}",
        ends.next(),
        ends.next_back()
    );

    // Short-circuiting consumers stop as soon as the answer is known.
    let first_long_name = ["Ada", "Grace", "Linus", "Margaret"]
        .into_iter()
        .find(|name| name.len() > 5);
    println!("first name longer than five letters: {first_long_name:?}");

    let mut packets = FixedCapacityQueue::with_capacity(2);
    packets.try_enqueue("packet-A").unwrap();
    packets.try_enqueue("packet-B").unwrap();
    let rejected = packets.try_enqueue("packet-C").unwrap_err();
    let processed = packets.dequeue().unwrap();
    packets.try_enqueue(rejected).unwrap();
    println!(
        "processed {processed}; pending packets: {:?}",
        packets.iter().collect::<Vec<_>>()
    );
}
