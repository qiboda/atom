import { env } from 'node:process';

const BRP_URL = 'http://127.0.0.1:15702';
const POLL_INTERVAL_MS = 2000;
const FACT_INTERVAL_MS = 30_000;   // 本地事实提取周期
const LLM_COOLDOWN_MS = 60_000;    // DeepSeek 冷却时间
const LOCAL_MODEL = env.LOCAL_MODEL ?? 'qwen3:4b';
const OLLAMA_URL = env.OLLAMA_URL ?? 'http://127.0.0.1:11434';

interface Position { x: number; y: number; z: number }
interface BrpResponse { jsonrpc: string; result?: unknown; error?: { message: string; code: number } }
interface QueryResultItem { entity: number; components: Record<string, unknown> }
interface LlmAction {
  action: 'nothing' | 'spawn_npc';
  reason: string;
  npc?: { name: string; offset_x: number; offset_z: number };
}

// ── BRP helpers ──

async function brp(method: string, params: unknown): Promise<unknown> {
  const resp = await fetch(BRP_URL, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ jsonrpc: '2.0', method, id: Date.now(), params }),
  });
  const data: BrpResponse = await resp.json();
  if (data.error) throw new Error(data.error.message);
  return data.result;
}

// ── Ollama local model ──

let localModelBusy = false;

/** 用本地模型（Qwen3）从事件日志中提取事实，纯文本摘要 */
async function extractFactsWithLocalModel(events: string): Promise<string[]> {
  if (localModelBusy) return [];
  localModelBusy = true;
  try {
    const resp = await fetch(`${OLLAMA_URL}/api/chat`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        model: LOCAL_MODEL,
        messages: [
          {
            role: 'system',
            content: '你是一个游戏世界的观察者。分析玩家的移动记录，提取出有意义的事实。每条事实用一行文本表示，不要编号，不要额外说明。如果没有有意义的信息，返回空。',
          },
          { role: 'user', content: `玩家移动记录（最近 30 秒）：\n${events}\n\n提取的事实：` },
        ],
        stream: false,
      }),
    });
    if (!resp.ok) return [];
    const data = await resp.json() as { message: { content: string } };
    return data.message.content.split('\n').map((l: string) => l.trim()).filter(Boolean);
  } finally {
    localModelBusy = false;
  }
}

// ── Event accumulator ──

interface GameEvent {
  kind: string;
  time: number;
  data: Record<string, unknown>;
}

const eventLog: GameEvent[] = [];
let lastPlayerPos: Position | null = null;

function recordMovement(pos: Position): void {
  if (lastPlayerPos) {
    const dist = Math.hypot(pos.x - lastPlayerPos.x, pos.z - lastPlayerPos.z);
    if (dist < 1.0) return;
  }
  eventLog.push({ kind: 'player_moved', time: Date.now(), data: { x: pos.x, z: pos.z } });
  lastPlayerPos = pos;
  if (eventLog.length > 200) eventLog.splice(0, eventLog.length - 200);
}

// ── Fact store ──

interface Fact {
  summary: string;
  time: number;
}

let facts: Fact[] = [];
let lastFactTime = 0;

async function runFactExtraction(): Promise<void> {
  const now = Date.now();
  if (now - lastFactTime < FACT_INTERVAL_MS) return;
  lastFactTime = now;

  // 取最近 30 秒的事件作为上下文
  const recent = eventLog.filter(e => now - e.time < 60_000);
  if (recent.length < 3) return; // 事件太少，没必要提取

  const lines = recent.map(e => `[${new Date(e.time).toISOString().slice(11, 19)}] ${e.kind} @ ${String(e.data.x)},${String(e.data.z)}`);
  const extracted = await extractFactsWithLocalModel(lines.join('\n'));

  for (const summary of extracted) {
    if (!facts.some(f => f.summary === summary)) {
      facts.push({ summary, time: now });
      console.log('[agent] Fact:', summary);
    }
  }

  if (facts.length > 50) facts = facts.slice(-50);
}

// ── DeepSeek 决策 ──

const DEEPSEEK_API_KEY = env.DEEPSEEK_API_KEY ?? '';
const DEEPSEEK_MODEL = env.DEEPSEEK_MODEL ?? 'deepseek-chat';
let llmLastCall = 0;
let npcSpawned = false;

