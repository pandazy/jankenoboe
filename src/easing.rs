use std::collections::HashMap;

fn fibo(n: u8) -> u16 {
    match n {
        0 => 0,
        1 => 1,
        _ => fibo(n - 1) + fibo(n - 2),
    }
}

fn shrink(n: u16) -> u16 {
    n * 2 / 9
}

fn default_easing(n: u8) -> u16 {
    let current_val = shrink(fibo(n));
    let next_val = shrink(fibo(n + 1));
    let diff = next_val - current_val;
    if diff == 0 {
        1
    } else {
        diff
    }
}

///
/// # new_easing_map
/// ```
/// use std::collections::HashMap;
/// use anisonstud::amq::learning::easing::new_easing_map;
///
/// let map = new_easing_map(20);
/// assert_eq!(map.get(&0), Some(&1));
/// assert_eq!(map.get(&1), Some(&1));
/// assert_eq!(map.get(&2), Some(&1));
/// assert_eq!(map.get(&3), Some(&1));
/// assert_eq!(map.get(&4), Some(&1));
/// assert_eq!(map.get(&5), Some(&1));
/// assert_eq!(map.get(&6), Some(&1));
/// assert_eq!(map.get(&7), Some(&2));
/// assert_eq!(map.get(&8), Some(&3));
/// assert_eq!(map.get(&9), Some(&5));
/// assert_eq!(map.get(&10), Some(&7));
/// assert_eq!(map.get(&11), Some(&13));
/// assert_eq!(map.get(&12), Some(&19));
/// assert_eq!(map.get(&13), Some(&32));
/// assert_eq!(map.get(&14), Some(&52));
/// assert_eq!(map.get(&15), Some(&84));
/// assert_eq!(map.get(&16), Some(&135));
/// assert_eq!(map.get(&17), Some(&220));
/// assert_eq!(map.get(&18), Some(&355));
/// assert_eq!(map.get(&19), Some(&574));
pub fn new_easing_map(max_level: u8) -> HashMap<u8, u16> {
    let mut map = HashMap::new();
    for i in 0..max_level {
        map.insert(i, default_easing(i));
    }
    map
}
