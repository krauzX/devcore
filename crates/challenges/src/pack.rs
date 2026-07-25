use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Difficulty {
    Easy,
    Medium,
    Hard,
}

impl Difficulty {
    pub fn from_level(level: u32) -> Option<Self> {
        match level {
            1 => Some(Self::Easy),
            2 => Some(Self::Medium),
            3 => Some(Self::Hard),
            _ => None,
        }
    }
}

impl FromStr for Difficulty {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "easy" => Ok(Self::Easy),
            "medium" => Ok(Self::Medium),
            "hard" => Ok(Self::Hard),
            _ => Err(format!("invalid difficulty '{}'. Valid: easy, medium, hard", s)),
        }
    }
}

impl fmt::Display for Difficulty {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Difficulty::Easy => write!(f, "Easy"),
            Difficulty::Medium => write!(f, "Medium"),
            Difficulty::Hard => write!(f, "Hard"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestCase {
    pub input: String,
    pub expected: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Problem {
    pub id: String,
    pub name: String,
    pub description: String,
    pub difficulty: Difficulty,
    pub hints: Vec<String>,
    pub test_cases: Vec<TestCase>,
    pub skeleton: String,
    pub solution: String,
    pub category: String,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProblemPack {
    pub id: String,
    pub name: String,
    pub category: String,
    pub difficulty: Difficulty,
    pub problems: Vec<Problem>,
    pub installed: bool,
}

pub fn builtin_packs() -> Vec<ProblemPack> {
    vec![
        arrays_easy_pack(),
        linked_lists_easy_pack(),
        stacks_easy_pack(),
        trees_medium_pack(),
        graphs_hard_pack(),
    ]
}

fn arrays_easy_pack() -> ProblemPack {
    ProblemPack {
        id: "arrays-easy".to_string(),
        name: "Array Fundamentals".to_string(),
        category: "Arrays".to_string(),
        difficulty: Difficulty::Easy,
        installed: false,
        problems: vec![
            Problem {
                id: "two-sum".to_string(),
                name: "Two Sum".to_string(),
                description: "Given an array of integers `nums` and an integer `target`, return the indices of the two numbers such that they add up to `target`.\n\nYou may assume that each input would have exactly one solution, and you may not use the same element twice.\n\nThe answer can be returned in any order.\n\n# Examples\n\n```\ntwo_sum(&[2, 7, 11, 15], 9) -> (0, 1)\ntwo_sum(&[3, 2, 4], 6) -> (1, 2)\ntwo_sum(&[3, 3], 6) -> (0, 1)\n```".to_string(),
                difficulty: Difficulty::Easy,
                hints: vec![
                    "A brute force approach checks every pair — O(n^2). Can you do better?".to_string(),
                    "Use a hash map to store each value's index as you iterate.".to_string(),
                    "For each element, check if `target - current` is already in the map.".to_string(),
                ],
                test_cases: vec![
                    TestCase {
                        input: "nums = [2,7,11,15], target = 9".to_string(),
                        expected: "(0, 1)".to_string(),
                        description: "Basic case: 2 + 7 = 9".to_string(),
                    },
                    TestCase {
                        input: "nums = [3,2,4], target = 6".to_string(),
                        expected: "(1, 2)".to_string(),
                        description: "3 is not the answer (3+3 would reuse the same element)".to_string(),
                    },
                    TestCase {
                        input: "nums = [3,3], target = 6".to_string(),
                        expected: "(0, 1)".to_string(),
                        description: "Duplicate values at different indices".to_string(),
                    },
                ],
                skeleton: r#"use std::collections::HashMap;

pub fn two_sum(nums: &[i32], target: i32) -> (usize, usize) {
    // TODO: use a HashMap to store each value's index as you iterate.
    // For each element, check if (target - current) is already in the map.
    todo!()
}
"#.to_string(),
                solution: r#"use std::collections::HashMap;

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
"#.to_string(),
                category: "Arrays".to_string(),
                tags: vec!["hash-map".to_string(), "two-pointer".to_string()],
            },
            Problem {
                id: "max-subarray".to_string(),
                name: "Maximum Subarray".to_string(),
                description: "Given an integer array `nums`, find the subarray with the largest sum, and return its sum.\n\nA subarray is a contiguous non-empty sequence of elements within an array.\n\n# Examples\n\n```\nmax_subarray(&[-2, 1, -3, 4, -1, 2, 1, -5, 4]) -> 6\n// The subarray [4, -1, 2, 1] has the largest sum = 6\n\nmax_subarray(&[1]) -> 1\n\nmax_subarray(&[5, 4, -1, 7, 8]) -> 23\n```".to_string(),
                difficulty: Difficulty::Easy,
                hints: vec![
                    "This is Kadane's algorithm — keep track of the running sum.".to_string(),
                    "If the running sum drops below 0, reset it to the current element.".to_string(),
                    "Maintain a global maximum that tracks the best sum seen so far.".to_string(),
                ],
                test_cases: vec![
                    TestCase {
                        input: "nums = [-2,1,-3,4,-1,2,1,-5,4]".to_string(),
                        expected: "6".to_string(),
                        description: "Subarray [4,-1,2,1] has the largest sum".to_string(),
                    },
                    TestCase {
                        input: "nums = [1]".to_string(),
                        expected: "1".to_string(),
                        description: "Single element".to_string(),
                    },
                    TestCase {
                        input: "nums = [5,4,-1,7,8]".to_string(),
                        expected: "23".to_string(),
                        description: "Entire array is the answer".to_string(),
                    },
                ],
                skeleton: r#"pub fn max_subarray(nums: &[i32]) -> i32 {
    // TODO: track current_sum and global_max using Kadane's algorithm.
    // If current_sum drops below 0, reset it to the current element.
    todo!()
}
"#.to_string(),
                solution: r#"pub fn max_subarray(nums: &[i32]) -> i32 {
    let mut best = nums[0];
    let mut current = nums[0];
    for &num in &nums[1..] {
        current = num.max(current + num);
        best = best.max(current);
    }
    best
}
"#.to_string(),
                category: "Arrays".to_string(),
                tags: vec!["dynamic-programming".to_string(), "kadane".to_string()],
            },
        ],
    }
}

fn linked_lists_easy_pack() -> ProblemPack {
    ProblemPack {
        id: "linked-lists-easy".to_string(),
        name: "Linked List Basics".to_string(),
        category: "Linked Lists".to_string(),
        difficulty: Difficulty::Easy,
        installed: false,
        problems: vec![Problem {
            id: "reverse-linked-list".to_string(),
            name: "Reverse Linked List".to_string(),
            description: "Given the `head` of a singly linked list, reverse the list, and return the reversed list.\n\nImplement the `reverse_list` function that takes a vector of i32 representing the linked list nodes in order, and returns the reversed vector.\n\n# Examples\n\n```\nreverse_list(&[1, 2, 3, 4, 5]) -> vec![5, 4, 3, 2, 1]\nreverse_list(&[1, 2]) -> vec![2, 1]\nreverse_list(&[]) -> vec![]\n```".to_string(),
            difficulty: Difficulty::Easy,
            hints: vec![
                "You can solve this iteratively with three pointers: prev, current, next.".to_string(),
                "At each step, redirect current.next to prev, then advance all three.".to_string(),
                "The iterative approach is O(n) time and O(1) extra space.".to_string(),
            ],
            test_cases: vec![
                TestCase {
                    input: "[1,2,3,4,5]".to_string(),
                    expected: "[5,4,3,2,1]".to_string(),
                    description: "Standard case with 5 nodes".to_string(),
                },
                TestCase {
                    input: "[1,2]".to_string(),
                    expected: "[2,1]".to_string(),
                    description: "Two-node list".to_string(),
                },
                TestCase {
                    input: "[]".to_string(),
                    expected: "[]".to_string(),
                    description: "Empty list".to_string(),
                },
            ],
            skeleton: r#"pub fn reverse_list(head: &[i32]) -> Vec<i32> {
    // TODO: iterate through the list and build a new reversed vector.
    // Hint: you can use a mutable result and push each element to the front,
    // or iterate in reverse.
    todo!()
}
"#.to_string(),
            solution: r#"pub fn reverse_list(head: &[i32]) -> Vec<i32> {
    head.iter().rev().copied().collect()
}
"#.to_string(),
            category: "Linked Lists".to_string(),
            tags: vec!["linked-list".to_string(), "two-pointer".to_string()],
        }],
    }
}

