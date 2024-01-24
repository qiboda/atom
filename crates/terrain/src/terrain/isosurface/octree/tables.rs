/*
  USING LEFT HANDED SYSTEM (... i know...)

    Data tables for marching squares


    Vertices have the following IDs
   0        2
    --------
    |      |
    |      |       ^-Z
    |      |       |
    --------       --> X
   1        3

    (divided by two from d[i] indices, since we're
     looking at a flat ASDF and we don't care about z)


    Edges are numbered as follows:
       0
    --------
    |      |
  2 |      | 3     ^-Z
    |      |       |
    --------       --> X
       1

*/

#[derive(EnumIter, EnumCount, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Face2DVertex {
    LeftUp = 0,
    LeftDown = 1,
    RightUp = 2,
    RightDown = 3,
}

#[derive(EnumIter, EnumCount, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Face2DEdge {
    Up = 0,
    Down = 1,
    Left = 2,
    Right = 3,
}

// 0        2
//  --------
//  |      |
//  |      |       ^-Z
//  |      |       |
//  --------       --> X
// 1        3
//

// For a given set of filled corners, this array defines
// the cell edges from which we draw interior edges
#[rustfmt::skip]
pub const EDGE_MAP: [[[Option<Face2DEdge>; 2]; 2]; 16] = [

    // Up = 0,
    // Down = 1,
    // Left = 2,
    // Right = 3,
    [[None, None], [None, None]],                                                                          // ----
    [[Some(Face2DEdge::Up), Some(Face2DEdge::Left)], [None, None]],                                        // ---0
    [[Some(Face2DEdge::Left), Some(Face2DEdge::Down)], [None, None]],                                      // --1-
    [[Some(Face2DEdge::Up), Some(Face2DEdge::Down)], [None, None]],                                        // --10
    [[Some(Face2DEdge::Right), Some(Face2DEdge::Up)], [None, None]],                                       // -2--
    [[Some(Face2DEdge::Right), Some(Face2DEdge::Left)], [None, None]],                                     // -2-0
    [[Some(Face2DEdge::Right), Some(Face2DEdge::Up)], [Some(Face2DEdge::Left), Some(Face2DEdge::Down)]],   // -21- //ambig
    [[Some(Face2DEdge::Right), Some(Face2DEdge::Down)], [None, None]],                                     // -210
    
    [[Some(Face2DEdge::Down), Some(Face2DEdge::Right)], [None, None]],                                     // 3---
    [[Some(Face2DEdge::Down), Some(Face2DEdge::Right)], [Some(Face2DEdge::Up), Some(Face2DEdge::Left)]],   // 3--0 //ambig // 可以通过法线来判断
    [[Some(Face2DEdge::Left), Some(Face2DEdge::Right)], [None, None]],                                     // 3-1-
    [[Some(Face2DEdge::Up), Some(Face2DEdge::Right)], [None, None]],                                       // 3-10
    [[Some(Face2DEdge::Down), Some(Face2DEdge::Up)], [None, None]],                                        // 32--
    [[Some(Face2DEdge::Down), Some(Face2DEdge::Left)], [None, None]],                                      // 32-0
    [[Some(Face2DEdge::Left), Some(Face2DEdge::Up)], [None, None]],                                        // 321-
    [[None, None], [None, None]],                                                                          // 3210
];

// pub enum Face2DVertex {
//     LeftUp = 0,
//     LeftDown = 1,
//     RightUp = 2,
//     RightDown = 3,
// }

// Indexed by edge number, returns vertex index
//
pub const VERTEX_MAP: [[Face2DVertex; 2]; Face2DEdge::COUNT] = [
    [Face2DVertex::LeftUp, Face2DVertex::RightUp],
    [Face2DVertex::RightDown, Face2DVertex::LeftDown],
    [Face2DVertex::LeftDown, Face2DVertex::LeftUp],
    [Face2DVertex::RightUp, Face2DVertex::RightDown],
];

/*

 Vertex and Edge Index Map

       2-------0------6
      /.             /|
     10.           11 |
    /  2           /  3
   /   .          /   |     ^ Y
  3-------5------7    |     |
  |    0 . . 1 . |. . 4     --> X
  |   .          |   /     /
  6  8           7  9     / z
  | .            | /     |/
  |.             |/
  1-------4------5


     Face Index Map

         -----
         | 0 |
     -----------------     ^ -z
     | 5 | 2 | 4 | 3 |     |
     -----------------     ---> x
         | 1 |
         -----

     Face Index Layout
       o--------------o
      /.             /|
     / .            / |
    /  .    3      /  |
   /   .      (0) /   |     ^ Y
  o--------------o  5 |     |
  |(4) . . . . . |. . o     --> X
  |   .   1      |   /
  |  .           |  /
  | .      (2)   | /
  |.             |/
  o--------------o



 Cell Point and Subcell Layout

     (2)o--------------o(6)
       /.             /|
      / .            / |
     /  .           /  |
    /   .          /   |     ^ Y
(3)o--------------o(7) |     |
   | (0). . . . . |. . o(4)  --> X
   |   .          |   /
   |  .           |  /
   | .            | /
   |.             |/
(1)o--------------o(5)

*/

