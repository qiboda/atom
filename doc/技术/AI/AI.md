---
date created: 2023-11-17 22:52
tags:
  - game/ai
  - atom
---

## Todo

使用utility 以及goap和smart object来实现AI。
1. 利用utlity ai 选择目标
2. 使用goap进行规划。
3. 其他交互物使用smart object来实现与角色的交互。

## HTN 的一些考虑
## Feature

1. 支持Utility AI，即多个Scores。Planning根据最后的多个分数，评估出最好的Plan。
2. 支持类似Unreal 行为树的装饰器以及服务器。
3. 移除黑板（世界状态），支持ECS的Query，将整个游戏作为黑板。
4. 感知组件。

## 
- [ ] 如何实现世界状态的转换？
- [ ] 每方案每黑板吗？ 那么sensing 如何和黑板交互？sensing直接设置值到黑板。
- [ ] 如何连接节点。表示节点的Pin？不需要, 只需要知道后续节点即可。
- [ ] 遍历节点，做预测。


分派一个用于percidt的system function。
添加一个类似于Extracted的system 参数类型，内部存储了模拟的改变值（值存储在）。且会


添加装饰器和服务器就是简单的添加组件，他们只是trait不同。不能同时实现两个trait。


常规写法的提取可变成员变量，在ecs里就简单的将一个Component
切分为两个Component，可变的和不可变的变量。。。。。

执行节点时，如何确定装饰器和服务器的执行顺序。
	将装饰器和服务器作为一个单独的Entity，作为Action的子类。


一个agent的HTN的所有节点都作为该Planner Instance的子节点。

Planning阶段可以像渲染系统一样，单独开一个Subapp，用来模拟运行，最后将计划结果返回原世界。

## 缺点

1. 只能预测自己的行为，但是不能预估敌人的行为。因此依然需要实时的replanning。