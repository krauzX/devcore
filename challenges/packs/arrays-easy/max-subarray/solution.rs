pub fn max_subarray(nums: &[i32]) -> i32 {
    let mut best = nums[0];
    let mut current = nums[0];
    for &num in &nums[1..] {
        current = num.max(current + num);
        best = best.max(current);
    }
    best
}
