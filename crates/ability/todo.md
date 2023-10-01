# ability

1. 如何比较两个相同tag的数据。
2. 或者是否ability的tag需要数据？
3. 是否自定义比较两个匹配的tag是否相同。或者是否有必要？
4. tag container 的 key要怎么处理？

## graph

1. graph node 在running状态再次触发，两种处理方式，决定使用方案1。
   1. 在node内部缓存多重状态，内部进行处理多次触发的情况。
   2. 申请节点，增加一个节点状态，区分是静态还是动态节点，内部也一样要进行处理（简单许多，主要是clone数据）。但需要申请节点（添加新的子节点到图中，在ecs中需要到下一帧才能生效），处理节点间引用关系。
2. ability ref graph but graph is not ability child.

## user

1. user custom ability bundle and ability components.
2. user custom graph builder to add custom nodes.
3. user custom ability cost energy

## todo

1. add ability.

2. remove ability, 同时移除graph。

3. ability command

4. ability handle input.

5. ability handle other state

6. add modify attribute node

7. how to use ability layer tag.

8. add check ability start, pause, abort, resume logic

9. ability state to component

10. ability graph ref to component

11. impl effect as buff and debuff, as if dota2 modifier. effect need a single graph, because  will grant to other units sometimes.

12. gameplay cue don't need, just a graph node.

13. ability to effectgraph: insert effect to call effect graph.

14. use one of lazy_static and once_cell.
