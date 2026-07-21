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
import { api, type Course, type Semester } from "@/lib/api";
import { BookOpen, FlaskConical, Users } from "lucide-react";

export default function CoursesPage() {
  const [courses, setCourses] = useState<Course[]>([]);
  const [semester, setSemester] = useState<Semester | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    api.currentSemester()
      .then((sem) => {
        setSemester(sem);
        if (!sem) return Promise.resolve([]);
        return api.courses(sem.id);
      })
      .then(setCourses)
      .catch(console.error)
      .finally(() => setLoading(false));
  }, []);

  if (loading) {
    return <div className="flex items-center justify-center h-96 text-zinc-500">Loading...</div>;
  }

  const theoryCourses = courses.filter((c) => c.course_type === "theory");
  const labCourses = courses.filter((c) => c.course_type === "lab");
  const totalCredits = courses.reduce((sum, c) => sum + c.credits, 0);

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-2xl font-bold">Courses</h1>
        <p className="text-zinc-500 text-sm">
          {semester?.name || "Current Semester"} — {totalCredits} total credits
        </p>
      </div>

      {/* Summary Cards */}
      <div className="grid grid-cols-3 gap-4">
        <Card className="bg-zinc-900 border-zinc-800">
          <CardContent className="p-4 text-center">
            <BookOpen className="w-5 h-5 mx-auto mb-1 text-blue-400" />
            <p className="text-2xl font-bold text-blue-400">{theoryCourses.length}</p>
            <p className="text-xs text-zinc-500">Theory Courses</p>
          </CardContent>
        </Card>
        <Card className="bg-zinc-900 border-zinc-800">
          <CardContent className="p-4 text-center">
            <FlaskConical className="w-5 h-5 mx-auto mb-1 text-emerald-400" />
            <p className="text-2xl font-bold text-emerald-400">{labCourses.length}</p>
            <p className="text-xs text-zinc-500">Lab Courses</p>
          </CardContent>
        </Card>
        <Card className="bg-zinc-900 border-zinc-800">
          <CardContent className="p-4 text-center">
            <Users className="w-5 h-5 mx-auto mb-1 text-amber-400" />
            <p className="text-2xl font-bold text-amber-400">{totalCredits}</p>
            <p className="text-xs text-zinc-500">Total Credits</p>
          </CardContent>
        </Card>
      </div>

      {/* Theory Courses */}
      <Card className="bg-zinc-900 border-zinc-800">
        <CardHeader>
          <CardTitle className="text-sm font-medium text-zinc-400">Theory Courses</CardTitle>
        </CardHeader>
        <CardContent>
          <Table>
            <TableHeader>
              <TableRow className="border-zinc-800">
                <TableHead className="text-zinc-400">Code</TableHead>
                <TableHead className="text-zinc-400">Course Name</TableHead>
                <TableHead className="text-zinc-400">Credits</TableHead>
                <TableHead className="text-zinc-400">Instructor</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {theoryCourses.map((course) => (
                <TableRow key={course.id} className="border-zinc-800">
                  <TableCell className="font-mono text-emerald-400">{course.code}</TableCell>
                  <TableCell>{course.name}</TableCell>
                  <TableCell>
                    <Badge variant="outline">{course.credits}</Badge>
                  </TableCell>
                  <TableCell className="text-zinc-500">{course.instructor || "—"}</TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        </CardContent>
      </Card>

      {/* Lab Courses */}
      <Card className="bg-zinc-900 border-zinc-800">
        <CardHeader>
          <CardTitle className="text-sm font-medium text-zinc-400">Lab Courses</CardTitle>
        </CardHeader>
        <CardContent>
          <Table>
            <TableHeader>
              <TableRow className="border-zinc-800">
                <TableHead className="text-zinc-400">Code</TableHead>
                <TableHead className="text-zinc-400">Course Name</TableHead>
                <TableHead className="text-zinc-400">Credits</TableHead>
                <TableHead className="text-zinc-400">Instructor</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {labCourses.map((course) => (
                <TableRow key={course.id} className="border-zinc-800">
                  <TableCell className="font-mono text-emerald-400">{course.code}</TableCell>
                  <TableCell>{course.name}</TableCell>
                  <TableCell>
                    <Badge variant="outline" className="text-blue-400 border-blue-400/30">
                      {course.credits}
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
