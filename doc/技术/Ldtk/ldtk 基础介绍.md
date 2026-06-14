
分层：
1. project 一个ldtk文件包含的。
2. worlds 多个世界。可选的(1.5.3时是实验性功能)。
3. levels 多个关卡。
4. layers 多个层级。


# world

layout: level布局

# level

world depth: 在世界中的深度，值越大，离摄像机越远。用于解决多个关卡重叠时难以辨认的问题，

# layer

## Integer grid 

标记grid的信息，例如碰撞，各种地块的类型等。

支持rules，根据值，来自动使用tileset。

枚举值需要从顶层的UI去创建。

## Entities 

玩家起始位置，或者可以拾取的物品。

值需要从顶层的UI去创建。

## Tiles 

选择一个tileset，手动绘制其中的一个sprite。

值需要从顶层的UI去创建。

## Auto-layer

使用Integer grid layer和动态规则来渲染。

