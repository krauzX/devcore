const API_BASE = process.env.NEXT_PUBLIC_API_URL || "http://localhost:3847";

export interface Semester {
  id: string;
  number: number;
  name: string;
  start_date: string;
  end_date: string;
  is_current: boolean;
}

export interface Course {
  id: string;
  semester_id: string;
  code: string;
  name: string;
  credits: number;
  course_type: string;
  instructor: string | null;
}

export interface Paper {
  id: string;
  title: string;
  authors: string | null;
  venue: string | null;
  year: number | null;
  doi: string | null;
  arxiv_id: string | null;
  status: string;
  tags: string | null;
  notes: string | null;
  added_at: string;
}

export interface PaperStats {
  total: number;
  to_read: number;
  reading: number;
  read: number;
  cited: number;
}

export interface AcademicEvent {
  id: string;
  title: string;
  event_type: string;
  date: string;
  time: string | null;
  course_id: string | null;
  notes: string | null;
}

export interface DashboardData {
  semester: Semester | null;
  courses: Course[];
  paper_stats: PaperStats;
  upcoming_events: AcademicEvent[];
  sgpa: number | null;
}

async function fetchApi<T>(endpoint: string, signal?: AbortSignal): Promise<T> {
  const res = await fetch(`${API_BASE}${endpoint}`, { signal });
  if (!res.ok) throw new Error(`API error: ${res.status}`);
  return res.json();
}

export const api = {
  semesters: () => fetchApi<Semester[]>("/api/semesters"),
  currentSemester: () => fetchApi<Semester>("/api/semesters/current"),
  courses: (semesterId: string) =>
    fetchApi<Course[]>(`/api/semester/${semesterId}/courses`),
  sgpa: (semesterId: string) =>
    fetchApi<{ semester: string; sgpa: number | null }>(
      `/api/semester/${semesterId}/sgpa`
    ),
  papers: () => fetchApi<Paper[]>("/api/papers"),
  paperStats: () => fetchApi<PaperStats>("/api/papers/stats"),
  upcoming: () => fetchApi<AcademicEvent[]>("/api/upcoming"),
  dashboard: () => fetchApi<DashboardData>("/api/dashboard"),
};
