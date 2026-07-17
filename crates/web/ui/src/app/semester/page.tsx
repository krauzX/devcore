"use client";

import { useEffect, useState } from "react";
import {
  Card,
  CardContent,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import { api, type Semester, type Course } from "@/lib/api";

export default function SemesterPage() {
  const [semesters, setSemesters] = useState<Semester[]>([]);
  const [current, setCurrent] = useState<Semester | null>(null);
  const [courses, setCourses] = useState<Course[]>([]);
  const [sgpa, setSgpa] = useState<number | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    Promise.all([
      api.semesters(),
      api.currentSemester(),
    ])
      .then(([sems, curr]) => {
        setSemesters(sems);
        setCurrent(curr);
        if (curr) {
          return Promise.all([api.courses(curr.id), api.sgpa(curr.id)]);
        }
        return Promise.resolve([[] as Course[], null] as [Course[], { semester: string; sgpa: number | null } | null]);
      })
      .then(([courses, sgpaData]) => {
        setCourses(courses);
        setSgpa(sgpaData?.sgpa ?? null);
      })
      .catch(console.error)
      .finally(() => setLoading(false));
  }, []);

  if (loading) {
    return <div className="flex items-center justify-center h-96 text-zinc-500">Loading...</div>;
  }

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-2xl font-bold">Semester Management</h1>
        <p className="text-zinc-500 text-sm">IIIT Kottayam — B.Tech CSE</p>
      </div>

      {/* Semester Grid */}
      <div className="grid grid-cols-2 md:grid-cols-4 gap-3">
        {semesters.map((sem) => (
          <Card
            key={sem.id}
            className={`bg-zinc-900 border-zinc-800 cursor-pointer transition-colors ${
              sem.is_current ? "border-emerald-500/50 bg-emerald-500/5" : ""
            }`}
          >
            <CardContent className="p-4 text-center">
              <p className="text-xs text-zinc-500">Semester {sem.number}</p>
              <p className="font-bold text-lg">{sem.name}</p>
              {sem.is_current && (
                <Badge className="mt-2 bg-emerald-500/20 text-emerald-400 border-emerald-500/30">
                  Current
                </Badge>
              )}
            </CardContent>
          </Card>
        ))}
      </div>

      {/* SGPA Card */}
      <Card className="bg-zinc-900 border-zinc-800">
        <CardHeader>
          <CardTitle className="text-sm font-medium text-zinc-400">
            Current Semester — {current?.name}
          </CardTitle>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-2 md:grid-cols-4 gap-4 mb-6">
            <div>
              <p className="text-xs text-zinc-500">SGPA</p>
              <p className="text-2xl font-bold text-emerald-400">
                {sgpa?.toFixed(2) || "N/A"}
              </p>
            </div>
            <div>
              <p className="text-xs text-zinc-500">Total Credits</p>
              <p className="text-2xl font-bold text-blue-400">
                {courses.reduce((sum, c) => sum + c.credits, 0)}
              </p>
            </div>
            <div>
              <p className="text-xs text-zinc-500">Courses</p>
              <p className="text-2xl font-bold text-zinc-200">{courses.length}</p>
            </div>
            <div>
              <p className="text-xs text-zinc-500">Period</p>
              <p className="text-sm text-zinc-300">
                {current?.start_date} — {current?.end_date}
              </p>
            </div>
          </div>

          {/* Course Table */}
          <Table>
            <TableHeader>
              <TableRow className="border-zinc-800">
                <TableHead className="text-zinc-400">Code</TableHead>
                <TableHead className="text-zinc-400">Course</TableHead>
                <TableHead className="text-zinc-400">Credits</TableHead>
                <TableHead className="text-zinc-400">Type</TableHead>
                <TableHead className="text-zinc-400">Instructor</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {courses.map((course) => (
                <TableRow key={course.id} className="border-zinc-800">
                  <TableCell className="font-mono text-emerald-400">{course.code}</TableCell>
                  <TableCell>{course.name}</TableCell>
                  <TableCell>{course.credits}</TableCell>
                  <TableCell>
                    <Badge variant="outline" className={
                      course.course_type === "lab" ? "text-blue-400 border-blue-400/30" : "text-zinc-400"
                    }>
                      {course.course_type}
                    </Badge>
                  </TableCell>
                  <TableCell className="text-zinc-500">{course.instructor || "—"}</TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        </CardContent>
      </Card>
    </div>
  );
}
