// Tsubasa (翼) — Speed Graph Component
// Real-time bandwidth graph using Canvas 2D.
// Shows download (accent) and upload (green) lines.
// Integrates with the speed graph IPC when connected.

import { useRef, useEffect, useCallback, useState } from "react";
import type { SpeedSample } from "@/types";
import { getSpeedGraph, getTorrentSpeedGraph } from "@/lib/tauri";

interface SpeedGraphProps {
    /** Optional specific torrent ID */
    torrentId?: string;
    width?: number;
    height?: number;
    /** How many seconds of history to show (default 60) */
    windowSecs?: number;
    /** Polling interval ms (default 1000) */
    pollInterval?: number;
    /** CSS class name */
    className?: string;
}

function formatSpeed(bytesPerSec: number): string {
    if (bytesPerSec === 0) return "0 B/s";
    if (bytesPerSec < 1024) return `${bytesPerSec.toFixed(0)} B/s`;
    if (bytesPerSec < 1024 * 1024) return `${(bytesPerSec / 1024).toFixed(1)} KB/s`;
    if (bytesPerSec < 1024 * 1024 * 1024) return `${(bytesPerSec / (1024 * 1024)).toFixed(1)} MB/s`;
    return `${(bytesPerSec / (1024 * 1024 * 1024)).toFixed(2)} GB/s`;
}

export function SpeedGraph({
    torrentId,
    width = 320,
    height = 100,
    windowSecs = 60,
    pollInterval = 1000,
    className,
}: SpeedGraphProps) {
    const canvasRef = useRef<HTMLCanvasElement>(null);
    const [samples, setSamples] = useState<SpeedSample[]>([]);
    const [currentDl, setCurrentDl] = useState(0);
    const [currentUl, setCurrentUl] = useState(0);

    // Poll for data
    useEffect(() => {
        let active = true;
        const poll = async () => {
            try {
                const data = torrentId
                    ? await getTorrentSpeedGraph(torrentId, windowSecs)
                    : await getSpeedGraph(windowSecs);
                if (active && data.length > 0) {
                    setSamples(data);
                    const last = data[data.length - 1];
                    setCurrentDl(last.download_speed);
                    setCurrentUl(last.upload_speed);
                }
            } catch { /* ignore - graph will show empty */ }
        };

        poll();
        const id = setInterval(poll, pollInterval);
        return () => { active = false; clearInterval(id); };
    }, [windowSecs, pollInterval, torrentId]);

    // Draw graph
    const draw = useCallback(() => {
        const canvas = canvasRef.current;
        if (!canvas) return;
        const ctx = canvas.getContext("2d");
        if (!ctx) return;

        const dpr = window.devicePixelRatio || 1;
        canvas.width = width * dpr;
        canvas.height = height * dpr;
        ctx.scale(dpr, dpr);

        // Clear
        ctx.clearRect(0, 0, width, height);

        // Background
        ctx.fillStyle = "rgba(0, 0, 0, 0.15)";
        ctx.beginPath();
        ctx.roundRect(0, 0, width, height, 8);
        ctx.fill();

        if (samples.length < 2) {
            // No data — draw placeholder grid
            ctx.strokeStyle = "rgba(255, 255, 255, 0.04)";
            ctx.lineWidth = 1;
            for (let i = 1; i < 4; i++) {
                const y = (height / 4) * i;
                ctx.beginPath();
                ctx.moveTo(0, y);
                ctx.lineTo(width, y);
                ctx.stroke();
            }
            ctx.fillStyle = "rgba(255, 255, 255, 0.15)";
            ctx.font = "10px system-ui";
            ctx.textAlign = "center";
            ctx.fillText("No speed data yet", width / 2, height / 2 + 3);
            return;
        }

        // Find max speed for scaling
        const maxSpeed = Math.max(
            ...samples.map((s) => Math.max(s.download_speed, s.upload_speed)),
            1024 // Minimum 1 KB/s scale
        );
        const scaleY = (v: number) => height - 8 - (v / maxSpeed) * (height - 16);
        const scaleX = (i: number) => (i / (samples.length - 1)) * width;

        // Grid lines
        ctx.strokeStyle = "rgba(255, 255, 255, 0.04)";
        ctx.lineWidth = 1;
        for (let i = 1; i < 4; i++) {
            const y = (height / 4) * i;
            ctx.beginPath();
            ctx.moveTo(0, y);
            ctx.lineTo(width, y);
            ctx.stroke();
        }

        // Draw download line (accent/indigo)
        drawLine(ctx, samples, scaleX, scaleY, "download_speed", "rgba(99, 102, 241, 0.9)", "rgba(99, 102, 241, 0.08)", height);

        // Draw upload line (green)
        drawLine(ctx, samples, scaleX, scaleY, "upload_speed", "rgba(34, 197, 94, 0.9)", "rgba(34, 197, 94, 0.06)", height);

        // Scale labels
        ctx.fillStyle = "rgba(255, 255, 255, 0.3)";
        ctx.font = "9px 'JetBrains Mono', monospace";
        ctx.textAlign = "right";
        ctx.fillText(formatSpeed(maxSpeed), width - 6, 14);
        ctx.fillText(formatSpeed(maxSpeed / 2), width - 6, height / 2 + 3);
    }, [samples, width, height]);

    useEffect(() => {
        draw();
    }, [draw]);

    return (
        <div className={className} style={{ position: "relative" }}>
            <canvas
                ref={canvasRef}
                style={{
                    width,
                    height,
                    borderRadius: 8,
                    display: "block",
                }}
            />
            {/* Current speed overlay */}
            <div style={{
                position: "absolute", top: 6, left: 8,
                display: "flex", gap: 12, fontSize: 10, fontFamily: "'JetBrains Mono', monospace",
            }}>
                <span style={{ color: "rgba(99, 102, 241, 0.9)" }}>↓ {formatSpeed(currentDl)}</span>
                <span style={{ color: "rgba(34, 197, 94, 0.9)" }}>↑ {formatSpeed(currentUl)}</span>
            </div>
        </div>
    );
}

function drawLine(
    ctx: CanvasRenderingContext2D,
    samples: SpeedSample[],
    scaleX: (i: number) => number,
    scaleY: (v: number) => number,
    key: "download_speed" | "upload_speed",
    lineColor: string,
    fillColor: string,
    height: number,
) {
    if (samples.length < 2) return;

    // Fill area
    ctx.beginPath();
    ctx.moveTo(scaleX(0), height);
    for (let i = 0; i < samples.length; i++) {
        ctx.lineTo(scaleX(i), scaleY(samples[i][key]));
    }
    ctx.lineTo(scaleX(samples.length - 1), height);
    ctx.closePath();
    ctx.fillStyle = fillColor;
    ctx.fill();

    // Line
    ctx.beginPath();
    ctx.moveTo(scaleX(0), scaleY(samples[0][key]));
    for (let i = 1; i < samples.length; i++) {
        // Smooth with bezier curves
        const prevX = scaleX(i - 1);
        const prevY = scaleY(samples[i - 1][key]);
        const currX = scaleX(i);
        const currY = scaleY(samples[i][key]);
        const cpX = (prevX + currX) / 2;
        ctx.bezierCurveTo(cpX, prevY, cpX, currY, currX, currY);
    }
    ctx.strokeStyle = lineColor;
    ctx.lineWidth = 1.5;
    ctx.stroke();
}
