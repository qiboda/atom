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


