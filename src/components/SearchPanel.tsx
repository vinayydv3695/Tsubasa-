// Tsubasa — Search Panel Component
// Full-featured torrent search UI with results table, history, and "Add" actions.
// Opens as a modal overlay triggered from the toolbar.

import React, { useEffect, useRef, useCallback } from "react";
import {
  Search,
  X,
  Download,
  Clock,
  Trash2,
  Loader2,
  AlertCircle,
  CheckCircle,
  ArrowUpDown,
  HardDrive,
  Users,
} from "lucide-react";
import { useSearchStore } from "@/stores/search";
import { useTorrentStore } from "@/stores/torrent";
import { formatBytes } from "@/lib/utils";
import type { SearchResult } from "@/types";
import { motion } from "framer-motion";

interface SearchPanelProps {
  onClose: () => void;
}

export function SearchPanel({ onClose }: SearchPanelProps) {
  const inputRef = useRef<HTMLInputElement>(null);

  const query = useSearchStore((s) => s.query);
  const results = useSearchStore((s) => s.results);
  const loading = useSearchStore((s) => s.loading);
  const error = useSearchStore((s) => s.error);
  const history = useSearchStore((s) => s.history);
  const historyLoaded = useSearchStore((s) => s.historyLoaded);
  const checkCache = useSearchStore((s) => s.checkCache);
  const setQuery = useSearchStore((s) => s.setQuery);
  const setCheckCache = useSearchStore((s) => s.setCheckCache);
  const search = useSearchStore((s) => s.search);
  const loadHistory = useSearchStore((s) => s.loadHistory);
  const clearHistory = useSearchStore((s) => s.clearHistory);
  const clearResults = useSearchStore((s) => s.clearResults);

  const addTorrent = useTorrentStore((s) => s.addTorrent);

  // Load history on mount + focus input
  useEffect(() => {
    if (!historyLoaded) loadHistory();
    inputRef.current?.focus();
  }, [historyLoaded, loadHistory]);

  // Close on Escape
  useEffect(() => {
    const handleKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") onClose();
    };
    window.addEventListener("keydown", handleKey);
    return () => window.removeEventListener("keydown", handleKey);
  }, [onClose]);

  const handleSearch = useCallback(() => {
    search();
  }, [search]);

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Enter") handleSearch();
  };

  const handleHistoryClick = (q: string) => {
    setQuery(q);
    search(q);
  };

  const handleAddTorrent = async (result: SearchResult) => {
    try {
      await addTorrent(result.magnet_uri);
    } catch (err) {
      console.error("Failed to add torrent from search:", err);
    }
  };

  const showHistory = !loading && results.length === 0 && !error && history.length > 0;

  return (
    <motion.div
      className="fixed inset-0 z-50 flex items-start justify-center pt-[10vh] bg-black/60 backdrop-blur-sm"
      onClick={(e) => {
        if (e.target === e.currentTarget) onClose();
      }}
      initial={{ opacity: 0 }}
      animate={{ opacity: 1 }}
      exit={{ opacity: 0 }}
      transition={{ duration: 0.15 }}
    >
      <motion.div
        className="w-[720px] max-h-[75vh] bg-surface border border-line rounded-xl shadow-lg overflow-hidden flex flex-col"
        initial={{ opacity: 0, y: -12, scale: 0.98 }}
        animate={{ opacity: 1, y: 0, scale: 1 }}
        exit={{ opacity: 0, y: -12, scale: 0.98 }}
        transition={{ duration: 0.2, ease: "easeOut" }}
      >
        {/* Search Input Header */}
        <div className="flex items-center gap-3 px-4 py-3 border-b border-line">
          <Search size={16} className="text-fg-3 flex-shrink-0" />
          <input
            ref={inputRef}
            type="text"
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            onKeyDown={handleKeyDown}
            placeholder="Search torrents..."
            className="flex-1 bg-transparent text-sm text-fg placeholder:text-fg-3 focus:outline-none"
          />
          {loading && <Loader2 size={14} className="text-accent animate-spin flex-shrink-0" />}

          {/* Cache check toggle */}
          <button
            onClick={() => setCheckCache(!checkCache)}
            className={`flex items-center gap-1.5 px-2.5 py-1 rounded-md text-xs transition-colors-fast flex-shrink-0 ${
              checkCache
                ? "bg-accent-soft text-accent"
                : "bg-overlay text-fg-3 hover:text-fg-2"
            }`}
            title="Check if results are cached on Torbox"
          >
            <HardDrive size={12} />
            Cache
          </button>

          <button
            onClick={handleSearch}
            disabled={!query.trim() || loading}
            className="px-3 py-1 rounded-md bg-accent hover:bg-accent-hover text-white text-xs font-medium transition-colors-fast disabled:opacity-30 flex-shrink-0"
          >
            Search
          </button>

          <button
            onClick={onClose}
            className="p-1 rounded-md hover:bg-muted text-fg-3 hover:text-fg-2 transition-colors-fast flex-shrink-0"
          >
            <X size={14} />
          </button>
        </div>

        {/* Content Area */}
        <div className="flex-1 overflow-y-auto min-h-0">
          {/* Error State */}
          {error && (
            <div className="flex items-center gap-2 px-4 py-3 text-xs text-red bg-red-soft">
              <AlertCircle size={14} />
              <span>{error}</span>
            </div>
          )}

          {/* Search History */}
          {showHistory && (
            <div className="px-4 py-3">
              <div className="flex items-center justify-between mb-2">
                <div className="flex items-center gap-1.5 text-xs text-fg-3">
                  <Clock size={12} />
                  Recent Searches
                </div>
                <button
                  onClick={clearHistory}
                  className="flex items-center gap-1 text-xs text-fg-3 hover:text-red transition-colors-fast"
                >
                  <Trash2 size={11} />
                  Clear
                </button>
              </div>
              <div className="flex flex-wrap gap-1.5">
                {history.slice(0, 20).map((entry) => (
                  <button
                    key={entry.id}
                    onClick={() => handleHistoryClick(entry.query)}
                    className="px-2.5 py-1 rounded-md bg-overlay hover:bg-muted text-xs text-fg-2 hover:text-fg transition-colors-fast"
                  >
                    {entry.query}
                  </button>
                ))}
              </div>
            </div>
          )}

          {/* Empty State */}
          {!loading && results.length === 0 && !error && !showHistory && query.trim() && (
            <div className="flex flex-col items-center justify-center py-12 text-fg-3">
              <Search size={24} className="mb-2 opacity-40" />
              <p className="text-xs">Press Enter or click Search to find torrents</p>
            </div>
          )}

          {/* No Results */}
          {!loading && results.length === 0 && !error && query.trim() !== "" && results !== null && (
            <></>
          )}

          {/* Results Table */}
          {results.length > 0 && (
            <div className="overflow-x-auto">
              <table className="w-full text-xs">
                <thead>
                  <tr className="text-left text-fg-3 border-b border-line sticky top-0 bg-surface">
                    <th className="px-4 py-2 font-medium">Name</th>
                    <th className="px-3 py-2 font-medium w-20">
                      <div className="flex items-center gap-1">
                        <HardDrive size={11} />
                        Size
                      </div>
                    </th>
                    <th className="px-3 py-2 font-medium w-16">
                      <div className="flex items-center gap-1">
                        <ArrowUpDown size={11} />
                        S/L
                      </div>
                    </th>
                    <th className="px-3 py-2 font-medium w-20">Source</th>
                    {checkCache && <th className="px-3 py-2 font-medium w-16">Cached</th>}
                    <th className="px-3 py-2 font-medium w-16"></th>
                  </tr>
                </thead>
                <tbody>
                  {results.map((result, idx) => (
                    <SearchResultRow
                      key={`${result.info_hash}-${idx}`}
                      result={result}
                      showCache={checkCache}
                      onAdd={handleAddTorrent}
                    />
                  ))}
                </tbody>
              </table>

              {/* Results footer */}
              <div className="px-4 py-2 text-xs text-fg-3 border-t border-line">
                {results.length} result{results.length !== 1 ? "s" : ""} found
                {results.length > 0 && (
                  <button
                    onClick={clearResults}
                    className="ml-3 text-fg-3 hover:text-fg-2 transition-colors-fast"
                  >
                    Clear
                  </button>
                )}
              </div>
            </div>
          )}

          {/* Loading shimmer */}
          {loading && results.length === 0 && (
            <div className="px-4 py-6 space-y-3">
              {Array.from({ length: 6 }).map((_, i) => (
                <div key={i} className="flex items-center gap-3">
                  <div className="h-3 rounded bg-overlay animate-pulse flex-1" />
                  <div className="h-3 w-16 rounded bg-overlay animate-pulse" />
                  <div className="h-3 w-12 rounded bg-overlay animate-pulse" />
                </div>
              ))}
            </div>
          )}
        </div>
      </motion.div>
    </motion.div>
  );
}

