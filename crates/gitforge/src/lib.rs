pub mod analysis;
pub mod progress;
pub mod streak;

pub use analysis::{analyze_repo, detect_languages, LanguageStat, RepoAnalysis};
pub use progress::{
    add_xp, get_progress, init_skill_schema, SkillAxis, SkillProgress, XP_PER_LEVEL, MAX_LEVEL,
};
pub use streak::{compute_streak, Streak};