use strum::EnumCount;
use strum_macros::{EnumIter, FromRepr};

use super::face::FaceIndex;


#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIter, EnumCount, Hash)]
pub enum EdgeIndex {
    XAxisTopBack = 0,
    XAxisBottomBack = 1,
    LeftYAxisBack = 2,
    RightYAxisBack = 3,
    XAxisBottomFront = 4,
    XAxisTopFront = 5,
    LeftYAxisFront = 6,
    RightYAxisFront = 7,
    LeftBottomZAxis = 8,
    RightBottomZAxis = 9,
    LeftTopZAxis = 10,
    RightTopZAxis = 11,
}

// x, y, z => xyz
#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIter, EnumCount, Hash)]
pub enum VertexIndex {
    LeftBottomBack = 0,
    LeftBottomFront = 1,
    LeftTopBack = 2,
    LeftTopFront = 3,
    RightBottomBack = 4,
    RightBottomFront = 5,
    RightTopBack = 6,
    RightTopFront = 7,
}

// x, y, z => xyz
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, FromRepr, EnumIter, EnumCount)]
pub enum SubCellIndex {
    LeftBottomBack = 0,
    LeftBottomFront = 1,
    LeftTopBack = 2,
    LeftTopFront = 3,
    RightBottomBack = 4,
    RightBottomFront = 5,
    RightTopBack = 6,
    RightTopFront = 7,
}

// x, y, z => xyz
#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIter, EnumCount)]
pub enum EdgeDirection {
    XAxis = 0,
    YAxis = 1,
    ZAxis = 2,
}

//----------------------------------------------

// FACE_VERTEX[i] gives the four vertices (as 3-bit corner indices)
// that define face i on an octree cell
//
pub const FACE_VERTEX: [[VertexIndex; Face2DVertex::COUNT]; FaceIndex::COUNT] = [
    [
        VertexIndex::LeftTopBack,
        VertexIndex::LeftBottomBack,
        VertexIndex::RightTopBack,
        VertexIndex::RightBottomBack,
    ], // face 0
    [
        VertexIndex::LeftBottomFront,
        VertexIndex::LeftTopFront,
        VertexIndex::RightBottomFront,
        VertexIndex::RightTopFront,
    ], // face 1
    [
        VertexIndex::LeftBottomBack,
        VertexIndex::LeftBottomFront,
        VertexIndex::RightBottomBack,
        VertexIndex::RightBottomFront,
    ], // face 2
    [
        VertexIndex::RightTopBack,
        VertexIndex::RightTopFront,
        VertexIndex::LeftTopBack,
        VertexIndex::LeftTopFront,
    ], // face 3
    [
        VertexIndex::LeftTopBack,
        VertexIndex::LeftTopFront,
        VertexIndex::LeftBottomBack,
        VertexIndex::LeftBottomFront,
    ], // face 4
    [
        VertexIndex::RightBottomBack,
        VertexIndex::RightBottomFront,
        VertexIndex::RightTopBack,
        VertexIndex::RightTopFront,
    ], // face 5
];

// Given an edge it gives back the two
// cell vertices that connect it
// @ at pos 0 and 1 - the order will always
// be in the positive direction...
//
pub const EDGE_VERTICES: [[VertexIndex; 2]; EdgeIndex::COUNT] = [
    [VertexIndex::LeftTopBack, VertexIndex::RightTopBack], // edge 0
    [VertexIndex::LeftBottomBack, VertexIndex::RightBottomBack], // edge 1
    [VertexIndex::LeftBottomBack, VertexIndex::LeftTopBack], // edge 2
    [VertexIndex::RightBottomBack, VertexIndex::RightTopBack], // edge 3
    [VertexIndex::LeftBottomFront, VertexIndex::RightBottomFront], // edge 4
    [VertexIndex::LeftTopFront, VertexIndex::RightTopFront], // edge 5
    [VertexIndex::LeftBottomFront, VertexIndex::LeftTopFront], // edge 6
    [VertexIndex::RightBottomFront, VertexIndex::RightTopFront], // edge 7
    [VertexIndex::LeftBottomBack, VertexIndex::LeftBottomFront], // edge 8
    [VertexIndex::RightBottomBack, VertexIndex::RightBottomFront], // edge 9
    [VertexIndex::LeftTopBack, VertexIndex::LeftTopFront], // edge 10
    [VertexIndex::RightTopBack, VertexIndex::RightTopFront], // edge 11
];

