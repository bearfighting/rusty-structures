use data_structures::range::{Maximum, Minimum, SparseTable};
use data_structures::{FenwickTree, LazySegmentTree, LazySegmentTreeSpec, SegmentTree};

struct RangeAddSum;
impl LazySegmentTreeSpec for RangeAddSum {
    type Value = i64;
    type Update = i64;

    fn identity() -> i64 {
        0
    }

    fn combine(left: &i64, right: &i64) -> i64 {
        left + right
    }

    fn apply(delta: &i64, sum: &i64, len: usize) -> i64 {
        sum + delta * len as i64
    }

    fn compose(old: &i64, new: &i64) -> i64 {
        old + new
    }
}

pub fn run() {
    let values = [3_i64, 1, 4, 1, 5, 9, 2, 6];
    let tree = SegmentTree::from_values(&values).expect("small sums do not overflow");

    // A range-query structure is designed to combine intervals efficiently;
    // it does not expose its internal tree as an element iterator.
    let whole = tree.range_sum(0, values.len()).unwrap().unwrap();
    let middle = tree.range_sum(2, 6).unwrap().unwrap();
    println!("sum of all values: {whole}; sum of [2, 6): {middle}");

    // Iterate the source slice when individual values are needed; query the
    // tree when an aggregate over an interval is needed.
    let odd_values: Vec<_> = values
        .iter()
        .copied()
        .filter(|value| value % 2 == 1)
        .collect();
    println!("odd source values: {odd_values:?}");

    let mut traffic = FenwickTree::from_values(&[3, 5, 2, 8]);
    traffic.add(2, 4);
    println!("traffic views in hours 1..4: {:?}", traffic.range_sum(1, 4));

    let mut rates = LazySegmentTree::<RangeAddSum>::from_values(&[10, 10, 10, 10]);
    rates.update_range(1, 4, 3).unwrap();
    println!("adjusted total: {:?}", rates.query(0, 4));

    let temperatures = [11, 7, 9, 3, 8, 6];
    let minima = SparseTable::new(&temperatures, Minimum);
    let maxima = SparseTable::new(&temperatures, Maximum);
    println!(
        "historical min/max: {:?}/{:?}",
        minima.query(1, 5),
        maxima.query(1, 5)
    );
}
