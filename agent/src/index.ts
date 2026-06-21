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

// BRP RPC helper — validates the JSON-RPC response envelope.
async function brp(method: string, params: unknown): Promise<unknown> {
  const resp = await fetch(BRP_URL, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
      jsonrpc: '2.0',
      method,
      id: Date.now(),
      params,
    }),
  });
  const data: BrpResponse = await resp.json();
  if (data.error) throw new Error(data.error.message);
  return data.result;
}

function delay(ms: number): Promise<void> {
  const { promise, resolve } = Promise.withResolvers<void>();
  setTimeout(resolve, ms);
  return promise;
}

// Wait for Bevy to be ready
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
  if (!Array.isArray(value)) return false;
  return value.every((item): item is QueryResultItem =>
    item != null &&
    typeof item === 'object' &&
    'entity' in item &&
    typeof (item as Record<string, unknown>).entity === 'number' &&
    'components' in item &&
    typeof (item as Record<string, unknown>).components === 'object'
  );
}

function extractTranslation(transform: unknown): [number, number, number] | null {
  if (!transform || typeof transform !== 'object') return null;
  const record = transform as Record<string, unknown>;
  const t = record.translation;
  if (!Array.isArray(t) || t.length < 3) return null;
  if (
    typeof t[0] !== 'number' ||
    typeof t[1] !== 'number' ||
    typeof t[2] !== 'number'
  ) {
    return null;
  }
  return [t[0], t[1], t[2]];
}

// Query player position via world.query
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

    if (!isQueryResultArray(result) || result.length === 0) return null;

    const player = result[0];
    const transform = player.components['bevy_transform::components::transform::Transform'];
    const translation = extractTranslation(transform);
    if (!translation) return null;

    return { x: translation[0], y: translation[1], z: translation[2] };
  } catch {
    // Bevy not ready yet
    return null;
  }
}

// Main loop
async function main(): Promise<void> {
  console.log('[agent] Starting Atom Agent sidecar...');
  await waitForBevy();

  let lastPos: Position | null = null;
  let npcSpawned = false;

  while (true) {
    const pos = await getPlayerPosition();
    if (pos) {
      console.log(
        `[agent] Player at (${pos.x.toFixed(1)}, ${pos.y.toFixed(1)}, ${pos.z.toFixed(1)})`,
      );

      // If player has moved significantly and no NPC yet, spawn one
      if (!npcSpawned && lastPos) {
        const dx = pos.x - lastPos.x;
        const dz = pos.z - lastPos.z;
        if (Math.abs(dx) > 5 || Math.abs(dz) > 5) {
          console.log('[agent] Player moved far enough, spawning NPC...');
          try {
            await brp('world.spawn_entity', {
              components: {
                'atom_terrain::game::player::Name': 'NPC',
                'bevy_transform::components::transform::Transform': {
                  translation: [pos.x + 3, pos.y, pos.z + 3],
                  rotation: [0, 0, 0, 1],
                  scale: [1, 1, 1],
                },
              },
            });
            console.log('[agent] NPC spawned!');
          } catch (e) {
            console.log('[agent] Failed to spawn NPC:', e);
          }
          npcSpawned = true;
        }
      }
      lastPos = pos;
    }

    await delay(POLL_INTERVAL_MS);
  }
}

main().catch(console.error);
