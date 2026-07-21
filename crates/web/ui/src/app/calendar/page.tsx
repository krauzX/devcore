"use client";

import { useEffect, useState } from "react";
import {
  Card,
  CardContent,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { api, type AcademicEvent } from "@/lib/api";
import { Calendar, Clock, AlertTriangle, BookOpen } from "lucide-react";

const eventTypeConfig: Record<string, { icon: React.ReactNode; color: string }> = {
  exam: { icon: <AlertTriangle className="w-4 h-4" />, color: "text-rose-400" },
  assignment: { icon: <BookOpen className="w-4 h-4" />, color: "text-amber-400" },
  lab: { icon: <BookOpen className="w-4 h-4" />, color: "text-blue-400" },
  lecture: { icon: <Clock className="w-4 h-4" />, color: "text-emerald-400" },
  holiday: { icon: <Calendar className="w-4 h-4" />, color: "text-purple-400" },
  submission: { icon: <AlertTriangle className="w-4 h-4" />, color: "text-rose-400" },
  presentation: { icon: <BookOpen className="w-4 h-4" />, color: "text-cyan-400" },
  other: { icon: <Calendar className="w-4 h-4" />, color: "text-zinc-400" },
};

export default function CalendarPage() {
  const [events, setEvents] = useState<AcademicEvent[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    api.upcoming()
      .then(setEvents)
      .catch(console.error)
      .finally(() => setLoading(false));
  }, []);

  if (loading) {
    return <div className="flex items-center justify-center h-96 text-zinc-500">Loading...</div>;
  }

  // Group events by date
  const grouped = events.reduce((acc, event) => {
    const date = event.date;
    if (!acc[date]) acc[date] = [];
    acc[date].push(event);
    return acc;
  }, {} as Record<string, AcademicEvent[]>);

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-2xl font-bold">Academic Calendar</h1>
        <p className="text-zinc-500 text-sm">Upcoming events and deadlines</p>
      </div>

      {Object.keys(grouped).length === 0 ? (
        <Card className="bg-zinc-900 border-zinc-800">
          <CardContent className="p-8 text-center">
            <Calendar className="w-12 h-12 mx-auto mb-4 text-zinc-600" />
            <p className="text-zinc-400">No upcoming events this week</p>
            <p className="text-xs text-zinc-600 mt-2">
              Add events via CLI:{" "}
              <code className="bg-zinc-800 px-1 rounded">devcore event add</code>
            </p>
          </CardContent>
        </Card>
      ) : (
        Object.entries(grouped).map(([date, dayEvents]) => (
          <Card key={date} className="bg-zinc-900 border-zinc-800">
            <CardHeader className="pb-3">
              <CardTitle className="text-sm font-medium text-zinc-400">
                {new Date(date + "T00:00:00").toLocaleDateString("en-US", {
                  weekday: "long",
                  month: "long",
                  day: "numeric",
                })}
              </CardTitle>
            </CardHeader>
            <CardContent>
              <div className="space-y-3">
                {dayEvents.map((event) => {
                  const config = eventTypeConfig[event.event_type] || eventTypeConfig.other;
                  return (
                    <div
                      key={event.id}
                      className="flex items-center justify-between p-3 rounded-lg bg-zinc-800/50"
                    >
                      <div className="flex items-center gap-3">
                        <span className={config.color}>{config.icon}</span>
                        <div>
                          <p className="font-medium text-sm">{event.title}</p>
                          {event.time && (
                            <p className="text-xs text-zinc-500">at {event.time}</p>
                          )}
                          {event.notes && (
                            <p className="text-xs text-zinc-600 mt-1">{event.notes}</p>
                          )}
                        </div>
                      </div>
                      <Badge variant="outline" className="text-xs">
                        {event.event_type}
                      </Badge>
                    </div>
                  );
                })}
              </div>
            </CardContent>
          </Card>
        ))
      )}
    </div>
  );
}
