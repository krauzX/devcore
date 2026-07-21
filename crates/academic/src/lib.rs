/// Assignment tracking and lifecycle management.
pub mod assignment;
/// Academic calendar with events and deadlines.
pub mod calendar;
/// Course catalog and credit management.
pub mod course;
/// Grade entry, SGPA/CGPA computation, and grade-point conversion.
pub mod grade;
/// Research paper reading-list management.
pub mod research;
/// Semester and institution configuration store.
pub mod semester;

pub use assignment::Assignment;
pub use calendar::AcademicEvent;
pub use course::Course;
pub use grade::GradeEntry;
pub use research::Paper;
pub use semester::SemesterStore;
