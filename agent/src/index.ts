import { env } from 'node:process';

const BRP_URL = 'http://127.0.0.1:15702';
const POLL_INTERVAL_MS = 2000;

interface Position {
  x: number;
  y: number;
  z: number;
}

interface BrpResponse {
  jsonrpc: string;
  result?: unknown;
  error?: { message: string; code: number };
}

interface QueryResultItem {
  entity: number;
  components: Record<string, unknown>;
}

interface LlmAction {
  action: 'nothing' | 'spawn_npc';
  reason: string;
  npc?: {
    name: string;
    offset_x: number;
    offset_z: number;
  };
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

function delay(ms: number): Promise<void> {
  return new Promise((r) => setTimeout(r, ms));
}

async function waitForBevy(maxRetries = 30): Promise<void> {
  for (let i = 0; i < maxRetries; i++) {
    try {
      await brp('rpc.discover', null);
      console.log('[agent] Connected to Bevy at', BRP_URL);
      return;
    } catch {
      console.log('[agent] Waiting for Bevy...');
      await delay(1000);
    }
  }
  throw new Error('Bevy not available after max retries');
}

function isQueryResultArray(value: unknown): value is QueryResultItem[] {
  return Array.isArray(value) && value.every((v) => typeof v === 'object' && v !== null && 'entity' in v);
}

function extractTranslation(transform: unknown): [number, number, number] | null {
  if (typeof transform === 'object' && transform !== null) {
    const t = transform as Record<string, unknown>;
    const translation = t['translation'];
    if (Array.isArray(translation) && translation.length === 3) {
      return [translation[0] as number, translation[1] as number, translation[2] as number];
    }
  }
  return null;
}

async function getPlayerPosition(): Promise<Position | null> {
  try {
    const result = await brp('world.query', {
      data: {
        components: [
          'atom_terrain::game::player::Player',
          'bevy_transform::components::transform::Transform',
        ],
      },
    });
    if (isQueryResultArray(result) && result.length > 0) {
      const transform = result[0].components['bevy_transform::components::transform::Transform'];
      const t = extractTranslation(transform);
      if (t) {
        return { x: t[0], y: t[1], z: t[2] };
      }
    }
  } catch {
    // Bevy not ready yet
  }
  return null;
}

// ── DeepSeek LLM ──

const DEEPSEEK_API_KEY = env.DEEPSEEK_API_KEY ?? '';
const DEEPSEEK_MODEL = env.DEEPSEEK_MODEL ?? 'deepseek-chat';

/// 调用 DeepSeek chat API，返回结构化 action。
async function decideAction(worldContext: string): Promise<LlmAction> {
  if (!DEEPSEEK_API_KEY) {
    // 无 API key 时走回退逻辑
    return { action: 'spawn_npc', reason: 'no api key fallback', npc: { name: 'NPC', offset_x: 3, offset_z: 3 } };
  }

  const resp = await fetch('https://api.deepseek.com/v1/chat/completions', {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      Authorization: `Bearer ${DEEPSEEK_API_KEY}`,
    },
    body: JSON.stringify({
      model: DEEPSEEK_MODEL,
      messages: [
        {
          role: 'system',
          content: `你是一个游戏世界 Agent（上帝）。你通过 JSON-RPC 控制 Bevy ECS 世界。
当前可用的动作：
- {"action":"nothing","reason":"..."}
- {"action":"spawn_npc","reason":"...","npc":{"name":"...","offset_x":N,"offset_z":N}} — 在玩家偏移处生成一个 NPC

请根据当前世界状态返回一个 action。只返回 JSON，不要多余文字。`,
        },
        {
          role: 'user',
          content: `当前世界状态：\n${worldContext}\n\n你要做什么？`,
        },
      ],
      temperature: 0.7,
      max_tokens: 256,
    }),
  });

  if (!resp.ok) {
    console.warn('[agent] DeepSeek API error:', resp.status);
    return { action: 'nothing', reason: 'api error' };
  }

  const data = await resp.json() as { choices: { message: { content: string } }[] };
  try {
    const text = data.choices[0].message.content;
    // Strip markdown code fences if present
    const json = text.replace(/```(?:json)?\s*/g, '').trim();
    return JSON.parse(json) as LlmAction;
  } catch {
    return { action: 'nothing', reason: 'parse error' };
  }
}

/// 构建世界上下文字符串（供 LLM 参考）
function buildWorldContext(pos: Position | null, hasNpc: boolean): string {
  const lines: string[] = [];
  if (pos) {
    lines.push(`- 玩家位置: (${pos.x.toFixed(1)}, ${pos.y.toFixed(1)}, ${pos.z.toFixed(1)})`);
  } else {
    lines.push('- 玩家未找到');
  }
  lines.push(`- 已有 NPC: ${hasNpc ? '是' : '否'}`);
  return lines.join('\n');
}

// ── Main loop ──

async function main(): Promise<void> {
  console.log('[agent] Starting Atom Agent sidecar...');
  console.log('[agent] DeepSeek API key present:', !!DEEPSEEK_API_KEY);
  await waitForBevy();
  console.log('[agent] Agent loop started');

  let lastPos: Position | null = null;
  let npcSpawned = false;

  while (true) {
    const pos = await getPlayerPosition();
    if (pos) {
      console.log('[agent] Player at', pos);

      // 只在玩家位置变化时才跑 LLM 决策
      const moved = !lastPos || Math.abs(pos.x - lastPos.x) > 0.1 || Math.abs(pos.z - lastPos.z) > 0.1;
      if (moved) {
        const context = buildWorldContext(pos, npcSpawned);
        const action = await decideAction(context);
        console.log('[agent] LLM decision:', JSON.stringify(action));

        if (action.action === 'spawn_npc' && !npcSpawned && action.npc) {
          try {
            await brp('world.spawn_entity', {
              components: {
                'bevy_transform::components::transform::Transform': {
                  translation: [pos.x + action.npc.offset_x, pos.y, pos.z + action.npc.offset_z],
                  rotation: [0, 0, 0, 1],
                  scale: [1, 1, 1],
                },
              },
            });
            console.log('[agent] NPC spawned:', action.npc.name);
          } catch (e) {
            console.error('[agent] Failed to spawn NPC:', e);
          }
          npcSpawned = true;
        } else if (action.action === 'spawn_npc' && npcSpawned) {
          console.log('[agent] LLM wants to spawn, but already have an NPC');
        }
      }
      lastPos = pos;
    } else {
      console.log('[agent] Player not found, waiting...');
    }

    await delay(POLL_INTERVAL_MS);
  }
}

main().catch(console.error);
