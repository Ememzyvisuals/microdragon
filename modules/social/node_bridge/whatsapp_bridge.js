// microdragon/modules/social/node_bridge/whatsapp_bridge.js
//
// MICRODRAGON WhatsApp Bridge
// ─────────────────────
// Uses whatsapp-web.js to connect WhatsApp to the MICRODRAGON engine.
// Sends commands to Rust core via HTTP, receives responses.
// Context memory per-user. Message queue with rate limiting.

const { Client, LocalAuth, MessageMedia } = require("whatsapp-web.js");
const qrcode = require("qrcode-terminal");
const http = require("http");
const https = require("https");
const fs = require("fs");
const path = require("path");

// ─── Config ──────────────────────────────────────────────────────────────────

const CONFIG = {
  microdragonCoreUrl: process.env.MICRODRAGON_CORE_URL || "http://127.0.0.1:7700",
  commandPrefix: process.env.MICRODRAGON_CMD_PREFIX || "/microdragon",
  allowAll: process.env.MICRODRAGON_ALLOW_ALL === "true",
  allowedNumbers: (process.env.MICRODRAGON_ALLOWED_NUMBERS || "").split(",").filter(Boolean),
  rateLimitMs: parseInt(process.env.MICRODRAGON_RATE_LIMIT_MS || "2000"),
  contextWindow: parseInt(process.env.MICRODRAGON_CONTEXT_WINDOW || "20"),
  humanTypingDelay: parseInt(process.env.MICRODRAGON_TYPING_DELAY_MS || "600"),
  sessionDir: path.join(process.env.HOME || ".", ".microdragon", "whatsapp_session"),
  logFile: path.join(process.env.HOME || ".", ".microdragon", "logs", "whatsapp.log"),
};

// ─── State ────────────────────────────────────────────────────────────────────

const contextStore = new Map(); // jid → [{role, content}]
const rateLimiter = new Map();  // jid → last message timestamp
const msgQueue = [];
let processing = false;

// ─── Logging ─────────────────────────────────────────────────────────────────

function log(level, msg) {
  const line = `[${new Date().toISOString()}] [${level.toUpperCase()}] ${msg}`;
  console.log(line);
  try {
    fs.mkdirSync(path.dirname(CONFIG.logFile), { recursive: true });
    fs.appendFileSync(CONFIG.logFile, line + "\n");
  } catch (_) {}
}

// ─── Rate limiter ─────────────────────────────────────────────────────────────

function isRateLimited(jid) {
  const last = rateLimiter.get(jid);
  const now = Date.now();
  if (last && now - last < CONFIG.rateLimitMs) return true;
  rateLimiter.set(jid, now);
  return false;
}

// ─── Context memory (per user) ───────────────────────────────────────────────

function getContext(jid) {
  if (!contextStore.has(jid)) contextStore.set(jid, []);
  return contextStore.get(jid);
}

function addToContext(jid, role, content) {
  const ctx = getContext(jid);
  ctx.push({ role, content });
  // Trim to window
  while (ctx.length > CONFIG.contextWindow * 2) ctx.shift();
}

function clearContext(jid) {
  contextStore.set(jid, []);
}

// ─── Microdragon Core API call ───────────────────────────────────────────────────────

async function callMicrodragonCore(input, context) {
  return new Promise((resolve, reject) => {
    const body = JSON.stringify({
      input,
      context,
      source: "whatsapp",
    });

    const url = new URL(CONFIG.microdragonCoreUrl + "/api/chat");
    const options = {
      hostname: url.hostname,
      port: url.port || (url.protocol === "https:" ? 443 : 80),
      path: url.pathname,
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        "Content-Length": Buffer.byteLength(body),
        "X-MICRODRAGON-Source": "whatsapp",
      },
    };

    const transport = url.protocol === "https:" ? https : http;
    const req = transport.request(options, (res) => {
      let data = "";
      res.on("data", (chunk) => (data += chunk));
      res.on("end", () => {
        try {
          const json = JSON.parse(data);
          resolve(json.response || "No response");
        } catch {
          resolve(data || "Error parsing response");
        }
      });
    });

    req.on("error", reject);
    req.setTimeout(60000, () => {
      req.destroy();
      reject(new Error("MICRODRAGON core timeout"));
    });
    req.write(body);
    req.end();
  });
}

// ─── Message processor ───────────────────────────────────────────────────────

async function processMessage(client, msg, body) {
  const jid = msg.from;
  const sender = msg.author || jid;

  // Add user message to context
  addToContext(jid, "user", body);
  const context = getContext(jid).slice(-CONFIG.contextWindow * 2);

  // Simulate human typing
  await client.sendPresenceAvailable();
  await sleep(CONFIG.humanTypingDelay);
  const chat = await msg.getChat();
  await chat.sendStateTyping();

  let response;
  try {
    response = await callMicrodragonCore(body, context);
  } catch (err) {
    log("error", `Core API call failed: ${err.message}`);
    response = "⚠ MICRODRAGON is currently unavailable. Please try again.";
  }

  await chat.clearState();
  await client.sendMessage(jid, response);

  // Add response to context
  addToContext(jid, "assistant", response);
  log("info", `[${jid}] Q: ${body.substring(0, 80)} | A: ${response.substring(0, 80)}`);
}

