/// Default number of levels in the spaced repetition system.
pub const MAX_LEVEL: u8 = 20;

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
    if diff == 0 { 1 } else { diff }
}

/// Generate a level_up_path as a vector of wait-days for each level.
///
/// Default 20-level path:
/// `[1, 1, 1, 1, 1, 1, 1, 2, 3, 5, 7, 13, 19, 32, 52, 84, 135, 220, 355, 574]`
pub fn generate_level_up_path(max_level: u8) -> Vec<u16> {
    (0..max_level).map(default_easing).collect()
}

/// Generate a level_up_path as a JSON array string.
pub fn generate_level_up_path_json(max_level: u8) -> String {
    let path = generate_level_up_path(max_level);
    serde_json::to_string(&path).expect("Failed to serialize level_up_path")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_level_up_path() {
        let path = generate_level_up_path(20);
        assert_eq!(
            path,
            vec![
                1, 1, 1, 1, 1, 1, 1, 2, 3, 5, 7, 13, 19, 32, 52, 84, 135, 220, 355, 574
            ]
        );
    }

    #[test]
    fn test_generate_level_up_path_json() {
        let json = generate_level_up_path_json(20);
        assert_eq!(
            json,
            "[1,1,1,1,1,1,1,2,3,5,7,13,19,32,52,84,135,220,355,574]"
        );
    }
}