pub const EDGE_DIRECTION: [EdgeDirection; EdgeIndex::COUNT] = [
    EdgeDirection::XAxis,
    EdgeDirection::XAxis,
    EdgeDirection::YAxis,
    EdgeDirection::YAxis,
    EdgeDirection::XAxis,
    EdgeDirection::XAxis,
    EdgeDirection::YAxis,
    EdgeDirection::YAxis,
    EdgeDirection::ZAxis,
    EdgeDirection::ZAxis,
    EdgeDirection::ZAxis,
    EdgeDirection::ZAxis,
];

pub const FACE_DIRECTION: [EdgeDirection; FaceIndex::COUNT] = [
    EdgeDirection::ZAxis,
    EdgeDirection::ZAxis,
    EdgeDirection::YAxis,
    EdgeDirection::YAxis,
    EdgeDirection::XAxis,
    EdgeDirection::XAxis,
];

/// The table for face twins.
/// Twins是两个相邻的Cell的重叠的面，
pub const FACE_TWIN_TABLE: [[FaceIndex; 2]; FaceIndex::COUNT] = [
    [FaceIndex::Back, FaceIndex::Front],
    [FaceIndex::Front, FaceIndex::Back],
    [FaceIndex::Bottom, FaceIndex::Top],
    [FaceIndex::Top, FaceIndex::Bottom],
    [FaceIndex::Left, FaceIndex::Right],
    [FaceIndex::Right, FaceIndex::Left],
];

pub const FACE_NEIGHBOUR: [FaceIndex; FaceIndex::COUNT] = [
    FaceIndex::Front,
    FaceIndex::Back,
    FaceIndex::Top,
    FaceIndex::Bottom,
    FaceIndex::Right,
    FaceIndex::Left,
];

pub const FACE_2_SUBCELL: [[SubCellIndex; 4]; FaceIndex::COUNT] = [
    [
        SubCellIndex::LeftBottomBack,
        SubCellIndex::LeftTopBack,
        SubCellIndex::RightBottomBack,
        SubCellIndex::RightTopBack,
    ],
    [
        SubCellIndex::LeftBottomFront,
        SubCellIndex::LeftTopFront,
        SubCellIndex::RightBottomFront,
        SubCellIndex::RightTopFront,
    ],
    [
        SubCellIndex::LeftBottomBack,
        SubCellIndex::LeftBottomFront,
        SubCellIndex::RightBottomBack,
        SubCellIndex::RightBottomFront,
    ],
    [
        SubCellIndex::RightTopBack,
        SubCellIndex::RightTopFront,
        SubCellIndex::LeftTopBack,
        SubCellIndex::LeftTopFront,
    ],
    [
        SubCellIndex::LeftTopBack,
        SubCellIndex::LeftTopFront,
        SubCellIndex::LeftBottomBack,
        SubCellIndex::LeftBottomFront,
    ],
    [
        SubCellIndex::RightBottomBack,
        SubCellIndex::RightBottomFront,
        SubCellIndex::RightTopBack,
        SubCellIndex::RightTopFront,
    ],
];

