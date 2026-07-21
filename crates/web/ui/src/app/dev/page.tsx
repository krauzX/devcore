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
  GitBranch,
  BarChart3,
  Terminal,
  FileSearch,
  Server,
} from "lucide-react";
import { api, type SystemInfo } from "@/lib/api";

const tools = [
  {
    name: "ShipForge",
    description: "Generate structured change receipts for AI-generated code",
    icon: <GitBranch className="w-5 h-5" />,
    color: "text-emerald-400",
    commands: ["receipt", "log", "explain", "blast", "show"],
    category: "Code Quality",
  },
  {
    name: "CodeTrail",
    description: "Query change receipts, find hotspots, blast radius analysis",
    icon: <FileSearch className="w-5 h-5" />,
    color: "text-blue-400",
    commands: ["history", "explain", "blast", "ai-log", "risk", "hotspots"],
    category: "Analysis",
  },
  {
    name: "DevPulse",
    description: "Developer workflow analyzer with TUI dashboard and weekly reports",
    icon: <BarChart3 className="w-5 h-5" />,
    color: "text-amber-400",
    commands: ["dashboard", "weekly", "report", "event", "suggest", "chart"],
    category: "Productivity",
  },
  {
    name: "DevCore API",
    description: "REST API server exposing all tools as HTTP endpoints",
    icon: <Terminal className="w-5 h-5" />,
    color: "text-purple-400",
    commands: ["health", "semesters", "courses", "papers", "dashboard"],
    category: "API",
  },
];

export default function DevToolsPage() {
  const [systemInfo, setSystemInfo] = useState<SystemInfo | null>(null);

  useEffect(() => {
    api.systemInfo().then(setSystemInfo).catch(() => {});
  }, []);

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold">Developer Tools</h1>
          <p className="text-zinc-500 text-sm">CLI tools for code quality, analysis, and productivity</p>
        </div>
        {systemInfo && (
          <Badge variant="outline" className="text-emerald-400 border-emerald-400/30">
            v{systemInfo.version}
          </Badge>
        )}
      </div>

      {/* System Info */}
      {systemInfo && (
        <Card className="bg-zinc-900 border-zinc-800">
          <CardHeader className="flex flex-row items-center justify-between pb-2">
            <CardTitle className="text-sm font-medium text-zinc-400">System Info</CardTitle>
            <Server className="w-4 h-4 text-zinc-400" />
          </CardHeader>
          <CardContent>
            <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
              <div>
                <p className="text-xs text-zinc-500">Version</p>
                <p className="text-lg font-bold text-emerald-400">{systemInfo.version}</p>
              </div>
              <div>
                <p className="text-xs text-zinc-500">Crates</p>
                <p className="text-lg font-bold text-blue-400">{systemInfo.crate_count}</p>
              </div>
              <div>
                <p className="text-xs text-zinc-500">Tests</p>
                <p className="text-lg font-bold text-amber-400">{systemInfo.test_count}</p>
              </div>
              <div>
                <p className="text-xs text-zinc-500">Languages</p>
                <p className="text-lg font-bold text-purple-400">{systemInfo.languages.join(", ")}</p>
              </div>
            </div>
          </CardContent>
        </Card>
      )}

      {/* Tool Cards */}
      <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
        {tools.map((tool) => (
          <Card key={tool.name} className="bg-zinc-900 border-zinc-800">
            <CardHeader>
              <div className="flex items-center justify-between">
                <div className="flex items-center gap-3">
                  <span className={tool.color}>{tool.icon}</span>
                  <CardTitle className="text-lg">{tool.name}</CardTitle>
                </div>
                <Badge variant="outline" className="text-xs">
                  {tool.category}
                </Badge>
              </div>
            </CardHeader>
            <CardContent>
              <p className="text-sm text-zinc-400 mb-4">{tool.description}</p>
              <div className="space-y-2">
                <p className="text-xs text-zinc-500 font-medium">Available Commands:</p>
                <div className="flex flex-wrap gap-2">
                  {tool.commands.map((cmd) => (
                    <code
                      key={cmd}
                      className="text-xs bg-zinc-800 px-2 py-1 rounded text-zinc-300"
                    >
                      {tool.name.toLowerCase()} {cmd}
                    </code>
                  ))}
                </div>
              </div>
            </CardContent>
          </Card>
        ))}
      </div>

      {/* Quick Start */}
      <Card className="bg-zinc-900 border-zinc-800">
        <CardHeader>
          <CardTitle className="text-sm font-medium text-zinc-400">Quick Start</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="space-y-3">
            <div className="flex items-start gap-3">
              <span className="text-emerald-400 font-mono text-sm">1</span>
              <div>
                <p className="text-sm font-medium">Initialize DevCore</p>
                <code className="text-xs bg-zinc-800 px-2 py-1 rounded block mt-1">
                  shipforge init
                </code>
              </div>
            </div>
            <div className="flex items-start gap-3">
              <span className="text-emerald-400 font-mono text-sm">2</span>
              <div>
                <p className="text-sm font-medium">Generate a receipt</p>
                <code className="text-xs bg-zinc-800 px-2 py-1 rounded block mt-1">
                  shipforge receipt
                </code>
              </div>
            </div>
            <div className="flex items-start gap-3">
              <span className="text-emerald-400 font-mono text-sm">3</span>
              <div>
                <p className="text-sm font-medium">Launch TUI dashboard</p>
                <code className="text-xs bg-zinc-800 px-2 py-1 rounded block mt-1">
                  devpulse dashboard
                </code>
              </div>
            </div>
            <div className="flex items-start gap-3">
              <span className="text-emerald-400 font-mono text-sm">4</span>
              <div>
                <p className="text-sm font-medium">View weekly report</p>
                <code className="text-xs bg-zinc-800 px-2 py-1 rounded block mt-1">
                  devpulse weekly
                </code>
              </div>
            </div>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
