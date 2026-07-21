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
import { api, type Paper, type PaperStats } from "@/lib/api";
import { BookOpen, Clock, CheckCircle, Quote, ExternalLink } from "lucide-react";

const statusColors: Record<string, string> = {
  to_read: "bg-amber-500/20 text-amber-400 border-amber-500/30",
  reading: "bg-blue-500/20 text-blue-400 border-blue-500/30",
  read: "bg-emerald-500/20 text-emerald-400 border-emerald-500/30",
  cited: "bg-purple-500/20 text-purple-400 border-purple-500/30",
  archived: "bg-zinc-500/20 text-zinc-400 border-zinc-500/30",
};

export default function ResearchPage() {
  const [papers, setPapers] = useState<Paper[]>([]);
  const [stats, setStats] = useState<PaperStats | null>(null);
  const [filter, setFilter] = useState<string>("all");
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const controller = new AbortController();
    const { signal } = controller;
    Promise.all([api.papers(signal), api.paperStats(signal)])
      .then(([p, s]) => {
        setPapers(p);
        setStats(s);
      })
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
          <p className="text-rose-400 mb-2">Failed to load research data</p>
          <p className="text-zinc-500 text-sm">{error}</p>
        </div>
      </div>
    );
  }

  const filteredPapers = filter === "all" ? papers : papers.filter((p) => p.status === filter);

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-2xl font-bold">Research Tracker</h1>
        <p className="text-zinc-500 text-sm">Track papers, citations, and reading progress</p>
      </div>

      {/* Stats Cards */}
      {stats && (
        <div className="grid grid-cols-2 md:grid-cols-5 gap-3">
          <Card className="bg-zinc-900 border-zinc-800">
            <CardContent className="p-4 text-center">
              <BookOpen className="w-5 h-5 mx-auto mb-1 text-zinc-400" />
              <p className="text-2xl font-bold">{stats.total}</p>
              <p className="text-xs text-zinc-500">Total</p>
            </CardContent>
          </Card>
          <Card className="bg-zinc-900 border-zinc-800">
            <CardContent className="p-4 text-center">
              <Clock className="w-5 h-5 mx-auto mb-1 text-amber-400" />
              <p className="text-2xl font-bold text-amber-400">{stats.to_read}</p>
              <p className="text-xs text-zinc-500">To Read</p>
            </CardContent>
          </Card>
          <Card className="bg-zinc-900 border-zinc-800">
            <CardContent className="p-4 text-center">
              <BookOpen className="w-5 h-5 mx-auto mb-1 text-blue-400" />
              <p className="text-2xl font-bold text-blue-400">{stats.reading}</p>
              <p className="text-xs text-zinc-500">Reading</p>
            </CardContent>
          </Card>
          <Card className="bg-zinc-900 border-zinc-800">
            <CardContent className="p-4 text-center">
              <CheckCircle className="w-5 h-5 mx-auto mb-1 text-emerald-400" />
              <p className="text-2xl font-bold text-emerald-400">{stats.read}</p>
              <p className="text-xs text-zinc-500">Read</p>
            </CardContent>
          </Card>
          <Card className="bg-zinc-900 border-zinc-800">
            <CardContent className="p-4 text-center">
              <Quote className="w-5 h-5 mx-auto mb-1 text-purple-400" />
              <p className="text-2xl font-bold text-purple-400">{stats.cited}</p>
              <p className="text-xs text-zinc-500">Cited</p>
            </CardContent>
          </Card>
        </div>
      )}

      {/* Filter Tabs */}
      <div className="flex gap-2 flex-wrap">
        {["all", "to_read", "reading", "read", "cited"].map((status) => (
          <button
            key={status}
            onClick={() => setFilter(status)}
            className={`px-3 py-1 rounded-full text-xs font-medium transition-colors ${
              filter === status
                ? "bg-emerald-500/20 text-emerald-400"
                : "bg-zinc-800 text-zinc-400 hover:bg-zinc-700"
            }`}
          >
            {status.replace("_", " ")}
          </button>
        ))}
      </div>

      {/* Papers Table */}
      <Card className="bg-zinc-900 border-zinc-800">
        <CardContent className="p-0">
          <Table>
            <TableHeader>
              <TableRow className="border-zinc-800">
                <TableHead className="text-zinc-400">Title</TableHead>
                <TableHead className="text-zinc-400">Authors</TableHead>
                <TableHead className="text-zinc-400">Venue</TableHead>
                <TableHead className="text-zinc-400">Year</TableHead>
                <TableHead className="text-zinc-400">Status</TableHead>
                <TableHead className="text-zinc-400">Link</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {filteredPapers.length === 0 ? (
                <TableRow>
                  <TableCell colSpan={6} className="text-center text-zinc-500 py-8">
                    No papers found. Add papers via the CLI:{" "}
                    <code className="bg-zinc-800 px-1 rounded">devcore paper add</code>
                  </TableCell>
                </TableRow>
              ) : (
                filteredPapers.map((paper) => (
                  <TableRow key={paper.id} className="border-zinc-800">
                    <TableCell className="font-medium max-w-[300px] truncate">
                      {paper.title}
                    </TableCell>
                    <TableCell className="text-zinc-500 text-sm max-w-[200px] truncate">
                      {paper.authors || "—"}
                    </TableCell>
                    <TableCell className="text-zinc-500 text-sm">
                      {paper.venue || "—"}
                    </TableCell>
                    <TableCell className="text-zinc-500 text-sm">
                      {paper.year || "—"}
                    </TableCell>
                    <TableCell>
                      <Badge variant="outline" className={statusColors[paper.status] || ""}>
                        {paper.status.replace("_", " ")}
                      </Badge>
                    </TableCell>
                    <TableCell>
                      {paper.arxiv_id && (
                        <a
                          href={`https://arxiv.org/abs/${paper.arxiv_id}`}
                          target="_blank"
                          rel="noopener noreferrer"
                          className="text-zinc-400 hover:text-emerald-400"
                        >
                          <ExternalLink className="w-4 h-4" />
                        </a>
                      )}
                      {paper.doi && !paper.arxiv_id && (
                        <a
                          href={`https://doi.org/${paper.doi}`}
                          target="_blank"
                          rel="noopener noreferrer"
                          className="text-zinc-400 hover:text-emerald-400"
                        >
                          <ExternalLink className="w-4 h-4" />
                        </a>
                      )}
                    </TableCell>
                  </TableRow>
                ))
              )}
            </TableBody>
          </Table>
        </CardContent>
      </Card>
    </div>
  );
}
