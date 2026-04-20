// lib/index.js
// @ememzyvisuals/microdragon — Node.js API for programmatic use
// Use MICRODRAGON from your own Node.js projects

const { spawn, spawnSync } = require("child_process");
const path = require("path");
const fs = require("fs");
const http = require("http");

/**
 * MICRODRAGON Node.js API
 * 
 * @example
 * const microdragon = require("@ememzyvisuals/microdragon");
 * const response = await microdragon.ask("Write a Python function to sort a list");
 * console.log(response);
 */

// ─── HTTP API client (connects to running MICRODRAGON daemon) ────────────────────────

class MicrodragonClient {
  constructor(options = {}) {
    this.host = options.host || "127.0.0.1";
    this.port = options.port || 7700;
    this.timeout = options.timeout || 60000;
  }

  /**
   * Ask MICRODRAGON a question or give it a task
   * @param {string} input - Your prompt or task
   * @param {object} options - { agent, session_id, context }
   * @returns {Promise<string>} AI response
   */
  async ask(input, options = {}) {
    const body = JSON.stringify({
      input,
      context: options.context || [],
      source: "nodejs_api",
      session_id: options.session_id,
      agent: options.agent,
    });

    return new Promise((resolve, reject) => {
      const req = http.request({
        hostname: this.host,
        port: this.port,
        path: "/api/chat",
        method: "POST",
        headers: {
          "Content-Type": "application/json",
          "Content-Length": Buffer.byteLength(body),
          "X-MICRODRAGON-Source": "nodejs_api",
        },
      }, (res) => {
        let data = "";
        res.on("data", (chunk) => (data += chunk));
        res.on("end", () => {
          try {
            const json = JSON.parse(data);
            resolve(json.response || json.error || "No response");
          } catch {
            resolve(data);
          }
        });
      });

      req.on("error", reject);
      req.setTimeout(this.timeout, () => { req.destroy(); reject(new Error("Timeout")); });
      req.write(body);
      req.end();
    });
  }

  /**
   * Get MICRODRAGON system status
   * @returns {Promise<object>} Status object
   */
  async status() {
    return new Promise((resolve, reject) => {
      http.get(`http://${this.host}:${this.port}/api/status`, (res) => {
        let data = "";
        res.on("data", (c) => (data += c));
        res.on("end", () => { try { resolve(JSON.parse(data)); } catch { resolve({}); } });
      }).on("error", reject);
    });
  }

  /**
   * Submit feedback for a task
   * @param {string} taskId
   * @param {"good"|"bad"|"neutral"} score
   */
  async feedback(taskId, score) {
    const body = JSON.stringify({ task_id: taskId, score });
    return new Promise((resolve, reject) => {
      const req = http.request({
        hostname: this.host, port: this.port,
        path: "/api/feedback", method: "POST",
        headers: { "Content-Type": "application/json", "Content-Length": Buffer.byteLength(body) },
      }, (res) => {
        let data = "";
        res.on("data", (c) => (data += c));
        res.on("end", () => { try { resolve(JSON.parse(data)); } catch { resolve({}); } });
      });
      req.on("error", reject);
      req.write(body);
      req.end();
    });
  }

  /** Check if MICRODRAGON daemon is running */
  async isRunning() {
    try {
      await this.status();
      return true;
    } catch {
      return false;
    }
  }
}

// ─── Process-based API (no daemon needed) ─────────────────────────────────────

class MicrodragonCLI {
  constructor() {
    this.binary = findBinary();
  }

  /**
   * Run a one-shot MICRODRAGON command
   * @param {string} input
   * @returns {string} Response
   */
  ask(input) {
    if (!this.binary) throw new Error("MICRODRAGON binary not found. Run: npx @ememzyvisuals/microdragon --install");
    const result = spawnSync(this.binary, ["ask", "--output", "json", input], {
      encoding: "utf-8",
      timeout: 60000,
    });
    if (result.error) throw result.error;
    try {
      return JSON.parse(result.stdout).response || result.stdout;
    } catch {
      return result.stdout || result.stderr;
    }
  }

  /** Start MICRODRAGON daemon */
  start() {
    if (!this.binary) throw new Error("MICRODRAGON binary not found");
    return spawn(this.binary, ["--daemon"], { detached: true, stdio: "ignore" });
  }
}

// ─── Convenience exports ──────────────────────────────────────────────────────

function findBinary() {
  const candidates = [
    path.join(__dirname, "..", "bin", process.platform === "win32" ? "microdragon.exe" : "microdragon"),
    path.join(__dirname, "..", "core", "target", "release",
               process.platform === "win32" ? "microdragon.exe" : "microdragon"),
  ];
  return candidates.find(c => fs.existsSync(c)) || null;
}

const defaultClient = new MicrodragonClient();

module.exports = {
  // Main client (HTTP API)
  MicrodragonClient,
  // CLI wrapper (process-based)
  MicrodragonCLI,
  // Default instance shortcuts
  ask: (input, opts) => defaultClient.ask(input, opts),
  status: () => defaultClient.status(),
  feedback: (id, score) => defaultClient.feedback(id, score),
  isRunning: () => defaultClient.isRunning(),
  // Create custom client
  createClient: (opts) => new MicrodragonClient(opts),
  // Metadata
  version: require("../package.json").version,
  author: "Emmanuel Ariyo / EMEMZYVISUALS DIGITALS",
};
