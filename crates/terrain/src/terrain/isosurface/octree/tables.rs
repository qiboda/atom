/*
  USING Right HANDED SYSTEM like bevy engine

  如果没有做说明，默认排序为x,y,z，从小到大。
*/

/*

 Vertex and Edge Index Map

       2-------1------3
      /.             /|
     10.           11 |
    /  4           /  5
   /   .          /   |     ^ Y
  6-------3------7    |     |
  |    0 . . 0 . |. . 1     --> X
  |   .          |   /     /
  6  8           7  9     / z
  | .            | /     |/
  |.             |/
  4-------2------5

*/

use bevy::reflect::Reflect;
use strum::{EnumCount, FromRepr};
use strum_macros::EnumIter;

bitflags::bitflags! {
    /**
     *  If Vertex or edge or face is axis left/buttom/back, then the value is 0, other than 1.
     */
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct AxisValue: u8 {
        const Zero = 0b000;
        const One = 0b001;
    }
}

type XAxisValue = AxisValue;
type YAxisValue = AxisValue;
type ZAxisValue = AxisValue;

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct AxisType :u8 {
        const XAxis = 0b001;
        const YAxis = 0b010;
        const ZAxis = 0b100;
    }
}

impl AxisType {
    pub const COUNT: usize = 3;
}

const fn get_axis_value(
    x_axis_value: XAxisValue,
    y_axis_value: YAxisValue,
    z_axis_value: ZAxisValue,
) -> isize {
    (AxisType::XAxis.bits() * x_axis_value.bits()
        + AxisType::YAxis.bits() * y_axis_value.bits()
        + AxisType::ZAxis.bits() * z_axis_value.bits()) as isize
}

/**
 * Edge Index Map
 *
 * Axis is edge direction
 */
#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIter, EnumCount, Hash)]
pub enum EdgeIndex {
    #[allow(clippy::identity_op, clippy::erasing_op)]
    XAxisY0Z0 = 4 * 0 + 1 * 0 + 2 * 0,
    #[allow(clippy::identity_op, clippy::erasing_op)]
    XAxisY1Z0 = 4 * 0 + 1 * 1 + 2 * 0,
    #[allow(clippy::identity_op, clippy::erasing_op)]
    XAxisY0Z1 = 4 * 0 + 1 * 0 + 2 * 1,
    #[allow(clippy::identity_op, clippy::erasing_op)]
    XAxisY1Z1 = 4 * 0 + 1 * 1 + 2 * 1,

    #[allow(clippy::identity_op, clippy::erasing_op)]
    YAxisX0Z0 = 4 * 1 + 1 * 0 + 2 * 0,
    #[allow(clippy::identity_op, clippy::erasing_op)]
    YAxisX1Z0 = 4 * 1 + 1 * 1 + 2 * 0,
    #[allow(clippy::identity_op, clippy::erasing_op)]
    YAxisX0Z1 = 4 * 1 + 1 * 0 + 2 * 1,
    #[allow(clippy::identity_op, clippy::erasing_op)]
    YAxisX1Z1 = 4 * 1 + 1 * 1 + 2 * 1,

    #[allow(clippy::identity_op, clippy::erasing_op)]
    ZAxisX0Y0 = 4 * 2 + 1 * 0 + 2 * 0,
    #[allow(clippy::identity_op, clippy::erasing_op)]
    ZAxisX1Y0 = 4 * 2 + 1 * 1 + 2 * 0,
    #[allow(clippy::identity_op, clippy::erasing_op)]
    ZAxisX0Y1 = 4 * 2 + 1 * 0 + 2 * 1,
    #[allow(clippy::identity_op, clippy::erasing_op)]
    ZAxisX1Y1 = 4 * 2 + 1 * 1 + 2 * 1,
}