fn stacks_easy_pack() -> ProblemPack {
    ProblemPack {
        id: "stacks-easy".to_string(),
        name: "Stack Essentials".to_string(),
        category: "Stacks".to_string(),
        difficulty: Difficulty::Easy,
        installed: false,
        problems: vec![Problem {
            id: "valid-parentheses".to_string(),
            name: "Valid Parentheses".to_string(),
            description: "Given a string `s` containing just the characters '(', ')', '{', '}', '[' and ']', determine if the input string is valid.\n\nAn input string is valid if:\n1. Open brackets must be closed by the same type of brackets.\n2. Open brackets must be closed in the correct order.\n3. Every close bracket has a corresponding open bracket of the same type.\n\n# Examples\n\n```\nis_valid(\"()\") -> true\nis_valid(\"()[]{}\") -> true\nis_valid(\"(]\") -> false\nis_valid(\"([)]\") -> false\nis_valid(\"{[]}\") -> true\n```".to_string(),
            difficulty: Difficulty::Easy,
            hints: vec![
                "Use a stack to track opening brackets.".to_string(),
                "When you see a closing bracket, check if the top of the stack is the matching opener.".to_string(),
                "At the end, the stack should be empty for the string to be valid.".to_string(),
            ],
            test_cases: vec![
                TestCase {
                    input: "\"()\"".to_string(),
                    expected: "true".to_string(),
                    description: "Simple matching pair".to_string(),
                },
                TestCase {
                    input: "\"()[]{}\"".to_string(),
                    expected: "true".to_string(),
                    description: "Multiple matching pairs".to_string(),
                },
                TestCase {
                    input: "\"(]\"".to_string(),
                    expected: "false".to_string(),
                    description: "Mismatched types".to_string(),
                },
                TestCase {
                    input: "\"([)]\"".to_string(),
                    expected: "false".to_string(),
                    description: "Interleaved brackets".to_string(),
                },
                TestCase {
                    input: "\"{[]}\"".to_string(),
                    expected: "true".to_string(),
                    description: "Nested brackets".to_string(),
                },
            ],
            skeleton: r#"pub fn is_valid(s: &str) -> bool {
    // TODO: use a stack to track opening brackets.
    // When you see a closing bracket, check if the top of the stack
    // is the matching opener. At the end, the stack should be empty.
    todo!()
}
"#.to_string(),
            solution: r#"pub fn is_valid(s: &str) -> bool {
    let mut stack: Vec<char> = Vec::new();
    for ch in s.chars() {
        match ch {
            '(' | '{' | '[' => stack.push(ch),
            ')' => {
                if stack.pop() != Some('(') {
                    return false;
                }
            }
            '}' => {
                if stack.pop() != Some('{') {
                    return false;
                }
            }
            ']' => {
                if stack.pop() != Some('[') {
                    return false;
                }
            }
            _ => {}
        }
    }
    stack.is_empty()
}
"#.to_string(),
            category: "Stacks".to_string(),
            tags: vec!["stack".to_string(), "string".to_string()],
        }],
    }
}

