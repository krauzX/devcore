"use client";

import { useEffect, useState } from "react";
import {
  Card,
  CardContent,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Activity, Clock, Zap } from "lucide-react";
import {
  BarChart,
  Bar,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  ResponsiveContainer,
} from "recharts";
import { api, type ActivityData } from "@/lib/api";

export default function ActivityPage() {
  const [data, setData] = useState<ActivityData | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const controller = new AbortController();
    api.activity(controller.signal)
      .then(setData)
      .catch((err) => {
        if (err.name !== "AbortError") setError(err.message);
      })
      .finally(() => setLoading(false));
    return () => controller.abort();
  }, []);

  if (loading) {
    return (
      <div className="flex items-center justify-center h-96">
        <div className="text-zinc-500">Loading activity...</div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="flex items-center justify-center h-96">
        <div className="text-center">
          <p className="text-rose-400 mb-2">Failed to load activity data</p>
          <p className="text-zinc-500 text-sm">{error}</p>
        </div>
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

  const hours = (data.total_minutes / 60).toFixed(1);
  const categoryData = Object.entries(data.categories).map(([name, count]) => ({
    name,
    count,
  }));

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold">Activity</h1>
          <p className="text-zinc-500 text-sm">Workflow activity from DevPulse</p>
        </div>
        <Badge variant="outline" className="text-amber-400 border-amber-400/30">
          This Week
        </Badge>
      </div>

      {/* Summary Cards */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
        <Card className="bg-zinc-900 border-zinc-800">
          <CardHeader className="flex flex-row items-center justify-between pb-2">
            <CardTitle className="text-sm font-medium text-zinc-400">Total Events</CardTitle>
            <Activity className="w-4 h-4 text-emerald-400" />
          </CardHeader>
          <CardContent>
            <div className="text-3xl font-bold text-emerald-400">{data.total_events}</div>
            <p className="text-xs text-zinc-500 mt-1">events tracked</p>
          </CardContent>
        </Card>

        <Card className="bg-zinc-900 border-zinc-800">
          <CardHeader className="flex flex-row items-center justify-between pb-2">
            <CardTitle className="text-sm font-medium text-zinc-400">Time Tracked</CardTitle>
            <Clock className="w-4 h-4 text-blue-400" />
          </CardHeader>
          <CardContent>
            <div className="text-3xl font-bold text-blue-400">{hours}h</div>
            <p className="text-xs text-zinc-500 mt-1">{data.total_minutes} minutes</p>
          </CardContent>
        </Card>

        <Card className="bg-zinc-900 border-zinc-800">
          <CardHeader className="flex flex-row items-center justify-between pb-2">
            <CardTitle className="text-sm font-medium text-zinc-400">Categories</CardTitle>
            <Zap className="w-4 h-4 text-amber-400" />
          </CardHeader>
          <CardContent>
            <div className="text-3xl font-bold text-amber-400">
              {Object.keys(data.categories).length}
            </div>
            <p className="text-xs text-zinc-500 mt-1">activity types</p>
          </CardContent>
        </Card>
      </div>

      {/* Category Bar Chart */}
      <Card className="bg-zinc-900 border-zinc-800">
        <CardHeader>
          <CardTitle className="text-sm font-medium text-zinc-400">
            Time by Category
          </CardTitle>
        </CardHeader>
        <CardContent>
          {categoryData.length > 0 ? (
            <ResponsiveContainer width="100%" height={250}>
              <BarChart data={categoryData}>
                <CartesianGrid strokeDasharray="3 3" stroke="#27272a" />
                <XAxis dataKey="name" stroke="#71717a" fontSize={12} />
                <YAxis stroke="#71717a" fontSize={12} />
                <Tooltip
                  contentStyle={{ backgroundColor: "#18181b", border: "1px solid #27272a" }}
                />
                <Bar dataKey="count" fill="#f59e0b" radius={[4, 4, 0, 0]} />
              </BarChart>
            </ResponsiveContainer>
          ) : (
            <div className="flex items-center justify-center h-[250px] text-zinc-500">
              No category data yet
            </div>
          )}
        </CardContent>
      </Card>

      {/* Recent Events */}
      <Card className="bg-zinc-900 border-zinc-800">
        <CardHeader>
          <CardTitle className="text-sm font-medium text-zinc-400">
            Recent Events
          </CardTitle>
        </CardHeader>
        <CardContent>
          {data.recent_events.length > 0 ? (
            <div className="space-y-3">
              {data.recent_events.map((event) => {
                const ts = new Date(event.date);
                const timeStr = ts.toLocaleTimeString("en-US", { hour: "2-digit", minute: "2-digit" });
                const cat = (event as Record<string, unknown>).details
                  ? String(((event as Record<string, unknown>).details as Record<string, unknown>)?.category ?? event.type)
                  : event.type;
                return (
                  <div
                    key={event.id}
                    className="flex items-center justify-between p-3 rounded-lg bg-zinc-800/50"
                  >
                    <div>
                      <p className="font-medium text-sm">{cat}</p>
                      <p className="text-xs text-zinc-500">{timeStr}</p>
                    </div>
                    <Badge variant="outline" className="text-xs">
                      {event.type}
                    </Badge>
                  </div>
                );
              })}
            </div>
          ) : (
            <p className="text-zinc-500 text-sm">No recent events</p>
          )}
        </CardContent>
      </Card>
    </div>
  );
}