// x, y, z => xyz
#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIter, EnumCount, Hash)]
pub enum VertexIndex {
    X0Y0Z0 = get_axis_value(XAxisValue::Zero, YAxisValue::Zero, ZAxisValue::Zero),
    X1Y0Z0 = get_axis_value(XAxisValue::One, YAxisValue::Zero, ZAxisValue::Zero),
    X0Y1Z0 = get_axis_value(XAxisValue::Zero, YAxisValue::One, ZAxisValue::Zero),
    X1Y1Z0 = get_axis_value(XAxisValue::One, YAxisValue::One, ZAxisValue::Zero),
    X0Y0Z1 = get_axis_value(XAxisValue::Zero, YAxisValue::Zero, ZAxisValue::One),
    X1Y0Z1 = get_axis_value(XAxisValue::One, YAxisValue::Zero, ZAxisValue::One),
    X0Y1Z1 = get_axis_value(XAxisValue::Zero, YAxisValue::One, ZAxisValue::One),
    X1Y1Z1 = get_axis_value(XAxisValue::One, YAxisValue::One, ZAxisValue::One),
}

impl VertexIndex {
    pub fn from_repr(repr: usize) -> Option<Self> {
        match repr {
            0 => Some(VertexIndex::X0Y0Z0),
            1 => Some(VertexIndex::X1Y0Z0),
            2 => Some(VertexIndex::X0Y1Z0),
            3 => Some(VertexIndex::X1Y1Z0),
            4 => Some(VertexIndex::X0Y0Z1),
            5 => Some(VertexIndex::X1Y0Z1),
            6 => Some(VertexIndex::X0Y1Z1),
            7 => Some(VertexIndex::X1Y1Z1),
            _ => None,
        }
    }
}

pub const EDGE_VERTEX_PAIRS: [[VertexIndex; 2]; EdgeIndex::COUNT] = [
    // x axis
    [VertexIndex::X0Y0Z0, VertexIndex::X1Y0Z0],
    [VertexIndex::X0Y1Z0, VertexIndex::X1Y1Z0],
    [VertexIndex::X0Y0Z1, VertexIndex::X1Y0Z1],
    [VertexIndex::X0Y1Z1, VertexIndex::X1Y1Z1],
    // y axis
    [VertexIndex::X0Y0Z0, VertexIndex::X0Y1Z0],
    [VertexIndex::X1Y0Z0, VertexIndex::X1Y1Z0],
    [VertexIndex::X0Y0Z1, VertexIndex::X0Y1Z1],
    [VertexIndex::X1Y0Z1, VertexIndex::X1Y1Z1],
    // z axis
    [VertexIndex::X0Y0Z0, VertexIndex::X0Y0Z1],
    [VertexIndex::X1Y0Z0, VertexIndex::X1Y0Z1],
    [VertexIndex::X0Y1Z0, VertexIndex::X0Y1Z1],
    [VertexIndex::X1Y1Z0, VertexIndex::X1Y1Z1],
];

pub type SubCellIndex = VertexIndex;

//  Cell Point and Subcell Layout
//
//      (010)o--------------o(011)
//        /.             /|
//       / .            / |
//      /  .           /  |
//     /   .          /   |     ^ Y
// (110)o--------------o(111) |     |
//    |(000)o. . . . |. . o(001)  --> X
//    |   .          |   /     /
//    |  .           |  /     /
//    | .            | /     z
//    |.             |/
// (100)o--------------o(101)
//
pub const AXIS_VALUE_SUBCELL_INDICES: [[[SubCellIndex; 4]; 2]; AxisType::COUNT] = [
    // x axis
    [
        // == 0
        [
            SubCellIndex::X0Y0Z0,
            SubCellIndex::X0Y1Z0,
            SubCellIndex::X0Y0Z1,
            SubCellIndex::X0Y1Z1,
        ],
        // == 1
        [
            SubCellIndex::X1Y0Z0,
            SubCellIndex::X1Y1Z0,
            SubCellIndex::X1Y0Z1,
            SubCellIndex::X1Y1Z1,
        ],
    ],
    // y axis
    [
        [
            SubCellIndex::X0Y0Z0,
            SubCellIndex::X1Y0Z0,
            SubCellIndex::X0Y0Z1,
            SubCellIndex::X1Y0Z1,
        ],
        [
            SubCellIndex::X0Y1Z0,
            SubCellIndex::X1Y1Z0,
            SubCellIndex::X0Y1Z1,
            SubCellIndex::X1Y1Z1,
        ],
    ],
    // z axis
    [
        [
            SubCellIndex::X0Y0Z0,
            SubCellIndex::X1Y0Z0,
            SubCellIndex::X0Y1Z0,
            SubCellIndex::X1Y1Z0,
        ],
        [
            SubCellIndex::X0Y0Z1,
            SubCellIndex::X1Y0Z1,
            SubCellIndex::X0Y1Z1,
            SubCellIndex::X1Y1Z1,
        ],
    ],
];

