
# ability

1. 是否自定义比较两个匹配的tag是否相同。或者是否有必要？

## graph

1. graph node 在running状态再次触发，两种处理方式，决定使用方案2。
   1. 在node内部缓存多重状态，内部进行处理多次触发的情况。
   2. 申请节点，增加一个节点状态，区分是静态还是动态节点，内部也一样要进行处理（简单许多，主要是clone数据）。但需要申请节点（添加新的子节点到图中，在ecs中需要到下一帧才能生效），处理节点间引用关系。
   3. 如何处理第10发子弹后续逻辑改变。设置变量。根据变量作为条件分支逻辑
   4. 每次技能触发，生成一个新的技能实例。
	   1. 如果不想生成新的技能实例，需要解决不同实例的context不同的问题。也包括graph exectuor的部分，技能状态等。
   5. 技能内部的脉冲触发，也会生成一个新的实例。
	   1. 复制节点实例，太过困难。后续节点的输入节点引用前面的节点的情况下，需要后续的节点也全部替换。改为复制内部状态。



## ecs还是内部逻辑

ability schema是一个资产。
ability instance 是一个Entity。
静态节点和动态节点都是component.

如何使用system进行更新操作。
start end等不使用system。那样无法获得数据。。。。

节点分类为3种。
1. entry：瞬发，入口(不存储运行时状态)(不需要system)
2. action：瞬发(不存储运行时状态)(使用context中的数据)(不需要system), 只有start
	1. set variable: 
	2. get variable: 
	3. seq, 
	4. for loop 
	5. 静态branch 
3. 动态分支: 持续(不存储运行时数据)(需要system)
	1. Select 执行内部逻辑(需要system)，应用trigger.
		1. 玩家的状态检测？
		2. npc的距离判断？
4. state: 持续(存储运行时数据)(需要system)
	1. 没有立即执行的后续exec，如果需要，前面添加seq的action节点来实现。

收缩节点到后面的state node，这样就能保证延迟问题。

根据后续节点类型：决定调用后续节点是直接调用函数，还是通过event trigger

## schema

在脚本中实现，配置对应的值。

## ability 

unit has attr set and state set components.
ability is a entity and is unit child.

a ability type has a effect graph(as asset)。in Resource

ability level is in effect graph context, and make effect graph node to query table row data.

run ability to run effect graph and clone state type effect node as ability children. other node only ref by node id.

effect graph is not clone and has a effect graph context in ability instance.

effect graph context is in effect graph asset; 
是否会运行时修改。



实体层次：
1. effect graph context, ability effect graph schema,
	1. effect graph nodes.
2. unit
	1. ability, ability state, ability context, effect graph context, effect executor, ability effect graph instance.
		1. effect graph nodes 

内存分层：
1. 资产层：创建Schema之后，不再改变。包括节点和context。
2. 实例层：复刻Schema，不包括静态节点。包括了包含运行时的节点和context（运行时数据）。复刻过程中，替换动态节点的entity id。
3. 运行时层：实例运行时，仍然需要到资产层复制的部分节点。(存储在实例层的context中)，复刻的entity id存储进去，根据类型id存储成hashmap。

buff 同理，替换ability即可
## user

1. user custom ability bundle and ability components.
2. user custom graph builder to add custom nodes.
3. user custom ability cost energy

## todo

- [ ] add ability how to auto crate graph. now, grant effect node cannot create a graph.
- [ ] ability effect timer and loop can exist in effect graph. timer end also can exist in effect graph. ability effect grant to player by check add and remove condition, and need to commit a context to effect graph. so ability effect is a simple ability.
- [ ] ability handle input.
- [ ] ability handle other state
- [ ] add modify attribute node
- [ ] add check ability start, pause, abort, resume logic
- [ ] ability to effect graph: insert effect to call effect graph.
- [ ] effect event use one-shot-system when bevy 0.13
- [ ] 解决多个节点无法在同一帧执行的问题。方法是不使用，event ，而是使用 one-shot system, 每当要start 一个node时，使用one shot system去执行。直到执行结束。
- [ ] layer tag 的 继承trait改为auto trait。

