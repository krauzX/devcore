use std::collections::HashMap;

pub fn two_sum(nums: &[i32], target: i32) -> (usize, usize) {
    let mut map: HashMap<i32, usize> = HashMap::new();
    for (i, &num) in nums.iter().enumerate() {
        if let Some(&j) = map.get(&(target - num)) {
            return (j, i);
        }
        map.insert(num, i);
    }
    (0, 0)
}
