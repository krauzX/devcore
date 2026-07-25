use std::time::Duration;

use anyhow::{Context, Result};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};

const GRAPHQL_URL: &str = "https://leetcode.com/graphql";

const PROBLEM_LIST_QUERY: &str = r#"
query problemsetQuestionList($categorySlug: String, $limit: Int, $skip: Int, $filters: QuestionListFilterInput) {
  problemsetQuestionList: questionList(
    categorySlug: $categorySlug
    limit: $limit
    skip: $skip
    filters: $filters
  ) {
    total: totalNum
    questions: data {
      title
      titleSlug
      difficulty
      acRate
      topicTags { name }
    }
  }
}
"#;

const QUESTION_DETAIL_QUERY: &str = r#"
query questionDetail($titleSlug: String!) {
  question(titleSlug: $titleSlug) {
    title
    content
    hints
    codeSnippets { langSlug, code }
    topicTags { name }
  }
}
"#;

const DAILY_CHALLENGE_QUERY: &str = r#"
query questionOfToday {
  activeDailyCodingChallengeQuestion {
    question {
      title
      titleSlug
      difficulty
      acRate
      content
      hints
      codeSnippets { langSlug, code }
      topicTags { name }
    }
  }
}
"#;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicTag {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CodeSnippet {
    pub lang_slug: String,
    pub code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProblemListItem {
    pub title: String,
    pub title_slug: String,
    pub difficulty: String,
    pub acceptance: f64,
    pub tags: Vec<TopicTag>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeetCodeProblem {
    pub title: String,
    pub title_slug: String,
    pub difficulty: String,
    pub acceptance: f64,
    pub tags: Vec<TopicTag>,
    pub description: String,
    pub hints: Vec<String>,
    pub code_snippets: Vec<CodeSnippet>,
}

#[derive(Deserialize)]
struct QuestionListResponse {
    data: QuestionListData,
}

#[derive(Deserialize)]
struct QuestionListData {
    problemset_question_list: ProblemSetQuestionList,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ProblemSetQuestionList {
    #[allow(dead_code)]
    total: usize,
    data: Vec<ProblemListItemRaw>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ProblemListItemRaw {
    title: String,
    title_slug: String,
    difficulty: String,
    #[serde(default)]
    ac_rate: f64,
    #[serde(default)]
    topic_tags: Vec<TopicTag>,
}

#[derive(Deserialize)]
struct QuestionDetailResponse {
    data: QuestionDetailData,
}

#[derive(Deserialize)]
struct QuestionDetailData {
    question: Option<QuestionDetailRaw>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct QuestionDetailRaw {
    title: String,
    title_slug: String,
    difficulty: String,
    #[serde(default)]
    ac_rate: f64,
    content: Option<String>,
    #[serde(default)]
    hints: Vec<String>,
    #[serde(default)]
    code_snippets: Vec<CodeSnippet>,
    #[serde(default)]
    topic_tags: Vec<TopicTag>,
}

#[derive(Deserialize)]
struct DailyChallengeResponse {
    data: DailyChallengeData,
}

#[derive(Deserialize)]
struct DailyChallengeData {
    active_daily_coding_challenge_question: DailyChallengeItem,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct DailyChallengeItem {
    question: QuestionDetailRaw,
}

#[derive(Serialize)]
struct ProblemListVariables {
    category_slug: String,
    limit: usize,
    skip: usize,
    filters: ProblemListFilters,
}

#[derive(Serialize)]
struct ProblemListFilters {
    #[serde(skip_serializing_if = "Option::is_none")]
    difficulty: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tags: Option<Vec<String>>,
}

#[derive(Serialize)]
struct QueryBody<T: Serialize> {
    query: &'static str,
    variables: T,
}

pub struct LeetCodeClient {
    client: Client,
}

impl LeetCodeClient {
    pub fn new() -> Result<Self, anyhow::Error> {
        let client = Client::builder()
            .user_agent("devcore/0.1")
            .timeout(Duration::from_secs(10))
            .connect_timeout(Duration::from_secs(5))
            .build()
            .context("failed to build HTTP client")?;
        Ok(Self { client })
    }

    pub fn fetch_problem_list(
        &self,
        difficulty: Option<&str>,
        tags: Option<&[String]>,
        limit: usize,
    ) -> Result<Vec<ProblemListItem>> {
        let body = QueryBody {
            query: PROBLEM_LIST_QUERY,
            variables: ProblemListVariables {
                category_slug: String::new(),
                limit,
                skip: 0,
                filters: ProblemListFilters {
                    difficulty: difficulty.map(|d| d.to_string()),
                    tags: tags.map(|t| t.to_vec()),
                },
            },
        };

        let resp: QuestionListResponse = self
            .client
            .post(GRAPHQL_URL)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .context("failed to send request to LeetCode")?
            .error_for_status()
            .context("LeetCode returned an error")?
            .json()
            .context("failed to parse LeetCode response")?;

        let items = resp
            .data
            .problemset_question_list
            .data
            .into_iter()
            .map(|raw| ProblemListItem {
                title: raw.title,
                title_slug: raw.title_slug,
                difficulty: raw.difficulty,
                acceptance: raw.ac_rate,
                tags: raw.topic_tags,
            })
            .collect();
        Ok(items)
    }

    pub fn fetch_problem(&self, title_slug: &str) -> Result<LeetCodeProblem> {
        let body = QueryBody {
            query: QUESTION_DETAIL_QUERY,
            variables: serde_json::json!({ "titleSlug": title_slug }),
        };

        let resp: QuestionDetailResponse = self
            .client
            .post(GRAPHQL_URL)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .context("failed to send request to LeetCode")?
            .error_for_status()
            .context("LeetCode returned an error")?
            .json()
            .context("failed to parse LeetCode response")?;

        let raw = resp
            .data
            .question
            .ok_or_else(|| anyhow::anyhow!("problem '{}' not found", title_slug))?;

        Ok(LeetCodeProblem {
            title: raw.title,
            title_slug: raw.title_slug,
            difficulty: raw.difficulty,
            acceptance: raw.ac_rate,
            tags: raw.topic_tags,
            description: raw.content.unwrap_or_default(),
            hints: raw.hints,
            code_snippets: raw.code_snippets,
        })
    }

    pub fn fetch_daily_challenge(&self) -> Result<LeetCodeProblem> {
        let body = QueryBody {
            query: DAILY_CHALLENGE_QUERY,
            variables: serde_json::json!({}),
        };

        let resp: DailyChallengeResponse = self
            .client
            .post(GRAPHQL_URL)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .context("failed to send request to LeetCode")?
            .error_for_status()
            .context("LeetCode returned an error")?
            .json()
            .context("failed to parse LeetCode response")?;

        let raw = resp
            .data
            .active_daily_coding_challenge_question
            .question;

        Ok(LeetCodeProblem {
            title: raw.title,
            title_slug: raw.title_slug,
            difficulty: raw.difficulty,
            acceptance: raw.ac_rate,
            tags: raw.topic_tags,
            description: raw.content.unwrap_or_default(),
            hints: raw.hints,
            code_snippets: raw.code_snippets,
        })
    }
}

pub fn format_problem_list(problems: &[ProblemListItem]) -> String {
    if problems.is_empty() {
        return "No problems found.".to_string();
    }

    let mut out = String::from("LeetCode Problems:\n");
    for (i, p) in problems.iter().enumerate() {
        let tags_str = p
            .tags
            .iter()
            .map(|t| t.name.as_str())
            .collect::<Vec<_>>()
            .join(", ");
        out.push_str(&format!(
            "{:>3}. {} [{}] ({:.1}%) — {}\n",
            i + 1,
            p.title,
            p.difficulty,
            p.acceptance,
            tags_str,
        ));
    }
    out
}

pub fn format_problem(problem: &LeetCodeProblem) -> String {
    let mut out = String::new();
    out.push_str(&format!("{} [{}] ({:.1}%)\n", problem.title, problem.difficulty, problem.acceptance));

    if !problem.tags.is_empty() {
        let tags_str = problem
            .tags
            .iter()
            .map(|t| t.name.as_str())
            .collect::<Vec<_>>()
            .join(", ");
        out.push_str(&format!("Tags: {}\n", tags_str));
    }

    out.push_str("\nDescription:\n");
    let desc = html_to_text(&problem.description);
    out.push_str(&desc);
    out.push('\n');

    if !problem.hints.is_empty() {
        out.push_str("\nHints:\n");
        for (i, h) in problem.hints.iter().enumerate() {
            let hint_text = html_to_text(h);
            out.push_str(&format!("  {}. {}\n", i + 1, hint_text));
        }
    }

    if !problem.code_snippets.is_empty() {
        out.push_str("\nCode Snippets:\n");
        for s in &problem.code_snippets {
            out.push_str(&format!("--- {} ---\n{}\n", s.lang_slug, s.code));
        }
    }

    out
}

fn html_to_text(html: &str) -> String {
    let mut out = String::with_capacity(html.len());
    let mut in_tag = false;
    let mut in_entity = false;
    let mut entity_buf = String::new();

    for ch in html.chars() {
        if ch == '<' {
            in_tag = true;
            continue;
        }
        if ch == '>' && in_tag {
            in_tag = false;
            continue;
        }
        if in_tag {
            continue;
        }
        if ch == '&' {
            in_entity = true;
            entity_buf.clear();
            continue;
        }
        if in_entity {
            if ch == ';' {
                in_entity = false;
                let replacement = if let Some(digit_str) = entity_buf.strip_prefix('#') {
                    let code = if digit_str.starts_with('x') || digit_str.starts_with('X') {
                        u32::from_str_radix(&digit_str[1..], 16).ok()
                    } else {
                        digit_str.parse::<u32>().ok()
                    };
                    match code.and_then(std::char::from_u32) {
                        Some(c) => {
                            out.push(c);
                            continue;
                        }
                        None => {
                            out.push('&');
                            out.push_str(&entity_buf);
                            ";"
                        }
                    }
                } else {
                    match entity_buf.as_str() {
                        "amp" => "&",
                        "lt" => "<",
                        "gt" => ">",
                        "quot" => "\"",
                        "apos" => "'",
                        "nbsp" => " ",
                        "le" => "≤",
                        "ge" => "≥",
                        "times" => "×",
                        "divide" => "÷",
                        _ => {
                            out.push('&');
                            out.push_str(&entity_buf);
                            ";"
                        }
                    }
                };
                out.push_str(replacement);
            } else {
                entity_buf.push(ch);
            }
            continue;
        }
        out.push(ch);
    }

    if in_entity {
        out.push('&');
        out.push_str(&entity_buf);
    }

    out
}
