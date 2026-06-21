# BRP Protocol Patterns

Bevy Remote Protocol (BRP) — HTTP JSON-RPC at `127.0.0.1:15702`。

## RPC 方法

### `rpc.discover`

检测 Bevy 是否就绪。

```json
{"jsonrpc":"2.0","method":"rpc.discover","id":1}
```

返回: `{"jsonrpc":"2.0","result":{"methods":["world.query","world.spawn_entity",...]},"id":1}`

Agent 侧用于 `waitForBevy()` 轮询，每 1s 重试，最多 30 次。

---

### `world.query`

查询 ECS 组件。组件路径为 Rust 完整路径字符串。

```json
{
  "jsonrpc":"2.0",
  "method":"world.query",
  "id":2,
  "params":{
    "data":{
      "components":[
        "atom_terrain::game::player::Player",
        "bevy_transform::components::transform::Transform"
      ]
    }
  }
}
```

返回:
```json
{
  "jsonrpc":"2.0",
  "result":[
    {
      "entity": 123,
      "components": {
        "atom_terrain::game::player::Player": {},
        "bevy_transform::components::transform::Transform": {
          "translation": [1.0, -24.0, 0.0],
          "rotation": [0.0, 0.0, 0.0, 1.0],
          "scale": [1.0, 1.0, 1.0]
        }
      }
    }
  ],
  "id":2
}
```

### 类型守卫模式

BRP 返回 `unknown`，Agent 侧必须逐层类型守卫：

```typescript
function isQueryResultArray(value: unknown): value is QueryResultItem[] {
  if (!Array.isArray(value)) return false;
  return value.every((item): item is QueryResultItem =>
    item != null &&
    typeof item === 'object' &&
    'entity' in item &&
    typeof (item as Record<string, unknown>).entity === 'number' &&
    'components' in item
  );
}
```

---

### `world.spawn_entity`

创建实体并附加组件。

```json
{
  "jsonrpc":"2.0",
  "method":"world.spawn_entity",
  "id":3,
  "params":{
    "components":{
      "atom_terrain::game::player::Name": "NPC",
      "bevy_transform::components::transform::Transform": {
        "translation": [4.0, -24.0, 3.0],
        "rotation": [0, 0, 0, 1],
        "scale": [1, 1, 1]
      }
    }
  }
}
```

### 关键限制

- **无法创建 asset handle**: `Mesh3d(Handle<Mesh>)`、`MeshMaterial3d(Handle<StandardMaterial>)` 需要 `Assets<Mesh>` / `Assets<StandardMaterial>` 资源，BRP 无法访问。
- **解决方案**: Agent spawn 时附带 `Name("NPC")` 标识，Bevy 侧 `decorate_agent_entities` 系统检测后补 mesh + material。

---

## JSON-RPC 错误处理

BRP 错误响应格式:
```json
{"jsonrpc":"2.0","error":{"code":-32600,"message":"..."},"id":N}
```

Agent 侧公共 `brp()` helper 自动检查 `data.error` 并 throw。

```typescript
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
```

---

## 序列化约定

| Rust 类型 | BRP JSON 表示 | 示例 |
|-----------|--------------|------|
| `Vec3` | `[x, y, z]` | `[1.0, -24.0, 0.0]` |
| `Quat` | `[x, y, z, w]` | `[0, 0, 0, 1]` |
| `Vec3` (scale) | `[x, y, z]` | `[1, 1, 1]` |
| Unit struct (`Player`) | `{}` | `{}` |
| Tuple struct (`Name(String)`) | 内部值 | `"NPC"` |
| Tuple struct (`Health(f32)`) | 内部值 | `100.0` |

## 组件路径速查

| 组件 | BRP 路径 |
|------|---------|
| Player | `atom_terrain::game::player::Player` |
| Name | `atom_terrain::game::player::Name` |
| Health | `atom_terrain::game::player::Health` |
| MoveSpeed | `atom_terrain::game::player::MoveSpeed` |
| TopDownCamera | `atom_terrain::game::camera::TopDownCamera` |
| Transform | `bevy_transform::components::transform::Transform` |
