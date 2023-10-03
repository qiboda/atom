# ability

1. 是否自定义比较两个匹配的tag是否相同。或者是否有必要？

## graph

1. graph node 在running状态再次触发，两种处理方式，决定使用方案1。
   1. 在node内部缓存多重状态，内部进行处理多次触发的情况。
   2. 申请节点，增加一个节点状态，区分是静态还是动态节点，内部也一样要进行处理（简单许多，主要是clone数据）。但需要申请节点（添加新的子节点到图中，在ecs中需要到下一帧才能生效），处理节点间引用关系。

## user

1. user custom ability bundle and ability components.
2. user custom graph builder to add custom nodes.
3. user custom ability cost energy

## todo

1. add ability how to auto crate graph. now, grant effect node cannot create a graph.

2. ability effect timer and loop can exist in effect graph. timer end also can exist in effect graph. ability effect grant to player by check add and remove condition, and need to commit a context to effect graph. so ability effect is a simple ability.

3. ability handle input.

4. ability handle other state

5. add modify attribute node

6. add check ability start, pause, abort, resume logic

7. ability to effectgraph: insert effect to call effect graph.

8. effect event use one-shot-system when bevy 0.13
