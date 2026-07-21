"use client";

import { useEffect, useState } from "react";
import {
  Card,
  CardContent,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
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
  LineChart,
  Line,
} from "recharts";
import { api, type DashboardData } from "@/lib/api";
import { TrendingUp, Target, Zap, Clock } from "lucide-react";

const COLORS = ["#10b981", "#3b82f6", "#f59e0b", "#ef4444", "#8b5cf6", "#06b6d4"];

export default function AnalyticsPage() {
  const [data, setData] = useState<DashboardData | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const controller = new AbortController();
    api.dashboard(controller.signal)
      .then(setData)
      .catch((err) => {
        if (err.name !== "AbortError") setError(err.message);
      })
      .finally(() => setLoading(false));
    return () => controller.abort();
  }, []);

  if (loading) {
    return <div className="flex items-center justify-center h-96 text-zinc-500">Loading...</div>;
  }

  if (error) {
    return (
      <div className="flex items-center justify-center h-96">
        <div className="text-center">
          <p className="text-rose-400 mb-2">Failed to load analytics data</p>
          <p className="text-zinc-500 text-sm">{error}</p>
        </div>
      </div>
    );
  }

  if (!data) {
    return (
      <div className="flex items-center justify-center h-96">
        <p className="text-zinc-400">API server not running</p>
      </div>
    );
  }

  const courseTypeData = [
    { name: "Theory", value: data.courses.filter((c) => c.course_type === "theory").length },
    { name: "Lab", value: data.courses.filter((c) => c.course_type === "lab").length },
  ];

  const paperStatusData = [
    { name: "To Read", value: data.paper_stats.to_read },
    { name: "Reading", value: data.paper_stats.reading },
    { name: "Read", value: data.paper_stats.read },
    { name: "Cited", value: data.paper_stats.cited },
  ].filter((d) => d.value > 0);

  const creditDistribution = data.courses.map((c) => ({
    name: c.code,
    credits: c.credits,
  }));

  // Simulated weekly activity data
  const weeklyActivity = Array.from({ length: 7 }, (_, i) => ({
    day: ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"][i],
    hours: Math.random() * 8 + 1,
    commits: Math.floor(Math.random() * 15),
  }));

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-2xl font-bold">Analytics</h1>
        <p className="text-zinc-500 text-sm">Insights into your academic and dev productivity</p>
      </div>

      {/* Key Metrics */}
      <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
        <Card className="bg-zinc-900 border-zinc-800">
          <CardContent className="p-4">
            <div className="flex items-center gap-2 mb-2">
              <TrendingUp className="w-4 h-4 text-emerald-400" />
              <span className="text-xs text-zinc-500">SGPA</span>
            </div>
            <p className="text-2xl font-bold text-emerald-400">
              {data.sgpa?.toFixed(2) || "N/A"}
            </p>
          </CardContent>
        </Card>

        <Card className="bg-zinc-900 border-zinc-800">
          <CardContent className="p-4">
            <div className="flex items-center gap-2 mb-2">
              <Target className="w-4 h-4 text-blue-400" />
              <span className="text-xs text-zinc-500">Papers Read</span>
            </div>
            <p className="text-2xl font-bold text-blue-400">{data.paper_stats.read}</p>
          </CardContent>
        </Card>

        <Card className="bg-zinc-900 border-zinc-800">
          <CardContent className="p-4">
            <div className="flex items-center gap-2 mb-2">
              <Zap className="w-4 h-4 text-amber-400" />
              <span className="text-xs text-zinc-500">Completion</span>
            </div>
            <p className="text-2xl font-bold text-amber-400">
              {data.paper_stats.total > 0
                ? Math.round((data.paper_stats.read / data.paper_stats.total) * 100)
                : 0}
              %
            </p>
          </CardContent>
        </Card>

        <Card className="bg-zinc-900 border-zinc-800">
          <CardContent className="p-4">
            <div className="flex items-center gap-2 mb-2">
              <Clock className="w-4 h-4 text-rose-400" />
              <span className="text-xs text-zinc-500">To Read</span>
            </div>
            <p className="text-2xl font-bold text-rose-400">{data.paper_stats.to_read}</p>
          </CardContent>
        </Card>
      </div>

      {/* Charts */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-4">
        <Card className="bg-zinc-900 border-zinc-800">
          <CardHeader>
            <CardTitle className="text-sm font-medium text-zinc-400">Credit Distribution</CardTitle>
          </CardHeader>
          <CardContent>
            <ResponsiveContainer width="100%" height={250}>
              <BarChart data={creditDistribution}>
                <CartesianGrid strokeDasharray="3 3" stroke="#27272a" />
                <XAxis dataKey="name" stroke="#71717a" fontSize={11} />
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
            <CardTitle className="text-sm font-medium text-zinc-400">Paper Reading Progress</CardTitle>
          </CardHeader>
          <CardContent>
            {paperStatusData.length > 0 ? (
              <ResponsiveContainer width="100%" height={250}>
                <PieChart>
                  <Pie
                    data={paperStatusData}
                    cx="50%"
                    cy="50%"
                    innerRadius={60}
                    outerRadius={90}
                    paddingAngle={5}
                    dataKey="value"
                    label={({ name, value }) => `${name}: ${value}`}
                  >
                    {paperStatusData.map((entry, index) => (
                      <Cell key={`cell-${index}`} fill={COLORS[index % COLORS.length]} />
                    ))}
                  </Pie>
                  <Tooltip
                    contentStyle={{ backgroundColor: "#18181b", border: "1px solid #27272a" }}
                  />
                </PieChart>
              </ResponsiveContainer>
            ) : (
              <div className="flex items-center justify-center h-[250px] text-zinc-500">
                No paper data yet
              </div>
            )}
          </CardContent>
        </Card>
      </div>

      {/* Weekly Activity */}
      <Card className="bg-zinc-900 border-zinc-800">
        <CardHeader>
          <CardTitle className="text-sm font-medium text-zinc-400">Weekly Activity (Simulated)</CardTitle>
        </CardHeader>
        <CardContent>
          <ResponsiveContainer width="100%" height={200}>
            <LineChart data={weeklyActivity}>
              <CartesianGrid strokeDasharray="3 3" stroke="#27272a" />
              <XAxis dataKey="day" stroke="#71717a" fontSize={12} />
              <YAxis stroke="#71717a" fontSize={12} />
              <Tooltip
                contentStyle={{ backgroundColor: "#18181b", border: "1px solid #27272a" }}
              />
              <Line type="monotone" dataKey="hours" stroke="#10b981" strokeWidth={2} dot={{ fill: "#10b981" }} />
              <Line type="monotone" dataKey="commits" stroke="#3b82f6" strokeWidth={2} dot={{ fill: "#3b82f6" }} />
            </LineChart>
          </ResponsiveContainer>
        </CardContent>
      </Card>
    </div>
  );
}
