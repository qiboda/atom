xz: terrain -> section -> voxel

y: terrain -> chunk -> span

## Bug
bug: cube transit face is zero.....

## todo:

 - [ ] change coord to terrain coord
 - [ ] add cfg file and cfg tools  => Don't need excel, just use UI tools or script to config game structure, damage curve, AIs and Abilities.
 - [ ] use bevy-settings to store config as resource.

 - [ ] isosurface systemset to state, to async isosurface extract. 
  

 - [ ] use telescope grep search and <c-q> to send search data to quick list
 - [ ] :cfdo %s/abc/deg/g  quick list replace every line
 - [ ] nvim-spectre like vscode replace UI.



radius | 128 | 64 | 32 | 32  32 | 32 | 64 | 128 |
    lod   3    2    1    0

2 ^ lod = voxel count every chunk
2 ^ 5 = 32
2 ^ 4 = 16
2 ^ 3 = 8

min voxle nount 8

128 + 64 + 32 + 32 = 256 max radius;

## octree 

1. 最小以及最大深度。
2. 是否细分层级。
3. 存储每个cell的位置等sdf信息。
4. cell，edge以及face的信息支持扩容。

## todo:

1. 卸载正在加载的地形。
2. 地形的mesh没有uv，必须修改材质去支持纹理。