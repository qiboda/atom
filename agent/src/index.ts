import { env } from 'node:process';

const BRP_URL = 'http://127.0.0.1:15702';
const POLL_INTERVAL_MS = 2000;
const FACT_INTERVAL_MS = 30_000;   // 本地事实提取周期
const LLM_COOLDOWN_MS = 60_000;    // DeepSeek 冷却时间

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

// ── Event accumulator ──

interface GameEvent {
  kind: 'player_moved' | 'player_stopped' | string;
  time: number;
  data: Record<string, unknown>;
}

const eventLog: GameEvent[] = [];
let lastPlayerPos: Position | null = null;

/** 记录玩家移动事件（带距离门限，避免阻塞 buffer） */
function recordMovement(pos: Position): void {
  if (lastPlayerPos) {
    const dx = pos.x - lastPlayerPos.x;
    const dz = pos.z - lastPlayerPos.z;
    const dist = Math.sqrt(dx * dx + dz * dz);
    if (dist < 1.0) return; // 太小不记
  }
  eventLog.push({ kind: 'player_moved', time: Date.now(), data: { x: pos.x, z: pos.z } });
  lastPlayerPos = pos;
  if (eventLog.length > 200) eventLog.splice(0, eventLog.length - 200); // 上限
}

// ── Local fact extraction（纯规则，无本地模型） ──

interface Fact {
  summary: string;
  time: number;
}

let facts: Fact[] = [];
let lastFactTime = 0;

function extractFacts(): Fact[] {
  const now = Date.now();
  if (now - lastFactTime < FACT_INTERVAL_MS) return [];
  lastFactTime = now;

  const newFacts: Fact[] = [];

  // 统计玩家到过的不同区域（以 10 为粒度）
  const areas = new Set<string>();
  for (const e of eventLog) {
    if (e.kind === 'player_moved') {
      const gx = Math.round((e.data.x as number) / 10);
      const gz = Math.round((e.data.z as number) / 10);
      areas.add(`${gx},${gz}`);
    }
  }

  if (areas.size > 0) {
    newFacts.push({ summary: `玩家探索了 ${areas.size} 个区域`, time: now });
  }

  // 判断是否应该触发 DeepSeek
  const totalEvents = eventLog.length;
  if (totalEvents > 5 && facts.length === 0) {
    newFacts.push({ summary: '首次积累足够事件', time: now });
  }
  if (areas.size >= 3) {
    newFacts.push({ summary: '玩家探索了 3 个以上不同区域', time: now });
  }

  return newFacts;
}

// ── DeepSeek 决策（带冷却 + 触发条件） ──

const DEEPSEEK_API_KEY = env.DEEPSEEK_API_KEY ?? '';
const DEEPSEEK_MODEL = env.DEEPSEEK_MODEL ?? 'deepseek-chat';
let llmLastCall = 0;
let npcSpawned = false;

function shouldTriggerDeepSeek(): boolean {
  const now = Date.now();
  if (now - llmLastCall < LLM_COOLDOWN_MS) return false;     // 冷却中
  if (npcSpawned) return false;                                // 已经 spawn 过了，暂时没新目标

  // 有值得关注的新事实才触发
  const interesting = facts.some(f =>
    f.summary.includes('首次') || f.summary.includes('3 个')
  );
  return interesting;
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
  console.log('[agent] DeepSeek:', DEEPSEEK_API_KEY ? 'configured' : 'fallback only');
  await waitForBevy();

  while (true) {
    const pos = await getPlayerPosition();
    if (pos) {
      recordMovement(pos);
    }

    // 定期提取本地事实
    for (const f of extractFacts()) {
      facts.push(f);
      console.log('[agent] Fact:', f.summary);
    }
    if (facts.length > 50) facts = facts.slice(-50);

    // 满足条件时调 DeepSeek
    if (shouldTriggerDeepSeek()) {
      const lines: string[] = ['玩家移动记录（最近 20 条）:'];
      for (const e of eventLog.slice(-20)) {
        lines.push(`- ${e.kind} @ (${String(e.data.x)}, ${String(e.data.z)})`);
      }
      lines.push('', `当前事实（${facts.length} 条）:`);
      for (const f of facts.slice(-10)) lines.push(`- ${f.summary}`);

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