// ─── Message queue processor ─────────────────────────────────────────────────

async function processQueue(client) {
  if (processing || msgQueue.length === 0) return;
  processing = true;
  const item = msgQueue.shift();
  try {
    await processMessage(client, item.msg, item.body);
  } catch (err) {
    log("error", `Queue processing error: ${err.message}`);
  } finally {
    processing = false;
    if (msgQueue.length > 0) setImmediate(() => processQueue(client));
  }
}

// ─── Built-in commands ────────────────────────────────────────────────────────

function isBuiltinCommand(body) {
  const b = body.toLowerCase().trim();
  return b === "/microdragon help" || b === "/microdragon status" || b === "/microdragon clear"
    || b === "/microdragon reset" || b.startsWith("/microdragon ");
}

async function handleBuiltinCommand(client, msg, body) {
  const b = body.toLowerCase().trim();
  const jid = msg.from;

  if (b === "/microdragon help") {
    await client.sendMessage(jid, `*MICRODRAGON Commands*\n\n` +
      `/microdragon help — show this message\n` +
      `/microdragon status — check MICRODRAGON status\n` +
      `/microdragon clear — clear your conversation history\n` +
      `/microdragon reset — reset to defaults\n\n` +
      `Any other message (no prefix needed) will be sent to MICRODRAGON AI.`);
    return;
  }

  if (b === "/microdragon clear") {
    clearContext(jid);
    await client.sendMessage(jid, "✓ Conversation history cleared.");
    return;
  }

  if (b === "/microdragon status") {
    const ctxLen = getContext(jid).length;
    await client.sendMessage(jid, `*MICRODRAGON Status*\n• Core: online\n• Your context: ${ctxLen} messages`);
    return;
  }

  // All other /microdragon commands → forward to core
  const command = body.replace(/^\/microdragon\s*/i, "").trim();
  if (command) {
    msgQueue.push({ msg, body: command });
    processQueue(client);
  }
}

// ─── Access control ───────────────────────────────────────────────────────────

function isAuthorized(jid) {
  if (CONFIG.allowAll) return true;
  if (CONFIG.allowedNumbers.length === 0) return true; // no restriction configured
  const number = jid.replace("@c.us", "").replace("@g.us", "");
  return CONFIG.allowedNumbers.some((n) => n.replace(/\D/g, "") === number.replace(/\D/g, ""));
}

// ─── WhatsApp client setup ───────────────────────────────────────────────────

function sleep(ms) {
  return new Promise((r) => setTimeout(r, ms));
}

const client = new Client({
  authStrategy: new LocalAuth({ dataPath: CONFIG.sessionDir }),
  puppeteer: {
    headless: true,
    args: [
      "--no-sandbox",
      "--disable-setuid-sandbox",
      "--disable-dev-shm-usage",
      "--disable-accelerated-2d-canvas",
      "--no-first-run",
      "--disable-gpu",
    ],
  },
});

client.on("qr", (qr) => {
  log("info", "Scan this QR code with WhatsApp:");
  qrcode.generate(qr, { small: true });
  console.log("\nOpen WhatsApp → Settings → Linked Devices → Link a Device\n");
});

client.on("ready", () => {
  log("info", "✓ WhatsApp bridge connected and ready");
  console.log("✓ MICRODRAGON WhatsApp bridge is live!");
});

client.on("auth_failure", (msg) => {
  log("error", `Auth failed: ${msg}`);
});

client.on("disconnected", (reason) => {
  log("warn", `Disconnected: ${reason}. Reconnecting...`);
  setTimeout(() => client.initialize(), 5000);
});

client.on("message", async (msg) => {
  // Ignore status broadcasts and own messages
  if (msg.fromMe || msg.isStatus) return;

  const jid = msg.from;
  const body = msg.body?.trim() || "";

  if (!body) return;
  if (!isAuthorized(jid)) {
    log("warn", `Unauthorized message from ${jid}`);
    return;
  }
  if (isRateLimited(jid)) {
    log("debug", `Rate limited: ${jid}`);
    return;
  }

  log("info", `Message from ${jid}: ${body.substring(0, 100)}`);

  if (isBuiltinCommand(body)) {
    await handleBuiltinCommand(client, msg, body);
    return;
  }

  // All non-command messages → MICRODRAGON AI
  msgQueue.push({ msg, body });
  processQueue(client);
});

// ─── Graceful shutdown ────────────────────────────────────────────────────────

process.on("SIGTERM", async () => {
  log("info", "Shutting down WhatsApp bridge...");
  await client.destroy();
  process.exit(0);
});
process.on("SIGINT", async () => {
  await client.destroy();
  process.exit(0);
});

// ─── Start ────────────────────────────────────────────────────────────────────

log("info", "Starting MICRODRAGON WhatsApp Bridge...");
log("info", `Core URL: ${CONFIG.microdragonCoreUrl}`);
client.initialize();