//  Cell Point and Subcell Layout
//
//      (2)o--------------o(6)
//        /.             /|
//       / .            / |
//      /  .           /  |
//     /   .          /   |     ^ Y
// (3)o--------------o(7) |     |
//    | (0). . . . . |. . o(4)  --> X
//    |   .          |   /
//    |  .           |  /
//    | .            | /
//    |.             |/
// (1)o--------------o(5)
//
//
/// @brief Cell neighbour table
/// See: 'Cell Point and Subcell Layout' in tables header
///
/// 在对应的方向上，当前Cell的ID，对应的相邻Cell的ID, 以及是否是相同的父cell。
///
pub const NEIGHBOUR_ADDRESS_TABLE: [[(SubCellIndex, bool); SubCellIndex::COUNT]; FaceIndex::COUNT] = [
    [
        (SubCellIndex::LeftBottomFront, false),
        (SubCellIndex::LeftBottomBack, true),
        (SubCellIndex::LeftTopFront, false),
        (SubCellIndex::LeftTopBack, true),
        (SubCellIndex::RightBottomFront, false),
        (SubCellIndex::RightBottomBack, true),
        (SubCellIndex::RightTopFront, false),
        (SubCellIndex::RightTopBack, true),
    ], // BACK NEIGHBOUR
    [
        (SubCellIndex::LeftBottomFront, true),
        (SubCellIndex::LeftBottomBack, false),
        (SubCellIndex::LeftTopFront, true),
        (SubCellIndex::LeftTopBack, false),
        (SubCellIndex::RightBottomFront, true),
        (SubCellIndex::RightBottomBack, false),
        (SubCellIndex::RightTopFront, true),
        (SubCellIndex::RightTopBack, false),
    ], // FRONT NEIGHBOUR
    [
        (SubCellIndex::LeftTopBack, false),
        (SubCellIndex::LeftTopFront, false),
        (SubCellIndex::LeftBottomBack, true),
        (SubCellIndex::LeftBottomFront, true),
        (SubCellIndex::RightTopBack, false),
        (SubCellIndex::RightTopFront, false),
        (SubCellIndex::RightBottomBack, true),
        (SubCellIndex::RightBottomFront, true),
    ], // BOTTOM NEIGHBOUR
    [
        (SubCellIndex::LeftTopBack, true),
        (SubCellIndex::LeftTopFront, true),
        (SubCellIndex::LeftBottomBack, false),
        (SubCellIndex::LeftBottomFront, false),
        (SubCellIndex::RightTopBack, true),
        (SubCellIndex::RightTopFront, true),
        (SubCellIndex::RightBottomBack, false),
        (SubCellIndex::RightBottomFront, false),
    ], // TOP NEIGHBOUR
    [
        (SubCellIndex::RightBottomBack, false),
        (SubCellIndex::RightBottomFront, false),
        (SubCellIndex::RightTopBack, false),
        (SubCellIndex::RightTopFront, false),
        (SubCellIndex::LeftBottomBack, true),
        (SubCellIndex::LeftBottomFront, true),
        (SubCellIndex::LeftTopBack, true),
        (SubCellIndex::LeftTopFront, true),
    ], // LEFT NEIGHBOUR
    [
        (SubCellIndex::RightBottomBack, true),
        (SubCellIndex::RightBottomFront, true),
        (SubCellIndex::RightTopBack, true),
        (SubCellIndex::RightTopFront, true),
        (SubCellIndex::LeftBottomBack, false),
        (SubCellIndex::LeftBottomFront, false),
        (SubCellIndex::LeftTopBack, false),
        (SubCellIndex::LeftTopFront, false),
    ], // RIGHT NEIGHBOUR
];

/// @brief neighbour face table between sub cell
pub const FACE_RELATIONSHIP_TABLE: [[FaceIndex; 3]; SubCellIndex::COUNT] = [
    [FaceIndex::Back, FaceIndex::Bottom, FaceIndex::Left],
    [FaceIndex::Front, FaceIndex::Bottom, FaceIndex::Left],
    [FaceIndex::Back, FaceIndex::Top, FaceIndex::Left],
    [FaceIndex::Front, FaceIndex::Top, FaceIndex::Left],
    [FaceIndex::Back, FaceIndex::Bottom, FaceIndex::Right],
    [FaceIndex::Front, FaceIndex::Bottom, FaceIndex::Right],
    [FaceIndex::Back, FaceIndex::Top, FaceIndex::Right],
    [FaceIndex::Front, FaceIndex::Top, FaceIndex::Right],
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIter, EnumCount)]
pub enum SubFaceIndex {
    LeftUp = 0,
    RightUp = 1,
    LeftDown = 2,
    RightDown = 3,
}

//
//      2-------0------6
//     /.             /|
//    10.           11 |
//   /  2           /  3
//  /   .          /   |     ^ Y
// 3-------5------7    |     |
// |    0 . . 1 . |. . 4     --> X
// |   .          |   /     /
// 6  8           7  9     / z
// | .            | /     |/
// |.             |/
// 1-------4------5
//

/// @brief subcell的Face在父Cell的Face的位置。
/// 从Face的正面看去，SubFace(SubFaceIndex)的顺序
///   _______________
///  |       |      |
///  |   0   |  1   |
///  |_______|______|
///  |       |      |
///  |   2   |  3   |
///  |_______|______|
pub const SUB_FACE_TABLE: [[SubFaceIndex; 3]; SubCellIndex::COUNT] = [
    [
        SubFaceIndex::RightDown,
        SubFaceIndex::LeftDown,
        SubFaceIndex::LeftDown,
    ],
    [
        SubFaceIndex::LeftDown,
        SubFaceIndex::LeftUp,
        SubFaceIndex::RightDown,
    ],
    [
        SubFaceIndex::RightUp,
        SubFaceIndex::LeftUp,
        SubFaceIndex::LeftUp,
    ],
    [
        SubFaceIndex::LeftUp,
        SubFaceIndex::LeftDown,
        SubFaceIndex::RightUp,
    ],
    [
        SubFaceIndex::LeftDown,
        SubFaceIndex::RightDown,
        SubFaceIndex::RightDown,
    ],
    [
        SubFaceIndex::RightDown,
        SubFaceIndex::RightUp,
        SubFaceIndex::LeftDown,
    ],
    [
        SubFaceIndex::LeftUp,
        SubFaceIndex::RightUp,
        SubFaceIndex::RightUp,
    ],
    [
        SubFaceIndex::RightUp,
        SubFaceIndex::RightDown,
        SubFaceIndex::LeftUp,
    ],
];
