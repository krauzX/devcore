use std::path::{Path, PathBuf};
use std::time::Duration;

use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use devcore_academic::{Course, Deadline, GradeEntry, SemesterStore};
use devcore_challenges::{
    ChallengeEngine, Difficulty, OfflineProblem, ProblemPack, ProjectEngine, ProjectPack,
    ProjectProgress,
};
use devcore_core::{DevCoreConfig, Store};
use devcore_devtrack::{LanguageStat, RepoAnalysis, SkillAxis, SkillProgress, Streak};
use ratatui::prelude::*;
use ratatui::widgets::*;

use crate::academic_tab::render_academic_tab;
use crate::challenges_tab::render_challenges_tab;
use crate::course_view::render_course_view;
use crate::dashboard::render_dashboard;
use crate::git_tab::render_git_tab;
use crate::theme;
use crate::widgets;
use crate::widgets::StatusKind;

const DEADLINE_DEFAULT_DAYS: u32 = 30;
const PROBLEMS_PER_PAGE: usize = 20;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Tab {
    Dashboard,
    Academic,
    Git,
    Challenges,
}

impl Tab {
    fn all() -> &'static [Tab] {
        &[Tab::Dashboard, Tab::Academic, Tab::Git, Tab::Challenges]
    }

    fn title(&self) -> &'static str {
        match self {
            Tab::Dashboard => "Dashboard",
            Tab::Academic => "Academic",
            Tab::Git => "Git",
            Tab::Challenges => "Challenges",
        }
    }

    fn key(&self) -> char {
        match self {
            Tab::Dashboard => '1',
            Tab::Academic => '2',
            Tab::Git => '3',
            Tab::Challenges => '4',
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum InputMode {
    Normal,
    AddingXp,
    AddingCourse,
    AddingDeadline,
    AddingGrade,
    SelectingSemester,
    ConfirmingInstall,
    ConfirmingRemove,
    ViewingDetail,
    ViewingCourse,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum XpField {
    Axis,
    Amount,
    Reason,
}

pub(crate) struct InputField {
    pub label: &'static str,
    pub value: String,
}

pub struct App {
    should_quit: bool,
    active_tab: Tab,
    project_root: PathBuf,
    pub(crate) config: DevCoreConfig,
    pub(crate) academic_store: SemesterStore,
    pub(crate) semesters: Vec<devcore_academic::Semester>,
    pub(crate) current_semester: Option<devcore_academic::Semester>,
    pub(crate) upcoming_deadlines: Vec<Deadline>,
    pub(crate) sgpa: Option<f64>,
    pub(crate) cgpa: Option<f64>,
    pub(crate) course_count: i32,
    pub(crate) total_credits: i32,
    pub(crate) streak: Option<Streak>,
    pub(crate) skill_progress: Vec<SkillProgress>,
    pub(crate) repo_analysis: Option<RepoAnalysis>,
    pub(crate) languages: Vec<LanguageStat>,
    pub(crate) input_mode: InputMode,
    pub(crate) deadline_days: u32,
    pub(crate) selected_deadline_index: Option<usize>,
    pub(crate) xp_field: XpField,
    pub(crate) xp_axis_index: usize,
    pub(crate) xp_amount: String,
    pub(crate) xp_reason: String,
    pub(crate) input_fields: Vec<InputField>,
    pub(crate) current_field: usize,
    pub(crate) semester_cursor: usize,
    pub(crate) status_msg: Option<String>,
    core_store: Store,
    pub(crate) packs: Vec<ProblemPack>,
    pub(crate) installed_count: usize,
    pub(crate) offline_problems: Vec<OfflineProblem>,
    pub(crate) offline_page: usize,
    pub(crate) offline_total_pages: usize,
    pub(crate) offline_total: usize,
    pub(crate) selected_pack: Option<usize>,
    pub(crate) selected_problem: Option<usize>,
    pub(crate) difficulty_filter: Option<Difficulty>,
    pub(crate) show_projects: bool,
    pub(crate) projects: Vec<ProjectPack>,
    pub(crate) selected_project: Option<usize>,
    pub(crate) installed_pack_ids: Vec<String>,
    pub(crate) project_progress: Vec<ProjectProgress>,
    pub(crate) course_stage_cursor: usize,
    pub(crate) show_solutions: bool,
}

impl App {
    pub fn new(project_root: &Path) -> Result<Self> {
        let config = DevCoreConfig::load(project_root)?;

        let academic_store = SemesterStore::open(project_root)?;
        let mut semesters = academic_store.list_semesters().unwrap_or_default();
        let mut seed_error: Option<String> = None;
        if semesters.is_empty() {
            if let Err(e) = academic_store.seed_2026_data() {
                seed_error = Some(format!("Seed failed: {}", e));
            }
            semesters = academic_store.list_semesters().unwrap_or_default();
        }
        let current_semester = academic_store.current_semester();
        let sgpa = current_semester.as_ref().and_then(|s| {
            let conn = academic_store.conn().ok()?;
            GradeEntry::compute_sgpa(&conn, &s.id)
        });
        let deadline_days = DEADLINE_DEFAULT_DAYS;
        let upcoming_deadlines = current_semester.as_ref().and_then(|_| {
            let conn = academic_store.conn().ok()?;
            Some(Deadline::upcoming(&conn, deadline_days as i64).unwrap_or_default())
        }).unwrap_or_default();

        let (course_count, total_credits) = current_semester.as_ref().map(|s| {
            let conn = academic_store.conn().ok();
            match conn {
                Some(c) => (
                    Course::count_for_semester(&c, &s.id),
                    Course::total_credits_for_semester(&c, &s.id),
                ),
                None => (0, 0),
            }
        }).unwrap_or((0, 0));

        let cgpa = academic_store.conn().ok().and_then(|conn| {
            GradeEntry::compute_cgpa(&conn)
        });

        let streak = devcore_devtrack::compute_streak(project_root).ok();
        let repo_analysis = devcore_devtrack::analyze_repo(project_root).ok();
        let languages = devcore_devtrack::detect_languages(project_root);

        let core_store = Store::open(project_root)?;
        let skill_progress = {
            let conn = core_store.conn()?;
            devcore_devtrack::init_skill_schema(&conn).ok();
            devcore_devtrack::get_progress(&conn).unwrap_or_default()
        };

        let data_dir = project_root.join(".devcore");
        let engine = ChallengeEngine::new(&data_dir);
        let packs = engine.list_available().to_vec();
        let installed_count = engine.list_installed().len();
        let installed_pack_ids: Vec<String> = engine.list_installed().into_iter().map(|p| p.id.clone()).collect();

        let per_page = PROBLEMS_PER_PAGE;
        let offline_result = engine.list_offline(None, 1, per_page);

        let project_engine = ProjectEngine::new(&data_dir);
        let projects = project_engine.list_available().to_vec();
        let project_progress = project_engine.list_progress();

        Ok(Self {
            should_quit: false,
            active_tab: Tab::Dashboard,
            project_root: project_root.to_path_buf(),
            config,
            academic_store,
            semesters,
            current_semester,
            upcoming_deadlines,
            sgpa,
            cgpa,
            course_count,
            total_credits,
            streak,
            skill_progress,
            repo_analysis,
            languages,
            input_mode: InputMode::Normal,
            deadline_days: DEADLINE_DEFAULT_DAYS,
            selected_deadline_index: None,
            xp_field: XpField::Axis,
            xp_axis_index: 0,
            xp_amount: String::new(),
            xp_reason: String::new(),
            input_fields: Vec::new(),
            current_field: 0,
            semester_cursor: 0,
            status_msg: seed_error,
            core_store,
            packs,
            installed_count,
            offline_problems: offline_result.problems,
            offline_page: offline_result.page,
            offline_total_pages: offline_result.total_pages,
            offline_total: offline_result.total,
            selected_pack: None,
            selected_problem: None,
            difficulty_filter: None,
            show_projects: false,
            projects,
            selected_project: None,
            installed_pack_ids,
            project_progress,
            course_stage_cursor: 0,
            show_solutions: false,
        })
    }

    pub fn run(&mut self) -> Result<()> {
        let exit_status = self.event_loop();
        ratatui::restore();
        exit_status
    }

    fn refresh_deadlines(&mut self) {
        self.upcoming_deadlines = self.current_semester.as_ref().and_then(|_| {
            let conn = self.academic_store.conn().ok()?;
            Some(Deadline::upcoming(&conn, self.deadline_days as i64).unwrap_or_default())
        }).unwrap_or_default();
        self.selected_deadline_index = None;
    }

    fn refresh_offline_problems(&mut self, difficulty: Option<&str>, page: usize) {
        let data_dir = self.project_root.join(".devcore");
        let engine = ChallengeEngine::new(&data_dir);
        let per_page = PROBLEMS_PER_PAGE;
        let result = engine.list_offline(difficulty, page, per_page);
        self.offline_problems = result.problems;
        self.offline_page = result.page;
        self.offline_total_pages = result.total_pages;
        self.offline_total = result.total;
    }

    fn refresh_packs(&mut self) {
        let data_dir = self.project_root.join(".devcore");
        let engine = ChallengeEngine::new(&data_dir);
        self.packs = engine.list_available().to_vec();
        self.installed_count = engine.list_installed().len();
        self.installed_pack_ids = engine.list_installed().into_iter().map(|p| p.id.clone()).collect();
    }

    fn refresh_project_progress(&mut self) {
        let data_dir = self.project_root.join(".devcore");
        let engine = ProjectEngine::new(&data_dir);
        self.project_progress = engine.list_progress();
    }

    fn get_difficulty_str(&self) -> Option<&'static str> {
        match self.difficulty_filter {
            Some(Difficulty::Easy) => Some("easy"),
            Some(Difficulty::Medium) => Some("medium"),
            Some(Difficulty::Hard) => Some("hard"),
            None => None,
        }
    }

    pub(crate) fn all_problems(&self) -> Vec<(&ProblemPack, &devcore_challenges::Problem)> {
        self.packs
            .iter()
            .flat_map(|pack| {
                pack.problems.iter().map(move |problem| (pack, problem))
            })
            .collect()
    }

    fn event_loop(&mut self) -> Result<()> {
        let mut terminal = ratatui::init();
        loop {
            terminal.draw(|frame| self.draw(frame))?;

            if event::poll(Duration::from_millis(250))? {
                if let Event::Key(key) = event::read()? {
                    if key.kind == KeyEventKind::Press {
                        match self.input_mode {
                            InputMode::Normal => self.handle_normal_input(key.code),
                            InputMode::AddingXp => self.handle_adding_xp(key.code),
                            InputMode::AddingCourse
                            | InputMode::AddingDeadline
                            | InputMode::AddingGrade => {
                                self.handle_academic_input(key.code);
                            }
                            InputMode::SelectingSemester => {
                                self.handle_semester_input(key.code);
                            }
                            InputMode::ConfirmingInstall => self.handle_confirm_install(key.code),
                            InputMode::ConfirmingRemove => self.handle_confirm_remove(key.code),
                            InputMode::ViewingDetail => self.handle_viewing_detail(key.code),
                            InputMode::ViewingCourse => {
                                match key.code {
                                    KeyCode::Esc => {
                                        self.input_mode = InputMode::Normal;
                                    }
                                    KeyCode::Up | KeyCode::Char('k') => {
                                        if self.course_stage_cursor > 0 {
                                            self.course_stage_cursor -= 1;
                                        }
                                    }
                                    KeyCode::Down | KeyCode::Char('j') => {
                                        if let Some(idx) = self.selected_project {
                                            if idx < self.projects.len() {
                                                let max = self.projects[idx].stages.len().saturating_sub(1);
                                                if self.course_stage_cursor < max {
                                                    self.course_stage_cursor += 1;
                                                }
                                            }
                                        }
                                    }
                                    KeyCode::Char('s') => {
                                        self.show_solutions = !self.show_solutions;
                                    }
                                    KeyCode::Char('c') => {
                                        if let Some(idx) = self.selected_project {
                                            if idx < self.projects.len() {
                                                let project_id = self.projects[idx].id.clone();
                                                let data_dir = self.project_root.join(".devcore");
                                                let mut engine = ProjectEngine::new(&data_dir);
                                                let stage = self.course_stage_cursor;
                                                match engine.get_progress(&project_id) {
                                                    Some(prog) if !prog.is_complete() => {
                                                        match engine.set_stage_completed(&project_id, stage) {
                                                            Ok(()) => {
                                                                self.status_msg = Some(format!(
                                                                    "Marked stage {} as complete",
                                                                    stage + 1
                                                                ));
                                                                self.refresh_project_progress();
                                                            }
                                                            Err(e) => {
                                                                self.status_msg = Some(format!("Failed: {}", e));
                                                            }
                                                        }
                                                    }
                                                    None => {
                                                        let total = self.projects[idx].stages.len();
                                                        let mut progress = ProjectProgress::new(&project_id, total);
                                                        progress.completed_stages.push(stage);
                                                        if stage + 1 < total {
                                                            progress.current_stage = stage + 1;
                                                        }
                                                        match engine.save_progress(&progress) {
                                                            Ok(()) => {
                                                                self.status_msg = Some(format!(
                                                                    "Started project and marked stage {} complete",
                                                                    stage + 1
                                                                ));
                                                                self.refresh_project_progress();
                                                            }
                                                            Err(e) => {
                                                                self.status_msg = Some(format!("Failed: {}", e));
                                                            }
                                                        }
                                                    }
                                                    Some(_) => {
                                                        self.status_msg = Some("Project already complete!".into());
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                }
            }

            if self.should_quit {
                break;
            }
        }
        Ok(())
    }

    fn handle_normal_input(&mut self, code: KeyCode) {
        match code {
            KeyCode::Char('q') => self.should_quit = true,
            KeyCode::Esc => self.should_quit = true,
            KeyCode::Tab => {
                let tabs = Tab::all();
                let idx = tabs.iter().position(|t| *t == self.active_tab).unwrap_or(0);
                self.active_tab = tabs[(idx + 1) % tabs.len()];
            }
            KeyCode::Char('1') => self.active_tab = Tab::Dashboard,
            KeyCode::Char('2') => self.active_tab = Tab::Academic,
            KeyCode::Char('3') => self.active_tab = Tab::Git,
            KeyCode::Char('4') => self.active_tab = Tab::Challenges,
            _ if self.active_tab == Tab::Academic => self.handle_academic_trigger(code),
            _ if self.active_tab == Tab::Challenges => self.handle_challenges_input(code),
            _ if self.active_tab == Tab::Git => self.handle_git_input(code),
            _ => {}
        }
    }

    fn handle_academic_trigger(&mut self, code: KeyCode) {
        match code {
            KeyCode::Char('c') => {
                self.input_mode = InputMode::AddingCourse;
                self.input_fields = vec![
                    InputField { label: "Name", value: String::new() },
                    InputField { label: "Code", value: String::new() },
                    InputField { label: "Credits", value: String::new() },
                ];
                self.current_field = 0;
                self.status_msg = None;
            }
            KeyCode::Char('d') => {
                self.input_mode = InputMode::AddingDeadline;
                self.input_fields = vec![
                    InputField { label: "Title", value: String::new() },
                    InputField { label: "Due Date (YYYY-MM-DD)", value: String::new() },
                ];
                self.current_field = 0;
                self.status_msg = None;
            }
            KeyCode::Char('g') => {
                self.input_mode = InputMode::AddingGrade;
                self.input_fields = vec![
                    InputField { label: "Course Code", value: String::new() },
                    InputField { label: "Grade (e.g. O, A+, A, B+, B, C, D, F)", value: String::new() },
                ];
                self.current_field = 0;
                self.status_msg = None;
            }
            KeyCode::Char('s') => {
                self.input_mode = InputMode::SelectingSemester;
                self.semester_cursor = 0;
            }
            KeyCode::Char('x') => {
                if let Some(idx) = self.selected_deadline_index {
                    if idx < self.upcoming_deadlines.len() {
                        let deadline_id = self.upcoming_deadlines[idx].id.clone();
                        if let Ok(conn) = self.academic_store.conn() {
                            match Deadline::complete(&conn, &deadline_id) {
                                Ok(()) => {
                                    self.status_msg = Some("Deadline completed".to_string());
                                }
                                Err(e) => {
                                    self.status_msg = Some(format!("Failed to complete deadline: {}", e));
                                }
                            }
                        }
                        self.refresh_deadlines();
                    }
                }
            }
            KeyCode::Char('j') => {
                if let Some(idx) = self.selected_deadline_index {
                    if idx + 1 < self.upcoming_deadlines.len() {
                        self.selected_deadline_index = Some(idx + 1);
                    }
                } else if !self.upcoming_deadlines.is_empty() {
                    self.selected_deadline_index = Some(0);
                }
            }
            KeyCode::Char('k') => {
                if let Some(idx) = self.selected_deadline_index {
                    if idx > 0 {
                        self.selected_deadline_index = Some(idx - 1);
                    }
                }
            }
            KeyCode::Char('+') => {
                self.deadline_days = self.deadline_days.saturating_add(7);
                self.refresh_deadlines();
            }
            KeyCode::Char('-') => {
                self.deadline_days = self.deadline_days.saturating_sub(7).max(1);
                self.refresh_deadlines();
            }
            _ => {}
        }
    }

    fn handle_academic_input(&mut self, code: KeyCode) {
        match code {
            KeyCode::Esc => {
                self.input_mode = InputMode::Normal;
                self.status_msg = None;
            }
            KeyCode::Tab => {
                if !self.input_fields.is_empty() {
                    self.current_field = (self.current_field + 1) % self.input_fields.len();
                }
            }
            KeyCode::Up => {
                if self.current_field > 0 {
                    self.current_field -= 1;
                }
            }
            KeyCode::Down => {
                if !self.input_fields.is_empty() && self.current_field + 1 < self.input_fields.len() {
                    self.current_field += 1;
                }
            }
            KeyCode::Char(c) => {
                if self.current_field < self.input_fields.len() {
                    self.input_fields[self.current_field].value.push(c);
                }
            }
            KeyCode::Backspace => {
                if !self.input_fields.is_empty() && self.current_field < self.input_fields.len() {
                    self.input_fields[self.current_field].value.pop();
                }
            }
            KeyCode::Enter => {
                match self.input_mode {
                    InputMode::AddingCourse => {
                        if self.input_fields.len() >= 3 {
                            let name = self.input_fields[0].value.trim().to_string();
                            let code = self.input_fields[1].value.trim().to_string();
                            let credits = self.input_fields[2].value.trim().parse::<i32>().unwrap_or(0);
                            if !name.is_empty() && !code.is_empty() && credits > 0 {
                                if let Some(ref sem) = self.current_semester {
                                    if let Ok(conn) = self.academic_store.conn() {
                                        let course = Course {
                                            id: uuid::Uuid::new_v4().to_string(),
                                            semester_id: sem.id.clone(),
                                            name,
                                            code,
                                            credits,
                                        };
                                        match Course::add(&conn, &course) {
                                            Ok(()) => {
                                                self.course_count = Course::count_for_semester(&conn, &sem.id);
                                                self.total_credits = Course::total_credits_for_semester(&conn, &sem.id);
                                                self.status_msg = Some("Course added".into());
                                            }
                                            Err(e) => {
                                                self.status_msg = Some(format!("Failed to add course: {}", e));
                                            }
                                        }
                                    }
                                } else {
                                    self.status_msg = Some("No semester selected".into());
                                }
                            } else {
                                self.status_msg = Some("Invalid course fields".into());
                            }
                        }
                    }
                    InputMode::AddingDeadline => {
                        if self.input_fields.len() >= 2 {
                            let title = self.input_fields[0].value.trim().to_string();
                            let date_str = self.input_fields[1].value.trim();
                            if !title.is_empty() {
                                if let Ok(due_date) = chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
                                    if let Some(ref sem) = self.current_semester {
                                        if let Ok(conn) = self.academic_store.conn() {
                                            let deadline = Deadline {
                                                id: uuid::Uuid::new_v4().to_string(),
                                                semester_id: sem.id.clone(),
                                                title,
                                                description: String::new(),
                                                due_date,
                                                completed: false,
                                            };
                                            match Deadline::add(&conn, &deadline) {
                                                Ok(()) => {
                                                    self.upcoming_deadlines = Deadline::upcoming(&conn, self.deadline_days as i64).unwrap_or_default();
                                                    self.status_msg = Some("Deadline added".into());
                                                }
                                                Err(e) => {
                                                    self.status_msg = Some(format!("Failed to add deadline: {}", e));
                                                }
                                            }
                                        }
                                    } else {
                                        self.status_msg = Some("No semester selected".into());
                                    }
                                } else {
                                    self.status_msg = Some("Invalid date format (use YYYY-MM-DD)".into());
                                }
                            } else {
                                self.status_msg = Some("Title cannot be empty".into());
                            }
                        }
                    }
                    InputMode::AddingGrade => {
                        if self.input_fields.len() >= 2 {
                            let course_code = self.input_fields[0].value.trim().to_string();
                            let grade = self.input_fields[1].value.trim().to_uppercase();
                            let valid_grades = ["O", "A+", "A", "B+", "B", "C", "D", "F"];
                            if !course_code.is_empty() && valid_grades.contains(&grade.as_str()) {
                                if let Some(ref sem) = self.current_semester {
                                    if let Ok(conn) = self.academic_store.conn() {
                                        if let Ok(Some(course)) = Course::find_by_code(&conn, &course_code, &sem.id) {
                                            let entry = GradeEntry {
                                                id: uuid::Uuid::new_v4().to_string(),
                                                course_id: course.id,
                                                semester_id: sem.id.clone(),
                                                grade,
                                                exam_name: None,
                                                score: None,
                                                total: None,
                                            };
                                            match GradeEntry::add(&conn, &entry) {
                                                Ok(()) => {
                                                    self.sgpa = GradeEntry::compute_sgpa(&conn, &sem.id);
                                                    self.cgpa = GradeEntry::compute_cgpa(&conn);
                                                    self.status_msg = Some("Grade added".into());
                                                }
                                                Err(e) => {
                                                    self.status_msg = Some(format!("Failed to add grade: {}", e));
                                                }
                                            }
                                        } else {
                                            self.status_msg = Some("Course not found".into());
                                        }
                                    }
                                } else {
                                    self.status_msg = Some("No semester selected".into());
                                }
                            } else {
                                self.status_msg = Some("Invalid grade (use O, A+, A, B+, B, C, D, or F)".into());
                            }
                        }
                    }
                    _ => {}
                }
                self.input_mode = InputMode::Normal;
            }
            _ => {}
        }
    }

    fn handle_semester_input(&mut self, code: KeyCode) {
        match code {
            KeyCode::Esc => {
                self.input_mode = InputMode::Normal;
            }
            KeyCode::Up => {
                if self.semester_cursor > 0 {
                    self.semester_cursor -= 1;
                }
            }
            KeyCode::Down => {
                if self.semester_cursor + 1 < self.semesters.len() {
                    self.semester_cursor += 1;
                }
            }
            KeyCode::Enter => {
                if let Some(sem) = self.semesters.get(self.semester_cursor) {
                    let sem_id = sem.id.clone();
                    if let Err(e) = self.academic_store.set_current_semester(&sem_id) {
                        self.status_msg = Some(format!("Failed to set semester: {}", e));
                    } else {
                        self.current_semester = self.academic_store.current_semester();
                        if let Some(ref sem) = self.current_semester {
                            let conn_result = self.academic_store.conn();
                            if let Ok(conn) = conn_result {
                                self.sgpa = GradeEntry::compute_sgpa(&conn, &sem.id);
                                self.course_count = Course::count_for_semester(&conn, &sem.id);
                                self.total_credits = Course::total_credits_for_semester(&conn, &sem.id);
                                self.upcoming_deadlines = Deadline::upcoming(&conn, self.deadline_days as i64).unwrap_or_default();
                            }
                        }
                        self.cgpa = self.academic_store.conn().ok().and_then(|conn| {
                            GradeEntry::compute_cgpa(&conn)
                        });
                        self.status_msg = Some(format!("Semester set to '{}'", self.semesters[self.semester_cursor].name));
                    }
                }
                self.input_mode = InputMode::Normal;
            }
            _ => {}
        }
    }

    fn handle_git_input(&mut self, code: KeyCode) {
        if let KeyCode::Char('x') = code {
            self.input_mode = InputMode::AddingXp;
            self.xp_field = XpField::Axis;
            self.xp_axis_index = 0;
            self.xp_amount.clear();
            self.xp_reason.clear();
        }
    }

    fn handle_adding_xp(&mut self, code: KeyCode) {
        let axis_count = SkillAxis::all().len();
        match code {
            KeyCode::Esc => {
                self.input_mode = InputMode::Normal;
            }
            KeyCode::Tab => {
                self.xp_field = match self.xp_field {
                    XpField::Axis => XpField::Amount,
                    XpField::Amount => XpField::Reason,
                    XpField::Reason => XpField::Axis,
                };
            }
            KeyCode::Up => {
                if self.xp_field == XpField::Axis {
                    self.xp_axis_index = self.xp_axis_index.saturating_sub(1);
                }
            }
            KeyCode::Down => {
                if self.xp_field == XpField::Axis && self.xp_axis_index + 1 < axis_count {
                    self.xp_axis_index += 1;
                }
            }
            KeyCode::Char(c) => {
                match self.xp_field {
                    XpField::Amount => {
                        if c.is_ascii_digit() {
                            self.xp_amount.push(c);
                        }
                    }
                    XpField::Reason => {
                        self.xp_reason.push(c);
                    }
                    XpField::Axis => {}
                }
            }
            KeyCode::Backspace => {
                match self.xp_field {
                    XpField::Amount => { self.xp_amount.pop(); }
                    XpField::Reason => { self.xp_reason.pop(); }
                    XpField::Axis => {}
                }
            }
            KeyCode::Enter => {
                if let Ok(amount) = self.xp_amount.parse::<u32>() {
                    if amount > 0 && !self.xp_reason.is_empty() {
                        let axis = SkillAxis::all()[self.xp_axis_index];
                        let reason = self.xp_reason.clone();
                        if let Ok(conn) = self.core_store.conn() {
                            match devcore_devtrack::add_xp(&conn, axis, amount, &reason) {
                                Ok(updated) => {
                                    self.skill_progress.retain(|s| s.axis != axis);
                                    self.skill_progress.push(updated);
                                    self.status_msg = Some(format!("Added {} XP to {}", amount, axis.as_str()));
                                }
                                Err(e) => {
                                    self.status_msg = Some(format!("Failed to add XP: {}", e));
                                }
                            }
                        }
                    }
                }
                self.input_mode = InputMode::Normal;
            }
            _ => {}
        }
    }

    fn handle_challenges_input(&mut self, code: KeyCode) {
        match code {
            KeyCode::Char('n') => {
                let next_page = (self.offline_page + 1).min(self.offline_total_pages.max(1));
                self.refresh_offline_problems(self.get_difficulty_str(), next_page);
            }
            KeyCode::Char('p') if !self.show_projects => {
                let prev_page = self.offline_page.saturating_sub(1).max(1);
                self.refresh_offline_problems(self.get_difficulty_str(), prev_page);
            }
            KeyCode::Char('i') => {
                if let Some(idx) = self.selected_pack {
                    if idx < self.packs.len() {
                        self.input_mode = InputMode::ConfirmingInstall;
                    }
                }
            }
            KeyCode::Char('r') => {
                if let Some(idx) = self.selected_pack {
                    if idx < self.packs.len() {
                        self.input_mode = InputMode::ConfirmingRemove;
                    }
                }
            }
            KeyCode::Char('c') if self.show_projects => {
                if let Some(idx) = self.selected_project {
                    if idx < self.projects.len() {
                        let project_id = self.projects[idx].id.clone();
                        let data_dir = self.project_root.join(".devcore");
                        let mut engine = ProjectEngine::new(&data_dir);
                        match engine.get_progress(&project_id) {
                            Some(prog) if !prog.is_complete() => {
                                let current_stage = prog.current_stage;
                                match engine.set_stage_completed(&project_id, current_stage) {
                                    Ok(()) => {
                                        self.status_msg = Some(format!(
                                            "Marked stage {} as complete for '{}'",
                                            current_stage + 1,
                                            self.projects[idx].name
                                        ));
                                        self.refresh_project_progress();
                                    }
                                    Err(e) => {
                                        self.status_msg = Some(format!("Failed to complete stage: {}", e));
                                    }
                                }
                            }
                            Some(_) => {
                                self.status_msg = Some(format!(
                                    "Project '{}' is already complete!",
                                    self.projects[idx].name
                                ));
                            }
                            None => {
                                let total = self.projects[idx].stages.len();
                                let mut progress = ProjectProgress::new(&project_id, total);
                                progress.completed_stages.push(0);
                                progress.current_stage = 1.min(total - 1);
                                match engine.save_progress(&progress) {
                                    Ok(()) => {
                                        self.status_msg = Some(format!(
                                            "Started project '{}' and marked stage 1 as complete",
                                            self.projects[idx].name
                                        ));
                                        self.refresh_project_progress();
                                    }
                                    Err(e) => {
                                        self.status_msg = Some(format!("Failed to start project: {}", e));
                                    }
                                }
                            }
                        }
                    }
                }
            }
            KeyCode::Enter => {
                if self.show_projects && self.selected_project.is_some() {
                    self.course_stage_cursor = 0;
                    self.show_solutions = false;
                    self.input_mode = InputMode::ViewingCourse;
                } else if self.selected_problem.is_some() {
                    self.input_mode = InputMode::ViewingDetail;
                }
            }
            KeyCode::Char('v') if self.show_projects => {
                if self.selected_project.is_some() {
                    self.course_stage_cursor = 0;
                    self.show_solutions = false;
                    self.input_mode = InputMode::ViewingCourse;
                }
            }
            KeyCode::Char('e') => {
                self.difficulty_filter = Some(Difficulty::Easy);
                self.refresh_offline_problems(Some("easy"), 1);
            }
            KeyCode::Char('m') => {
                self.difficulty_filter = Some(Difficulty::Medium);
                self.refresh_offline_problems(Some("medium"), 1);
            }
            KeyCode::Char('h') => {
                self.difficulty_filter = Some(Difficulty::Hard);
                self.refresh_offline_problems(Some("hard"), 1);
            }
            KeyCode::Char('a') => {
                self.difficulty_filter = None;
                self.refresh_offline_problems(None, 1);
            }
            KeyCode::Char('p') if self.show_projects => {
                self.show_projects = false;
                self.selected_project = None;
            }
            KeyCode::Char('o') => {
                self.show_projects = !self.show_projects;
                if !self.show_projects {
                    self.selected_project = None;
                }
            }
            KeyCode::Up => {
                if self.show_projects {
                    if let Some(ref mut idx) = self.selected_project {
                        *idx = idx.saturating_sub(1);
                    } else if !self.projects.is_empty() {
                        self.selected_project = Some(0);
                    }
                } else if let Some(ref mut idx) = self.selected_pack {
                    *idx = idx.saturating_sub(1);
                } else if !self.packs.is_empty() {
                    self.selected_pack = Some(0);
                }
            }
            KeyCode::Down => {
                if self.show_projects {
                    let len = self.projects.len();
                    if let Some(ref mut idx) = self.selected_project {
                        if *idx + 1 < len {
                            *idx += 1;
                        }
                    } else if !self.projects.is_empty() {
                        self.selected_project = Some(0);
                    }
                } else {
                    let len = self.packs.len();
                    if let Some(ref mut idx) = self.selected_pack {
                        if *idx + 1 < len {
                            *idx += 1;
                        }
                    } else if !self.packs.is_empty() {
                        self.selected_pack = Some(0);
                    }
                }
            }
            KeyCode::Left => {
                if !self.show_projects {
                    if let Some(ref mut idx) = self.selected_problem {
                        *idx = idx.saturating_sub(1);
                    }
                }
            }
            KeyCode::Right => {
                if !self.show_projects {
                    let problems_len = self.all_problems().len();
                    if let Some(ref mut idx) = self.selected_problem {
                        if *idx + 1 < problems_len {
                            *idx += 1;
                        }
                    } else if problems_len > 0 {
                        self.selected_problem = Some(0);
                    }
                }
            }
            _ => {}
        }
    }

    fn handle_confirm_install(&mut self, code: KeyCode) {
        match code {
            KeyCode::Char('y') | KeyCode::Char('Y') => {
                if let Some(idx) = self.selected_pack {
                    if idx < self.packs.len() {
                        let pack_id = self.packs[idx].id.clone();
                        let data_dir = self.project_root.join(".devcore");
                        let mut engine = ChallengeEngine::new(&data_dir);
                        match engine.install_pack(&pack_id) {
                            Ok(()) => {
                                self.status_msg = Some(format!("Installed pack '{}'", self.packs[idx].name));
                            }
                            Err(e) => {
                                self.status_msg = Some(format!("Failed to install: {}", e));
                            }
                        }
                        self.refresh_packs();
                    }
                }
                self.input_mode = InputMode::Normal;
            }
            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                self.input_mode = InputMode::Normal;
            }
            _ => {}
        }
    }

    fn handle_confirm_remove(&mut self, code: KeyCode) {
        match code {
            KeyCode::Char('y') | KeyCode::Char('Y') => {
                if let Some(idx) = self.selected_pack {
                    if idx < self.packs.len() {
                        let pack_id = self.packs[idx].id.clone();
                        let data_dir = self.project_root.join(".devcore");
                        let mut engine = ChallengeEngine::new(&data_dir);
                        match engine.remove_pack(&pack_id) {
                            Ok(()) => {
                                self.status_msg = Some(format!("Removed pack '{}'", self.packs[idx].name));
                            }
                            Err(e) => {
                                self.status_msg = Some(format!("Failed to remove: {}", e));
                            }
                        }
                        self.refresh_packs();
                    }
                }
                self.input_mode = InputMode::Normal;
            }
            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                self.input_mode = InputMode::Normal;
            }
            _ => {}
        }
    }

    fn handle_viewing_detail(&mut self, code: KeyCode) {
        match code {
            KeyCode::Esc | KeyCode::Enter => {
                self.input_mode = InputMode::Normal;
            }
            KeyCode::Char('a') => {
                if let Some(idx) = self.selected_problem {
                    let problems = self.all_problems();
                    if idx < problems.len() {
                        let (pack, problem) = problems[idx];
                        if let Ok(conn) = self.core_store.conn() {
                            if let Err(e) = devcore_challenges::progress::init_challenge_schema(&conn) {
                                self.status_msg = Some(format!("Failed to init schema: {}", e));
                            } else if let Err(e) = devcore_challenges::progress::record_attempt(
                                &conn,
                                &problem.id,
                                &pack.id,
                                false,
                                0,
                            ) {
                                self.status_msg = Some(format!("Failed to record attempt: {}", e));
                            } else {
                                self.status_msg = Some("Attempt recorded".to_string());
                            }
                        } else {
                            self.status_msg = Some("Failed to open database".to_string());
                        }
                    }
                }
            }
            KeyCode::Char('h') => {
                if let Some(idx) = self.selected_problem {
                    let problems = self.all_problems();
                    if idx < problems.len() {
                        let (pack, problem) = problems[idx];
                        if let Ok(conn) = self.core_store.conn() {
                            if let Err(e) = devcore_challenges::progress::init_challenge_schema(&conn) {
                                self.status_msg = Some(format!("Failed to init schema: {}", e));
                            } else if let Err(e) = devcore_challenges::progress::record_hint(
                                &conn,
                                &problem.id,
                                &pack.id,
                            ) {
                                self.status_msg = Some(format!("Failed to record hint: {}", e));
                            } else {
                                self.status_msg = Some("Hint recorded".to_string());
                            }
                        } else {
                            self.status_msg = Some("Failed to open database".to_string());
                        }
                    }
                }
            }
            _ => {}
        }
    }

    fn draw(&self, frame: &mut Frame) {
        let area = frame.area();

        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(theme::MAUVE))
            .title(Span::styled(
                " DevCore TUI ",
                Style::default().fg(theme::MAUVE).add_modifier(Modifier::BOLD),
            ))
            .style(Style::default().bg(theme::BASE));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(0), Constraint::Length(1)])
            .split(inner);

        let spans: Vec<Span> = Tab::all()
            .iter()
            .map(|tab| {
                let style = if *tab == self.active_tab {
                    Style::default()
                        .fg(theme::YELLOW)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(theme::SUBTEXT)
                };
                Span::styled(
                    format!(" {} {} ", tab.key(), tab.title()),
                    style,
                )
            })
            .collect();
        let tab_bar = Paragraph::new(Line::from(spans)).alignment(Alignment::Center);
        frame.render_widget(tab_bar, chunks[0]);

        match self.active_tab {
            Tab::Dashboard => render_dashboard(frame, chunks[1], self),
            Tab::Academic => render_academic_tab(frame, chunks[1], self),
            Tab::Git => render_git_tab(frame, chunks[1], self),
            Tab::Challenges => render_challenges_tab(frame, chunks[1], self),
        }

        let keybindings: Vec<widgets::KeyBinding> = match self.input_mode {
            InputMode::Normal => match self.active_tab {
                Tab::Dashboard => vec![
                    widgets::KeyBinding { key: "1-4", label: "Tabs", color: theme::MAUVE },
                    widgets::KeyBinding { key: "Tab", label: "Next", color: theme::BLUE },
                    widgets::KeyBinding { key: "q", label: "Quit", color: theme::RED },
                ],
                Tab::Academic => vec![
                    widgets::KeyBinding { key: "c", label: "Add Course", color: theme::GREEN },
                    widgets::KeyBinding { key: "d", label: "Add Deadline", color: theme::YELLOW },
                    widgets::KeyBinding { key: "g", label: "Add Grade", color: theme::BLUE },
                    widgets::KeyBinding { key: "s", label: "Set Semester", color: theme::MAUVE },
                    widgets::KeyBinding { key: "j/k", label: "Nav", color: theme::TEAL },
                    widgets::KeyBinding { key: "x", label: "Complete", color: theme::RED },
                    widgets::KeyBinding { key: "q", label: "Quit", color: theme::RED },
                ],
                Tab::Git => vec![
                    widgets::KeyBinding { key: "x", label: "Add XP", color: theme::GREEN },
                    widgets::KeyBinding { key: "q", label: "Quit", color: theme::RED },
                ],
                Tab::Challenges => vec![
                    widgets::KeyBinding { key: "i", label: "Install", color: theme::GREEN },
                    widgets::KeyBinding { key: "r", label: "Remove", color: theme::RED },
                    widgets::KeyBinding { key: "Enter", label: "View Course", color: theme::BLUE },
                    widgets::KeyBinding { key: "v", label: "Full View", color: theme::TEAL },
                    widgets::KeyBinding { key: "e/m/h/a", label: "Filter", color: theme::YELLOW },
                    widgets::KeyBinding { key: "o", label: "Projects", color: theme::MAUVE },
                    widgets::KeyBinding { key: "c", label: "Complete Stage", color: theme::GREEN },
                    widgets::KeyBinding { key: "n/p", label: "Page", color: theme::TEAL },
                    widgets::KeyBinding { key: "q", label: "Quit", color: theme::RED },
                ],
            },
            InputMode::ViewingCourse => vec![
                widgets::KeyBinding { key: "↑↓", label: "Stages", color: theme::BLUE },
                widgets::KeyBinding { key: "s", label: "Solution", color: theme::GREEN },
                widgets::KeyBinding { key: "c", label: "Complete", color: theme::TEAL },
                widgets::KeyBinding { key: "Esc", label: "Back", color: theme::RED },
            ],
            _ => vec![
                widgets::KeyBinding { key: "Tab", label: "Field", color: theme::BLUE },
                widgets::KeyBinding { key: "Enter", label: "Submit", color: theme::GREEN },
                widgets::KeyBinding { key: "Esc", label: "Cancel", color: theme::RED },
            ],
        };
        widgets::status_bar(frame, chunks[2], &keybindings, self.status_msg.as_deref());

        match self.input_mode {
            InputMode::ConfirmingInstall => {
                if let Some(idx) = self.selected_pack {
                    if idx < self.packs.len() {
                        let pack = &self.packs[idx];
                        widgets::render_confirm_popup(
                            frame,
                            area,
                            &format!("Install pack '{}'?", pack.name),
                            "Press Y to confirm, N/Esc to cancel",
                        );
                    }
                }
            }
            InputMode::ConfirmingRemove => {
                if let Some(idx) = self.selected_pack {
                    if idx < self.packs.len() {
                        let pack = &self.packs[idx];
                        widgets::render_confirm_popup(
                            frame,
                            area,
                            &format!("Remove pack '{}'?", pack.name),
                            "Press Y to confirm, N/Esc to cancel",
                        );
                    }
                }
            }
            InputMode::ViewingDetail => {
                if !self.show_projects {
                    let problems = self.all_problems();
                    if let Some(idx) = self.selected_problem {
                        if idx < problems.len() {
                            let (_pack, problem) = problems[idx];
                            widgets::render_problem_detail(frame, area, problem);
                        }
                    }
                }
            }
            InputMode::ViewingCourse => {
                if let Some(idx) = self.selected_project {
                    if idx < self.projects.len() {
                        let project = &self.projects[idx];
                        let progress = self.project_progress.iter().find(|p| p.project_id == project.id);
                        let cursor = self.course_stage_cursor.min(project.stages.len().saturating_sub(1));
                        render_course_view(frame, area, project, progress, cursor, self.show_solutions);
                    }
                }
            }
            InputMode::AddingXp => {
                let axes: Vec<&str> = SkillAxis::all().iter().map(|a| a.as_str()).collect();
                widgets::render_add_xp_popup(
                    frame,
                    area,
                    &axes,
                    self.xp_axis_index,
                    &self.xp_amount,
                    &self.xp_reason,
                    self.xp_field,
                );
            }
            InputMode::AddingCourse | InputMode::AddingDeadline | InputMode::AddingGrade => {
                let title = match self.input_mode {
                    InputMode::AddingCourse => "Add Course",
                    InputMode::AddingDeadline => "Add Deadline",
                    InputMode::AddingGrade => "Add Grade",
                    _ => unreachable!(),
                };
                let labels: Vec<&str> = self.input_fields.iter().map(|f| f.label).collect();
                let values: Vec<String> = self.input_fields.iter().map(|f| f.value.clone()).collect();
                let status_kind = match self.status_msg.as_deref() {
                    Some(msg) if msg.starts_with("Course added")
                        || msg.starts_with("Deadline added")
                        || msg.starts_with("Grade added") => StatusKind::Success,
                    Some(_) => StatusKind::Error,
                    None => StatusKind::Info,
                };
                widgets::render_input_form(
                    frame,
                    area,
                    title,
                    &labels,
                    &values,
                    self.current_field,
                    self.status_msg.as_deref(),
                    status_kind,
                );
            }
            InputMode::SelectingSemester => {
                let num_sems = self.semesters.len().min(10) as u16;
                let form_height = num_sems + 4;
                let form_width = 50.min(area.width.saturating_sub(4));
                if form_height > area.height || form_width < 20 {
                    return;
                }
                let x = area.x + (area.width.saturating_sub(form_width)) / 2;
                let y = area.y + area.height.saturating_sub(form_height);
                let form_area = Rect::new(x, y, form_width, form_height);
                let mut lines: Vec<Line> = Vec::new();
                for (i, sem) in self.semesters.iter().take(10).enumerate() {
                    let marker = if i == self.semester_cursor { " > " } else { "   " };
                    let style = if i == self.semester_cursor {
                        Style::default().fg(theme::YELLOW).add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(theme::TEXT)
                    };
                    lines.push(Line::from(Span::styled(
                        format!("{}{}", marker, sem.name),
                        style,
                    )));
                }
                lines.push(Line::from(""));
                lines.push(Line::from(Span::styled(
                    "  ↑↓: navigate │ Enter: select │ Esc: cancel",
                    Style::default().fg(theme::SUBTEXT),
                )));
                let block = Block::default()
                    .title(" Select Semester ")
                    .title_style(Style::default().fg(theme::MAUVE).add_modifier(Modifier::BOLD))
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(theme::MAUVE))
                    .style(Style::default().bg(theme::SURFACE));
                let inner_block = block.inner(form_area);
                frame.render_widget(Clear, form_area);
                frame.render_widget(block, form_area);
                frame.render_widget(Paragraph::new(lines), inner_block);
            }
            InputMode::Normal => {}
        }
    }
}