function shouldTriggerDeepSeek(): boolean {
  const now = Date.now();
  if (now - llmLastCall < LLM_COOLDOWN_MS) return false;
  if (npcSpawned) return false;
  // 有足够的新事实才触发
  const recentFacts = facts.filter(f => now - f.time < 120_000);
  return recentFacts.length >= 2;
}

async function callDeepSeek(context: string): Promise<LlmAction> {
  llmLastCall = Date.now();
  if (!DEEPSEEK_API_KEY) {
    return { action: 'spawn_npc', reason: 'no api key fallback', npc: { name: 'NPC', offset_x: 3, offset_z: 3 } };
  }
  const resp = await fetch('https://api.deepseek.com/v1/chat/completions', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json', Authorization: `Bearer ${DEEPSEEK_API_KEY}` },
    body: JSON.stringify({
      model: DEEPSEEK_MODEL,
      messages: [
        {
          role: 'system',
          content: '你是一个游戏世界 Agent。根据世界状态返回 JSON action。',
        },
        { role: 'user', content: `世界：\n${context}\n动作？` },
      ],
      temperature: 0.7,
      max_tokens: 256,
    }),
  });
  if (!resp.ok) return { action: 'nothing', reason: `api error ${resp.status}` };
  const data = await resp.json() as { choices: { message: { content: string } }[] };
  try {
    const text = data.choices[0].message.content.replace(/```(?:json)?\s*/g, '').trim();
    return JSON.parse(text) as LlmAction;
  } catch {
    return { action: 'nothing', reason: 'parse error' };
  }
}

// ── Bevy 通信 ──

async function getPlayerPosition(): Promise<Position | null> {
  try {
    const result = await brp('world.query', {
      data: { components: ['atom_terrain::game::player::Player', 'bevy_transform::components::transform::Transform'] },
    });
    if (Array.isArray(result) && result.length > 0) {
      const tr = (result[0] as QueryResultItem).components['bevy_transform::components::transform::Transform'];
      if (typeof tr === 'object' && tr !== null) {
        const t = (tr as Record<string, unknown>)['translation'];
        if (Array.isArray(t) && t.length === 3) return { x: t[0] as number, y: t[1] as number, z: t[2] as number };
      }
    }
  } catch { /* bevy not ready */ }
  return null;
}

async function waitForBevy(maxRetries = 30): Promise<void> {
  for (let i = 0; i < maxRetries; i++) {
    try { await brp('rpc.discover', null); return; }
    catch { await new Promise(r => setTimeout(r, 1000)); }
  }
  throw new Error('Bevy not available');
}

// ── Main loop ──

async function main(): Promise<void> {
  console.log('[agent] Starting...');
  console.log('[agent] Local model:', LOCAL_MODEL, `(${OLLAMA_URL})`);
  console.log('[agent] DeepSeek:', DEEPSEEK_API_KEY ? 'configured' : 'fallback only');
  await waitForBevy();

  while (true) {
    const pos = await getPlayerPosition();
    if (pos) recordMovement(pos);

    // 用本地模型提取事实
    await runFactExtraction();

    // 满足条件时调 DeepSeek
    if (shouldTriggerDeepSeek()) {
      const lines: string[] = ['事实摘要:'];
      for (const f of facts.slice(-15)) lines.push(`- ${f.summary}`);

      const action = await callDeepSeek(lines.join('\n'));
      console.log('[agent] DeepSeek:', JSON.stringify(action));

      if (action.action === 'spawn_npc' && !npcSpawned && action.npc) {
        try {
          await brp('world.spawn_entity', {
            components: {
              'bevy_transform::components::transform::Transform': {
                translation: [pos!.x + action.npc.offset_x, pos!.y, pos!.z + action.npc.offset_z],
                rotation: [0, 0, 0, 1], scale: [1, 1, 1],
              },
            },
          });
          console.log('[agent] NPC spawned:', action.npc.name);
        } catch (e) { console.error('[agent] spawn failed:', e); }
        npcSpawned = true;
      }
    }

    const { promise, resolve } = Promise.withResolvers<void>();
    setTimeout(resolve, POLL_INTERVAL_MS);
    await promise;
  }
}

main().catch(console.error);
