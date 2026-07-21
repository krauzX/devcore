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
  GraduationCap,
  BookOpen,
  FlaskConical,
  Calendar,
  TrendingUp,
  Clock,
} from "lucide-react";
import {
  BarChart,
  Bar,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  ResponsiveContainer,
  PieChart,
  Pie,
  Cell,
} from "recharts";
import { api, type DashboardData } from "@/lib/api";

export default function DashboardPage() {
  const [data, setData] = useState<DashboardData | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    api.dashboard().then(setData).catch(console.error).finally(() => setLoading(false));
  }, []);

  if (loading) {
    return (
      <div className="flex items-center justify-center h-96">
        <div className="text-zinc-500">Loading dashboard...</div>
      </div>
    );
  }

  if (!data) {
    return (
      <div className="flex items-center justify-center h-96">
        <div className="text-center">
          <p className="text-zinc-400 mb-4">API server not running</p>
          <code className="text-sm bg-zinc-800 px-3 py-1 rounded">
            cargo run -p devcore-api
          </code>
        </div>
      </div>
    );
  }

  const gradeData = data.courses.map((c) => ({
    name: c.code,
    credits: c.credits,
    type: c.course_type,
  }));

  const paperPieData = [
    { name: "To Read", value: data.paper_stats.to_read, color: "#f59e0b" },
    { name: "Reading", value: data.paper_stats.reading, color: "#3b82f6" },
    { name: "Read", value: data.paper_stats.read, color: "#10b981" },
    { name: "Cited", value: data.paper_stats.cited, color: "#8b5cf6" },
  ].filter((d) => d.value > 0);

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold">Dashboard</h1>
          <p className="text-zinc-500 text-sm">
            {data.semester?.name || "No semester"} — {data.semester?.id || ""}
          </p>
        </div>
        <Badge variant="outline" className="text-emerald-400 border-emerald-400/30">
          {new Date().toLocaleDateString("en-US", { weekday: "long", month: "long", day: "numeric" })}
        </Badge>
      </div>

      {/* Stats Cards */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
        <Card className="bg-zinc-900 border-zinc-800">
          <CardHeader className="flex flex-row items-center justify-between pb-2">
            <CardTitle className="text-sm font-medium text-zinc-400">SGPA</CardTitle>
            <GraduationCap className="w-4 h-4 text-emerald-400" />
          </CardHeader>
          <CardContent>
            <div className="text-3xl font-bold text-emerald-400">
              {data.sgpa?.toFixed(2) || "N/A"}
            </div>
            <p className="text-xs text-zinc-500 mt-1">
              {data.courses.length} courses enrolled
            </p>
          </CardContent>
        </Card>

        <Card className="bg-zinc-900 border-zinc-800">
          <CardHeader className="flex flex-row items-center justify-between pb-2">
            <CardTitle className="text-sm font-medium text-zinc-400">Credits</CardTitle>
            <BookOpen className="w-4 h-4 text-blue-400" />
          </CardHeader>
          <CardContent>
            <div className="text-3xl font-bold text-blue-400">
              {data.courses.reduce((sum, c) => sum + c.credits, 0)}
            </div>
            <p className="text-xs text-zinc-500 mt-1">
              {data.courses.filter((c) => c.course_type === "lab").length} labs
            </p>
          </CardContent>
        </Card>

        <Card className="bg-zinc-900 border-zinc-800">
          <CardHeader className="flex flex-row items-center justify-between pb-2">
            <CardTitle className="text-sm font-medium text-zinc-400">Papers</CardTitle>
            <FlaskConical className="w-4 h-4 text-amber-400" />
          </CardHeader>
          <CardContent>
            <div className="text-3xl font-bold text-amber-400">
              {data.paper_stats.total}
            </div>
            <p className="text-xs text-zinc-500 mt-1">
              {data.paper_stats.to_read} to read
            </p>
          </CardContent>
        </Card>

        <Card className="bg-zinc-900 border-zinc-800">
          <CardHeader className="flex flex-row items-center justify-between pb-2">
            <CardTitle className="text-sm font-medium text-zinc-400">Upcoming</CardTitle>
            <Calendar className="w-4 h-4 text-rose-400" />
          </CardHeader>
          <CardContent>
            <div className="text-3xl font-bold text-rose-400">
              {data.upcoming_events.length}
            </div>
            <p className="text-xs text-zinc-500 mt-1">events this week</p>
          </CardContent>
        </Card>
      </div>

      {/* Charts Row */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-4">
        <Card className="bg-zinc-900 border-zinc-800">
          <CardHeader>
            <CardTitle className="text-sm font-medium text-zinc-400">
              Course Credits
            </CardTitle>
          </CardHeader>
          <CardContent>
            <ResponsiveContainer width="100%" height={200}>
              <BarChart data={gradeData}>
                <CartesianGrid strokeDasharray="3 3" stroke="#27272a" />
                <XAxis dataKey="name" stroke="#71717a" fontSize={12} />
                <YAxis stroke="#71717a" fontSize={12} />
                <Tooltip
                  contentStyle={{ backgroundColor: "#18181b", border: "1px solid #27272a" }}
                />
                <Bar dataKey="credits" fill="#10b981" radius={[4, 4, 0, 0]} />
              </BarChart>
            </ResponsiveContainer>
          </CardContent>
        </Card>

        <Card className="bg-zinc-900 border-zinc-800">
          <CardHeader>
            <CardTitle className="text-sm font-medium text-zinc-400">
              Paper Reading Status
            </CardTitle>
          </CardHeader>
          <CardContent>
            {paperPieData.length > 0 ? (
              <ResponsiveContainer width="100%" height={200}>
                <PieChart>
                  <Pie
                    data={paperPieData}
                    cx="50%"
                    cy="50%"
                    innerRadius={50}
                    outerRadius={80}
                    paddingAngle={5}
                    dataKey="value"
                  >
                    {paperPieData.map((entry, index) => (
                      <Cell key={`cell-${index}`} fill={entry.color} />
                    ))}
                  </Pie>
                  <Tooltip
                    contentStyle={{ backgroundColor: "#18181b", border: "1px solid #27272a" }}
                  />
                </PieChart>
              </ResponsiveContainer>
            ) : (
              <div className="flex items-center justify-center h-[200px] text-zinc-500">
                No papers tracked yet
              </div>
            )}
          </CardContent>
        </Card>
      </div>

      {/* Upcoming Events */}
      <Card className="bg-zinc-900 border-zinc-800">
        <CardHeader>
          <CardTitle className="text-sm font-medium text-zinc-400">
            Upcoming Events
          </CardTitle>
        </CardHeader>
        <CardContent>
          {data.upcoming_events.length > 0 ? (
            <div className="space-y-3">
              {data.upcoming_events.map((event) => (
                <div
                  key={event.id}
                  className="flex items-center justify-between p-3 rounded-lg bg-zinc-800/50"
                >
                  <div>
                    <p className="font-medium text-sm">{event.title}</p>
                    <p className="text-xs text-zinc-500">
                      {event.date} {event.time && `at ${event.time}`}
                    </p>
                  </div>
                  <Badge variant="outline" className="text-xs">
                    {event.event_type}
                  </Badge>
                </div>
              ))}
            </div>
          ) : (
            <p className="text-zinc-500 text-sm">No upcoming events</p>
          )}
        </CardContent>
      </Card>
    </div>
  );
}
