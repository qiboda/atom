# TypeScript 编码规范 — Pi Agent

## 运行时

- **运行时**: `tsx`（TypeScript Execute — 零配置执行 `.ts`）
- **模块系统**: ESM (`"type": "module"` in `package.json`)
- **依赖**: 无运行时 npm 包，仅 `tsx` 为 devDependency
- **Node 版本**: 内置 `fetch`（Node 18+），无 polyfill

## 类型

- `strict: true` — 所有类型必须显式声明
- 接口优先于 type alias（`interface` 可扩展）
- BRP 返回值统一 `unknown` → 类型守卫收窄
- `Promise<void>` 不省略 void

```typescript
// ✅ 好
async function getPlayerPosition(): Promise<Position | null> { ... }

// ❌ 坏
async function getPlayerPosition() { ... }  // 隐式 any
```

## 错误处理

- `try/catch` 包裹所有 BRP 调用
- `brp()` helper 自动 throw on `data.error`
- 不吞异常：catch 后至少 `console.error` 或 fallback 值

```typescript
try {
  await brp('world.spawn_entity', { ... });
} catch (e) {
  console.log('[agent] Failed to spawn NPC:', e);  // 不让进程崩溃
}
```

## 异步模式

- `Promise.withResolvers()` 封装 `setTimeout` / 回调
- `async/await` 不混用 `.then()`
- 主循环用 `while (true)` + `await delay()`

```typescript
function delay(ms: number): Promise<void> {
  const { promise, resolve } = Promise.withResolvers<void>();
  setTimeout(resolve, ms);
  return promise;
}
```

## 命名

- 函数: camelCase (`getPlayerPosition`, `waitForBevy`)
- 接口: PascalCase (`BrpResponse`, `QueryResultItem`)
- 常量: UPPER_SNAKE (`BRP_URL`, `POLL_INTERVAL_MS`)
- 类型守卫: `is` 前缀 (`isQueryResultArray`)

## 项目结构

```
agent/
├── package.json          # 仅 tsx 依赖，ESM
├── tsconfig.json         # strict, ESNext
└── src/
    └── index.ts          # 单文件（MVP）；未来按功能拆
```