// ─── Individual Result Row ───────────────────────────────

interface SearchResultRowProps {
  result: SearchResult;
  showCache: boolean;
  onAdd: (result: SearchResult) => void;
}

function SearchResultRow({ result, showCache, onAdd }: SearchResultRowProps) {
  const [adding, setAdding] = React.useState(false);
  const [added, setAdded] = React.useState(false);

  const handleAdd = async () => {
    if (adding || added) return;
    setAdding(true);
    try {
      await onAdd(result);
      setAdded(true);
    } catch {
      // Error handled by parent
    } finally {
      setAdding(false);
    }
  };

  // Health color based on seeders
  const healthColor =
    result.seeders >= 50
      ? "text-green"
      : result.seeders >= 10
        ? "text-amber"
        : result.seeders >= 1
          ? "text-red"
          : "text-fg-3";

  return (
    <tr className="border-b border-line-subtle hover:bg-muted/50 transition-colors-fast group">
      {/* Name */}
      <td className="px-4 py-2">
        <div className="truncate max-w-[340px] text-fg group-hover:text-accent-hover transition-colors-fast" title={result.name}>
          {result.name}
        </div>
        <div className="text-2xs text-fg-3 mt-0.5 truncate max-w-[340px]" title={result.info_hash}>
          {result.category}
        </div>
      </td>

      {/* Size */}
      <td className="px-3 py-2 text-fg-2 tabular-nums font-mono">
        {result.size > 0 ? formatBytes(result.size) : "--"}
      </td>

      {/* Seeders / Leechers */}
      <td className="px-3 py-2 tabular-nums font-mono">
        <div className="flex items-center gap-1">
          <Users size={10} className={healthColor} />
          <span className={healthColor}>{result.seeders}</span>
          <span className="text-fg-3">/</span>
          <span className="text-fg-3">{result.leechers}</span>
        </div>
      </td>

      {/* Source */}
      <td className="px-3 py-2 text-fg-3 truncate max-w-[100px]" title={result.source}>
        {result.source}
      </td>

      {/* Cached indicator */}
      {showCache && (
        <td className="px-3 py-2">
          {result.cached === true ? (
            <span className="flex items-center gap-1 text-green">
              <CheckCircle size={12} />
              Yes
            </span>
          ) : result.cached === false ? (
            <span className="text-fg-3">No</span>
          ) : (
            <span className="text-fg-3">--</span>
          )}
        </td>
      )}

      {/* Add button */}
      <td className="px-3 py-2">
        <button
          onClick={handleAdd}
          disabled={adding || added}
          className={`flex items-center gap-1 px-2 py-1 rounded text-xs transition-colors-fast ${
            added
              ? "bg-green-soft text-green cursor-default"
              : "bg-accent-soft text-accent hover:bg-accent hover:text-white"
          } disabled:opacity-50`}
        >
          {adding ? (
            <Loader2 size={11} className="animate-spin" />
          ) : added ? (
            <CheckCircle size={11} />
          ) : (
            <Download size={11} />
          )}
          {added ? "Added" : "Add"}
        </button>
      </td>
    </tr>
  );
}
