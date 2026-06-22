# BRP Protocol Reference

Bevy Remote Protocol (BRP) — HTTP JSON-RPC at `127.0.0.1:15702`。

## RPC 方法

### `rpc.discover`

检测 Bevy 是否就绪。

```json
{"jsonrpc":"2.0","method":"rpc.discover","id":1}
```

返回: `{"jsonrpc":"2.0","result":{"methods":["world.query","world.spawn_entity",...]},"id":1}`

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
        "bevy_transform::components::transform::Transform"
      ]
    }
  }
}
```

返回 entity 数组，每个包含 `entity` (number) 和 `components` (object)。

### 类型守卫模式

BRP 返回 `unknown`，调用侧必须逐层类型守卫：

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

创建实体并附加组件。注意：**无法创建 asset handle**（`Mesh`/`Material` 需要 `Assets<T>` 资源，BRP 无法访问）。需 Bevy 侧系统补全可视组件。

---

## JSON-RPC 错误处理

错误响应格式:
```json
{"jsonrpc":"2.0","error":{"code":-32600,"message":"..."},"id":N}
```

公共 `brp()` helper 自动检查 `data.error` 并 throw：

```typescript
async function brp(method: string, params: unknown): Promise<unknown> {
  const resp = await fetch(BRP_URL, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ jsonrpc: '2.0', method, id: Date.now(), params }),
  });
  const data = await resp.json();
  if (data.error) throw new Error(data.error.message);
  return data.result;
}
```

## 序列化约定

| Rust 类型 | BRP JSON 表示 | 示例 |
|-----------|--------------|------|
| `Vec3` | `[x, y, z]` | `[1.0, -24.0, 0.0]` |
| `Quat` | `[x, y, z, w]` | `[0, 0, 0, 1]` |
| Unit struct | `{}` | `{}` |
| Tuple struct | 内部值 | `"NPC"` |