fn trees_medium_pack() -> ProblemPack {
    ProblemPack {
        id: "trees-medium".to_string(),
        name: "Binary Tree Traversal".to_string(),
        category: "Trees".to_string(),
        difficulty: Difficulty::Medium,
        installed: false,
        problems: vec![Problem {
            id: "binary-tree-inorder".to_string(),
            name: "Binary Tree Inorder Traversal".to_string(),
            description: "Given the root of a binary tree, return the inorder traversal of its nodes' values.\n\nInorder traversal visits nodes in the order: Left, Root, Right.\n\nYou are given the tree as a level-order vector where `None` represents a missing node.\n\n# Examples\n\n```\ninorder_traversal(&[Some(1), None, Some(2), None, None, Some(3)]) -> vec![1, 3, 2]\ninorder_traversal(&[]) -> vec![]\ninorder_traversal(&[Some(1)]) -> vec![1]\n```".to_string(),
            difficulty: Difficulty::Medium,
            hints: vec![
                "Inorder traversal is recursive: visit left subtree, then root, then right subtree.".to_string(),
                "You can solve this iteratively using an explicit stack.".to_string(),
                "The recursive solution has O(n) time and O(h) space where h is the tree height.".to_string(),
            ],
            test_cases: vec![
                TestCase {
                    input: "[Some(1), None, Some(2), None, None, Some(3)]".to_string(),
                    expected: "[1, 3, 2]".to_string(),
                    description: "Root has only right subtree with left child".to_string(),
                },
                TestCase {
                    input: "[]".to_string(),
                    expected: "[]".to_string(),
                    description: "Empty tree".to_string(),
                },
                TestCase {
                    input: "[Some(1)]".to_string(),
                    expected: "[1]".to_string(),
                    description: "Single node".to_string(),
                },
            ],
            skeleton: r#"pub fn inorder_traversal(tree: &[Option<i32>]) -> Vec<i32> {
    // TODO: implement inorder traversal (left, root, right).
    // The tree is given as a level-order vector where None = missing node.
    // For index i: left child = 2*i+1, right child = 2*i+2.
    // Hint: use recursion or an explicit stack.
    todo!()
}
"#.to_string(),
            solution: r#"pub fn inorder_traversal(tree: &[Option<i32>]) -> Vec<i32> {
    if tree.is_empty() {
        return vec![];
    }
    let mut result = Vec::new();
    fn traverse(tree: &[Option<i32>], idx: usize, result: &mut Vec<i32>) {
        if idx >= tree.len() || tree[idx].is_none() {
            return;
        }
        traverse(tree, 2 * idx + 1, result);
        if let Some(val) = tree[idx] {
            result.push(val);
        }
        traverse(tree, 2 * idx + 2, result);
    }
    traverse(tree, 0, &mut result);
    result
}
"#.to_string(),
            category: "Trees".to_string(),
            tags: vec!["binary-tree".to_string(), "recursion".to_string(), "dfs".to_string()],
        }],
    }
}