//  Cell Point and Subcell Layout
//
//      (010)o--------------o(011)
//        /.             /|
//       / .            / |
//      /  .           /  |
//     /   .          /   |     ^ Y
// (110)o--------------o(111) |     |
//    |(000)o. . . . |. . o(001)  --> X
//    |   .          |   /     /
//    |  .           |  /     /
//    | .            | /     z
//    |.             |/
// (100)o--------------o(101)
//
pub const SUBCELL_FACES_NEIGHBOUR_PAIRS: [[(SubCellIndex, SubCellIndex); 4]; AxisType::COUNT] = [
    [
        (SubCellIndex::X0Y0Z0, SubCellIndex::X1Y0Z0),
        (SubCellIndex::X0Y1Z0, SubCellIndex::X1Y1Z0),
        (SubCellIndex::X0Y0Z1, SubCellIndex::X1Y0Z1),
        (SubCellIndex::X0Y1Z1, SubCellIndex::X1Y1Z1),
    ],
    [
        (SubCellIndex::X0Y0Z0, SubCellIndex::X0Y1Z0),
        (SubCellIndex::X1Y0Z0, SubCellIndex::X1Y1Z0),
        (SubCellIndex::X0Y0Z1, SubCellIndex::X0Y1Z1),
        (SubCellIndex::X1Y0Z1, SubCellIndex::X1Y1Z1),
    ],
    [
        (SubCellIndex::X0Y0Z0, SubCellIndex::X0Y0Z1),
        (SubCellIndex::X1Y0Z0, SubCellIndex::X1Y0Z1),
        (SubCellIndex::X0Y1Z0, SubCellIndex::X0Y1Z1),
        (SubCellIndex::X1Y1Z0, SubCellIndex::X1Y1Z1),
    ],
];

//  Cell Point and Subcell Layout
//
//      (010)o--------------o(011)
//        /.             /|
//       / .            / |
//      /  .           /  |
//     /   .          /   |     ^ Y
// (110)o--------------o(111) |     |
//    |(000)o. . . . |. . o(001)  --> X
//    |   .          |   /     /
//    |  .           |  /     /
//    | .            | /     z
//    |.             |/
// (100)o--------------o(101)
//
// 出于dual contouring的需要，便于三角化，顶点按照从外部看,从左下角按逆时针的顺序排列
pub const SUBCELL_EDGES_NEIGHBOUR_PAIRS: [[(
    SubCellIndex,
    SubCellIndex,
    SubCellIndex,
    SubCellIndex,
); 2]; AxisType::COUNT] = [
    // two group indices is x axis
    [
        (
            SubCellIndex::X0Y0Z0,
            SubCellIndex::X0Y0Z1,
            SubCellIndex::X0Y1Z1,
            SubCellIndex::X0Y1Z0,
        ),
        (
            SubCellIndex::X1Y0Z1,
            SubCellIndex::X1Y0Z0,
            SubCellIndex::X1Y1Z0,
            SubCellIndex::X1Y1Z1,
        ),
    ],
    //  Cell Point and Subcell Layout
    //
    //      (010)o--------------o(011)
    //        /.             /|
    //       / .            / |
    //      /  .           /  |
    //     /   .          /   |     ^ Y
    // (110)o--------------o(111) |     |
    //    |(000)o. . . . |. . o(001)  --> X
    //    |   .          |   /     /
    //    |  .           |  /     /
    //    | .            | /     z
    //    |.             |/
    // (100)o--------------o(101)
    //
    // two group indices is y axis
    [
        (
            SubCellIndex::X0Y0Z0,
            SubCellIndex::X1Y0Z0,
            SubCellIndex::X1Y0Z1,
            SubCellIndex::X0Y0Z1,
        ),
        (
            SubCellIndex::X1Y1Z0,
            SubCellIndex::X1Y1Z1,
            SubCellIndex::X1Y1Z0,
            SubCellIndex::X0Y1Z0,
        ),
    ],
    //  Cell Point and Subcell Layout
    //
    //      (010)o--------------o(011)
    //        /.             /|
    //       / .            / |
    //      /  .           /  |
    //     /   .          /   |     ^ Y
    // (110)o--------------o(111) |     |
    //    |(000)o. . . . |. . o(001)  --> X
    //    |   .          |   /     /
    //    |  .           |  /     /
    //    | .            | /     z
    //    |.             |/
    // (100)o--------------o(101)
    //
    [
        (
            SubCellIndex::X1Y0Z0,
            SubCellIndex::X0Y0Z0,
            SubCellIndex::X0Y1Z0,
            SubCellIndex::X1Y1Z0,
        ),
        (
            SubCellIndex::X0Y0Z1,
            SubCellIndex::X1Y0Z1,
            SubCellIndex::X1Y1Z1,
            SubCellIndex::X0Y1Z1,
        ),
    ],
];

//  Face Index Layout
//
//         o--------------o
//        /.             /|
//       / .            / |
//      /  .           /  |
//     /   .          /   |     ^ Y
//    o--------------o    |     |
//    |    o. . . . . |. . o      --> X
//    |   .          |   /     /
//    |  .           |  /     /
//    | .            | /     z
//    |.             |/
//    o--------------o
//
// order is from x to y to z
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumIter, EnumCount, FromRepr, Reflect)]
pub enum FaceIndex {
    Left = 0,
    Right = 1,
    Bottom = 2,
    Top = 3,
    Back = 4,
    Front = 5,
}

pub const AXIS_FACE_INDEX_PAIR: [(FaceIndex, FaceIndex); AxisType::COUNT] = [
    (FaceIndex::Right, FaceIndex::Left),
    (FaceIndex::Top, FaceIndex::Bottom),
    (FaceIndex::Front, FaceIndex::Back),
];

pub const FACE_INDEX_TWINS: [FaceIndex; FaceIndex::COUNT] = [
    FaceIndex::Right,
    FaceIndex::Left,
    FaceIndex::Top,
    FaceIndex::Bottom,
    FaceIndex::Front,
    FaceIndex::Back,
];

//  Cell Point and Subcell Layout
//
//      (2)o--------------o(3)
//        /.             /|
//       / .            / |
//      /  .           /  |
//     /   .          /   |     ^ Y
// (6)o--------------o(7) |     |
//    | (0)o. . . . . |. . o(1)  --> X
//    |   .          |   /     /
//    |  .           |  /     /
//    | .            | /     z
//    |.             |/
// (4)o--------------o(5)
//
//
/// @brief Cell neighbour table
///  找到一个subcell在FaceIndex的方向是否有邻居subcell,以及相邻的Subcell的位置
///
pub const NEIGHBOUR_ADDRESS_TABLE: [[(SubCellIndex, bool); SubCellIndex::COUNT]; FaceIndex::COUNT] = [
    // left
    [
        (SubCellIndex::X1Y0Z0, false),
        (SubCellIndex::X0Y0Z0, true),
        (SubCellIndex::X1Y1Z0, false),
        (SubCellIndex::X0Y1Z0, true),
        (SubCellIndex::X1Y0Z1, false),
        (SubCellIndex::X0Y0Z1, true),
        (SubCellIndex::X1Y1Z1, false),
        (SubCellIndex::X0Y1Z1, true),
    ],
    //      (2)o--------------o(3)
    //        /.             /|
    //       / .            / |
    //      /  .           /  |
    //     /   .          /   |     ^ Y
    // (6)o--------------o(7) |     |
    //    | (0)o. . . . . |. . o(1)  --> X
    //    |   .          |   /     /
    //    |  .           |  /     /
    //    | .            | /     z
    //    |.             |/
    // (4)o--------------o(5)
    //
    // right
    [
        (SubCellIndex::X1Y0Z0, true),
        (SubCellIndex::X0Y0Z0, false),
        (SubCellIndex::X1Y1Z0, true),
        (SubCellIndex::X0Y1Z0, false),
        (SubCellIndex::X1Y0Z1, true),
        (SubCellIndex::X0Y0Z1, false),
        (SubCellIndex::X1Y1Z1, true),
        (SubCellIndex::X0Y1Z1, false),
    ],
    //      (2)o--------------o(3)
    //        /.             /|
    //       / .            / |
    //      /  .           /  |
    //     /   .          /   |     ^ Y
    // (6)o--------------o(7) |     |
    //    | (0)o. . . . . |. . o(1)  --> X
    //    |   .          |   /     /
    //    |  .           |  /     /
    //    | .            | /     z
    //    |.             |/
    // (4)o--------------o(5)
    //
    // Bottom
    [
        (SubCellIndex::X0Y1Z0, false),
        (SubCellIndex::X1Y1Z0, false),
        (SubCellIndex::X0Y0Z0, true),
        (SubCellIndex::X1Y0Z0, true),
        (SubCellIndex::X0Y1Z1, false),
        (SubCellIndex::X1Y1Z1, false),
        (SubCellIndex::X0Y0Z1, true),
        (SubCellIndex::X1Y0Z1, true),
    ],
    //      (2)o--------------o(3)
    //        /.             /|
    //       / .            / |
    //      /  .           /  |
    //     /   .          /   |     ^ Y
    // (6)o--------------o(7) |     |
    //    | (0)o. . . . . |. . o(1)  --> X
    //    |   .          |   /     /
    //    |  .           |  /     /
    //    | .            | /     z
    //    |.             |/
    // (4)o--------------o(5)
    //
    // Top
    [
        (SubCellIndex::X0Y1Z0, true),
        (SubCellIndex::X1Y1Z0, true),
        (SubCellIndex::X0Y0Z0, false),
        (SubCellIndex::X1Y0Z0, false),
        (SubCellIndex::X0Y1Z1, true),
        (SubCellIndex::X1Y1Z1, true),
        (SubCellIndex::X0Y0Z1, false),
        (SubCellIndex::X1Y0Z1, false),
    ],
    //      (2)o--------------o(3)
    //        /.             /|
    //       / .            / |
    //      /  .           /  |
    //     /   .          /   |     ^ Y
    // (6)o--------------o(7) |     |
    //    | (0)o. . . . . |. . o(1)  --> X
    //    |   .          |   /     /
    //    |  .           |  /     /
    //    | .            | /     z
    //    |.             |/
    // (4)o--------------o(5)
    //
    // Back
    [
        (SubCellIndex::X0Y0Z1, false),
        (SubCellIndex::X1Y0Z1, false),
        (SubCellIndex::X0Y1Z1, false),
        (SubCellIndex::X1Y1Z1, false),
        (SubCellIndex::X0Y0Z0, true),
        (SubCellIndex::X1Y0Z0, true),
        (SubCellIndex::X0Y1Z0, true),
        (SubCellIndex::X1Y1Z0, true),
    ],
    //      (2)o--------------o(3)
    //        /.             /|
    //       / .            / |
    //      /  .           /  |
    //     /   .          /   |     ^ Y
    // (6)o--------------o(7) |     |
    //    | (0)o. . . . . |. . o(1)  --> X
    //    |   .          |   /     /
    //    |  .           |  /     /
    //    | .            | /     z
    //    |.             |/
    // (4)o--------------o(5)
    //
    // Front
    [
        (SubCellIndex::X0Y0Z1, true),
        (SubCellIndex::X1Y0Z1, true),
        (SubCellIndex::X0Y1Z1, true),
        (SubCellIndex::X1Y1Z1, true),
        (SubCellIndex::X0Y0Z0, false),
        (SubCellIndex::X1Y0Z0, false),
        (SubCellIndex::X0Y1Z0, false),
        (SubCellIndex::X1Y1Z0, false),
    ],
];