fn graphs_hard_pack() -> ProblemPack {
    ProblemPack {
        id: "graphs-hard".to_string(),
        name: "Graph Algorithms".to_string(),
        category: "Graphs".to_string(),
        difficulty: Difficulty::Hard,
        installed: false,
        problems: vec![Problem {
            id: "word-ladder".to_string(),
            name: "Word Ladder".to_string(),
            description: "A transformation sequence from word `begin_word` to word `end_word` uses a dictionary `word_list` where each transformed word must be exactly one letter different from the previous word, and every intermediate word must be in `word_list`.\n\nGiven `begin_word`, `end_word`, and `word_list`, return the number of words in the shortest transformation sequence from `begin_word` to `end_word`, or 0 if no such sequence exists.\n\n# Examples\n\n```\nword_ladder(\"hit\", \"cog\", &[\"hot\",\"dot\",\"dog\",\"lot\",\"log\",\"cog\"]) -> 5\n// hit -> hot -> dot -> dog -> cog\n\nword_ladder(\"hit\", \"cog\", &[\"hot\",\"dot\",\"dog\",\"lot\",\"log\"]) -> 0\n// cog is not in word_list, so no path exists\n```".to_string(),
            difficulty: Difficulty::Hard,
            hints: vec![
                "This is a classic BFS problem on an implicit graph.".to_string(),
                "Each word is a node; edges connect words that differ by exactly one letter.".to_string(),
                "Start from begin_word and do level-by-level BFS, checking all one-letter mutations against the word set.".to_string(),
                "For efficiency, preprocess the dictionary or use a pattern-matching approach to find neighbors.".to_string(),
            ],
            test_cases: vec![
                TestCase {
                    input: "begin = \"hit\", end = \"cog\", word_list = [\"hot\",\"dot\",\"dog\",\"lot\",\"log\",\"cog\"]".to_string(),
                    expected: "5".to_string(),
                    description: "hit -> hot -> dot -> dog -> cog (length 5)".to_string(),
                },
                TestCase {
                    input: "begin = \"hit\", end = \"cog\", word_list = [\"hot\",\"dot\",\"dog\",\"lot\",\"log\"]".to_string(),
                    expected: "0".to_string(),
                    description: "Target word not in dictionary".to_string(),
                },
                TestCase {
                    input: "begin = \"a\", end = \"c\", word_list = [\"a\",\"b\",\"c\"]".to_string(),
                    expected: "2".to_string(),
                    description: "Direct transformation: a -> c".to_string(),
                },
            ],
            skeleton: r#"use std::collections::{VecDeque, HashSet};

pub fn word_ladder(begin_word: &str, end_word: &str, word_list: &[&str]) -> i32 {
    // TODO: BFS from begin_word to end_word.
    // Each node is a word; edges connect words differing by one letter.
    // Use a HashSet for the dictionary and another for visited words.
    // At each BFS level, try all 26 letters at every position.
    // Return the level count when end_word is reached, or 0 if impossible.
    todo!()
}
"#.to_string(),
            solution: r#"use std::collections::{VecDeque, HashSet};

pub fn word_ladder(begin_word: &str, end_word: &str, word_list: &[&str]) -> i32 {
    let word_set: HashSet<&str> = word_list.iter().copied().collect();
    if !word_set.contains(end_word) {
        return 0;
    }
    let mut queue = VecDeque::new();
    let mut visited = HashSet::new();
    queue.push_back((begin_word.to_string(), 1));
    visited.insert(begin_word.to_string());
    while let Some((word, level)) = queue.pop_front() {
        let mut word_chars: Vec<char> = word.chars().collect();
        for i in 0..word_chars.len() {
            let orig = word_chars[i];
            for c in 'a'..='z' {
                if c == orig {
                    continue;
                }
                word_chars[i] = c;
                let next: String = word_chars.iter().collect();
                if next == end_word {
                    return level + 1;
                }
                if word_set.contains(next.as_str()) && !visited.contains(&next) {
                    visited.insert(next.clone());
                    queue.push_back((next, level + 1));
                }
            }
            word_chars[i] = orig;
        }
    }
    0
}
"#.to_string(),
            category: "Graphs".to_string(),
            tags: vec!["bfs".to_string(), "graph".to_string(), "string".to_string()],
        }],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builtin_packs_count() {
        let packs = builtin_packs();
        assert_eq!(packs.len(), 5);
    }

    #[test]
    fn test_pack_has_problems() {
        let packs = builtin_packs();
        for pack in &packs {
            assert!(!pack.problems.is_empty(), "pack '{}' has no problems", pack.id);
        }
    }
}