//      (2)o--------------o(3)
//        /.             /|                    z ---------->
//       / .            / |                  |--------------|  /\
//      /  .           /  |                  |       |      |  |
//     /   .          /   |     ^ Y          |       1      |  |
// (6)o--------------o(7) |     |            |- -1 --x-- 1--|  |
//    | (0)o. . . . . |. . o(1)  --> X       |      -1      |  |
//    |   .          |   /     /             |       |      |  |
//    |  .           |  /     /              |--------------|  x
//    | .            | /     z
//    |.             |/
// (4)o--------------o(5)
//
// four subcells shared edges on one face
pub const FACES_EDGES_SUBCELLS: [[[SubCellIndex; 4]; 4]; AxisType::COUNT] = [
    // x axis
    [
        // y axis  -1
        [
            SubCellIndex::X0Y0Z0,
            SubCellIndex::X0Y0Z1,
            SubCellIndex::X0Y1Z1,
            SubCellIndex::X0Y1Z0,
        ],
        // y axis  1
        [
            SubCellIndex::X1Y0Z1,
            SubCellIndex::X1Y0Z0,
            SubCellIndex::X1Y1Z0,
            SubCellIndex::X1Y1Z1,
        ],
        // z axis  -1
        [
            SubCellIndex::X0Y0Z0,
            SubCellIndex::X0Y1Z0,
            SubCellIndex::X0Y1Z1,
            SubCellIndex::X0Y0Z1,
        ],
        // z axis  1
        [
            SubCellIndex::X1Y0Z1,
            SubCellIndex::X1Y1Z1,
            SubCellIndex::X1Y1Z0,
            SubCellIndex::X1Y0Z0,
        ],
    ],
    //      [2]o--------------o[3]
    //        /.             /|
    //       / .            / |
    //      /  .           /  |
    //     /   .          /   |     ^ Y
    // [6]o--------------o[7] |     |
    //    | [0]o. . . . . |. . o[1]  --> X
    //    |   .          |   /     /
    //    |  .           |  /     /
    //    | .            | /     z
    //    |.             |/
    // [4]o--------------o[5]
    //
    // y axis
    [
        [
            SubCellIndex::X0Y0Z0,
            SubCellIndex::X1Y0Z0,
            SubCellIndex::X1Y0Z1,
            SubCellIndex::X0Y0Z1,
        ],
        [
            SubCellIndex::X0Y1Z1,
            SubCellIndex::X1Y1Z1,
            SubCellIndex::X1Y1Z0,
            SubCellIndex::X0Y1Z0,
        ],
        [
            SubCellIndex::X0Y0Z0,
            SubCellIndex::X0Y0Z1,
            SubCellIndex::X0Y1Z1,
            SubCellIndex::X0Y1Z0,
        ],
        [
            SubCellIndex::X1Y0Z1,
            SubCellIndex::X1Y0Z0,
            SubCellIndex::X1Y1Z0,
            SubCellIndex::X1Y1Z1,
        ],
    ],
    //      [2]o--------------o[3]
    //        /.             /|
    //       / .            / |
    //      /  .           /  |
    //     /   .          /   |     ^ Y
    // [6]o--------------o[7] |     |
    //    | [0]o. . . . . |. . o[1]  --> X
    //    |   .          |   /     /
    //    |  .           |  /     /
    //    | .            | /     z
    //    |.             |/
    // [4]o--------------o[5]
    //
    // z axis
    [
        [
            SubCellIndex::X1Y0Z0,
            SubCellIndex::X0Y0Z0,
            SubCellIndex::X0Y1Z0,
            SubCellIndex::X1Y1Z0,
        ],
        [
            SubCellIndex::X0Y0Z1,
            SubCellIndex::X1Y0Z1,
            SubCellIndex::X1Y1Z1,
            SubCellIndex::X0Y1Z1,
        ],
        [
            SubCellIndex::X0Y0Z0,
            SubCellIndex::X0Y1Z0,
            SubCellIndex::X0Y1Z1,
            SubCellIndex::X0Y0Z1,
        ],
        [
            SubCellIndex::X1Y0Z1,
            SubCellIndex::X1Y1Z1,
            SubCellIndex::X1Y1Z0,
            SubCellIndex::X1Y0Z0,
        ],
    ],
];
