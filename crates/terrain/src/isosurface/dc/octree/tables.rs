/*
  USING Right HANDED SYSTEM like bevy engine

  如果没有做说明，默认排序为x,y,z，从小到大, 从右到左。
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumIter, EnumCount, FromRepr, Reflect)]
pub enum AxisType {
    XAxis = 0,
    YAxis = 1,
    ZAxis = 2,
}

const fn get_axis_value(
    x_axis_value: XAxisValue,
    y_axis_value: YAxisValue,
    z_axis_value: ZAxisValue,
) -> isize {
    (x_axis_value.bits() + (y_axis_value.bits() << 1) + (z_axis_value.bits() << 2)) as isize
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
    pub fn to_array(&self) -> [u8; 3] {
        match self {
            VertexIndex::X0Y0Z0 => [0, 0, 0],
            VertexIndex::X1Y0Z0 => [1, 0, 0],
            VertexIndex::X0Y1Z0 => [0, 1, 0],
            VertexIndex::X1Y1Z0 => [1, 1, 0],
            VertexIndex::X0Y0Z1 => [0, 0, 1],
            VertexIndex::X1Y0Z1 => [1, 0, 1],
            VertexIndex::X0Y1Z1 => [0, 1, 1],
            VertexIndex::X1Y1Z1 => [1, 1, 1],
        }
    }

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

/// 边连接的顶点
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

pub type SubNodeIndex = VertexIndex;

//  Node Point and Subnode Layout
//
//      (010)o--------------o(011)
//        /.             /|
//       / .            / |
//      /  .           /  |
//     /   .          /   |     ^ Y
//(110)o------------o(111)|     |
//    |(000)o. . . . |. . o(001)  --> X
//    |   .          |   /     /
//    |  .           |  /     /
//    | .            | /     z
//    |.             |/
// (100)o--------------o(101)
//
// 连接两个Node的Face，根据方向轴，找到分解的四个面的两侧的SubNodeIndex
// 在输入轴方向，从小到大，连接的SubNodeIndex， ==0表示在输入轴的左边， ==1表示在输入轴的右边
pub const FACES_SUBNODES_NEIGHBOUR_PAIRS: [[(SubNodeIndex, SubNodeIndex); 4]; AxisType::COUNT] = [
    // x axis
    [
        (SubNodeIndex::X1Y0Z0, SubNodeIndex::X0Y0Z0),
        (SubNodeIndex::X1Y1Z0, SubNodeIndex::X0Y1Z0),
        (SubNodeIndex::X1Y0Z1, SubNodeIndex::X0Y0Z1),
        (SubNodeIndex::X1Y1Z1, SubNodeIndex::X0Y1Z1),
    ],
    // y axis
    [
        (SubNodeIndex::X0Y1Z0, SubNodeIndex::X0Y0Z0),
        (SubNodeIndex::X1Y1Z0, SubNodeIndex::X1Y0Z0),
        (SubNodeIndex::X0Y1Z1, SubNodeIndex::X0Y0Z1),
        (SubNodeIndex::X1Y1Z1, SubNodeIndex::X1Y0Z1),
    ],
    // z axis
    [
        (SubNodeIndex::X0Y0Z1, SubNodeIndex::X0Y0Z0),
        (SubNodeIndex::X1Y0Z1, SubNodeIndex::X1Y0Z0),
        (SubNodeIndex::X0Y1Z1, SubNodeIndex::X0Y1Z0),
        (SubNodeIndex::X1Y1Z1, SubNodeIndex::X1Y1Z0),
    ],
];

//  Node Point and Subnode Layout
//
//      (010)o--------------o(011)
//        /.             /|
//       / .            / |
//      /  .           /  |     ^ Y
//     /   .          /   |     |
//(110)o-------------o(111)     |
//    |(000)o. . . . |. . o(001)  --> X
//    |   .          |   /     /
//    |  .           |  /     /
//    | .            | /     z
//    |.             |/
// (100)o--------------o(101)
//
// 在一个Node中，坐标轴垂直的内部的四个面，所对应的SubNodeIndex
// 根据输入轴，找到面相邻的四个SubNodeIndex
// 轴垂直穿过的Node中间内部的面, 负半轴的SubNodeIndex,在tuple的左边, 正半轴的SubNodeIndex,在tuple的右边
pub const SUBNODE_FACES_NEIGHBOUR_PAIRS: [[(SubNodeIndex, SubNodeIndex); 4]; AxisType::COUNT] = [
    // x axis
    [
        (SubNodeIndex::X0Y0Z0, SubNodeIndex::X1Y0Z0),
        (SubNodeIndex::X0Y1Z0, SubNodeIndex::X1Y1Z0),
        (SubNodeIndex::X0Y0Z1, SubNodeIndex::X1Y0Z1),
        (SubNodeIndex::X0Y1Z1, SubNodeIndex::X1Y1Z1),
    ],
    // y axis
    [
        (SubNodeIndex::X0Y0Z0, SubNodeIndex::X0Y1Z0),
        (SubNodeIndex::X1Y0Z0, SubNodeIndex::X1Y1Z0),
        (SubNodeIndex::X0Y0Z1, SubNodeIndex::X0Y1Z1),
        (SubNodeIndex::X1Y0Z1, SubNodeIndex::X1Y1Z1),
    ],
    // z axis
    [
        (SubNodeIndex::X0Y0Z0, SubNodeIndex::X0Y0Z1),
        (SubNodeIndex::X1Y0Z0, SubNodeIndex::X1Y0Z1),
        (SubNodeIndex::X0Y1Z0, SubNodeIndex::X0Y1Z1),
        (SubNodeIndex::X1Y1Z0, SubNodeIndex::X1Y1Z1),
    ],
];

//  Node Point and Subnode Layout
//
//      (010)o--------------o(011)
//        /.             /|
//       / .            / |
//      /  .           /  |
//     /   .          /   |         ^ Y
// (110)o--------------o(111)      |
//    |(000)o. . . . |. . o(001)     --> X
//    |   .          |   /        /
//    |  .           |  /        /
//    | .            | /        z
//    |.             |/
// (100)o--------------o(101)
//
// 根据边的朝向，获取负半轴和正半轴的四个SubNodeIndex.
// node index 排列顺序间 EdgeNodes的注释。
pub const SUBNODE_EDGES_NEIGHBOUR_PAIRS: [[(
    SubNodeIndex,
    SubNodeIndex,
    SubNodeIndex,
    SubNodeIndex,
); 2]; AxisType::COUNT] = [
    // two group indices is x axis
    [
        (
            SubNodeIndex::X0Y0Z0,
            SubNodeIndex::X0Y0Z1,
            SubNodeIndex::X0Y1Z0,
            SubNodeIndex::X0Y1Z1,
        ),
        (
            SubNodeIndex::X1Y0Z0,
            SubNodeIndex::X1Y0Z1,
            SubNodeIndex::X1Y1Z0,
            SubNodeIndex::X1Y1Z1,
        ),
    ],
    //  Node Point and Subnode Layout
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
            SubNodeIndex::X0Y0Z0,
            SubNodeIndex::X1Y0Z0,
            SubNodeIndex::X0Y0Z1,
            SubNodeIndex::X1Y0Z1,
        ),
        (
            SubNodeIndex::X0Y1Z0,
            SubNodeIndex::X1Y1Z0,
            SubNodeIndex::X0Y1Z1,
            SubNodeIndex::X1Y1Z1,
        ),
    ],
    //  Node Point and Subnode Layout
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
            SubNodeIndex::X1Y0Z0,
            SubNodeIndex::X0Y0Z0,
            SubNodeIndex::X1Y1Z0,
            SubNodeIndex::X0Y1Z0,
        ),
        (
            SubNodeIndex::X1Y0Z1,
            SubNodeIndex::X0Y0Z1,
            SubNodeIndex::X1Y1Z1,
            SubNodeIndex::X0Y1Z1,
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

//  Node Point and Subnode Layout
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
/// @brief Node neighbour table
///  找到一个subnode在FaceIndex的方向是否有邻居subnode,以及相邻的Subnode的位置
///
pub const NEIGHBOUR_ADDRESS_TABLE: [[(SubNodeIndex, bool); SubNodeIndex::COUNT]; FaceIndex::COUNT] = [
    // left
    [
        (SubNodeIndex::X1Y0Z0, false),
        (SubNodeIndex::X0Y0Z0, true),
        (SubNodeIndex::X1Y1Z0, false),
        (SubNodeIndex::X0Y1Z0, true),
        (SubNodeIndex::X1Y0Z1, false),
        (SubNodeIndex::X0Y0Z1, true),
        (SubNodeIndex::X1Y1Z1, false),
        (SubNodeIndex::X0Y1Z1, true),
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
        (SubNodeIndex::X1Y0Z0, true),
        (SubNodeIndex::X0Y0Z0, false),
        (SubNodeIndex::X1Y1Z0, true),
        (SubNodeIndex::X0Y1Z0, false),
        (SubNodeIndex::X1Y0Z1, true),
        (SubNodeIndex::X0Y0Z1, false),
        (SubNodeIndex::X1Y1Z1, true),
        (SubNodeIndex::X0Y1Z1, false),
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
        (SubNodeIndex::X0Y1Z0, false),
        (SubNodeIndex::X1Y1Z0, false),
        (SubNodeIndex::X0Y0Z0, true),
        (SubNodeIndex::X1Y0Z0, true),
        (SubNodeIndex::X0Y1Z1, false),
        (SubNodeIndex::X1Y1Z1, false),
        (SubNodeIndex::X0Y0Z1, true),
        (SubNodeIndex::X1Y0Z1, true),
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
        (SubNodeIndex::X0Y1Z0, true),
        (SubNodeIndex::X1Y1Z0, true),
        (SubNodeIndex::X0Y0Z0, false),
        (SubNodeIndex::X1Y0Z0, false),
        (SubNodeIndex::X0Y1Z1, true),
        (SubNodeIndex::X1Y1Z1, true),
        (SubNodeIndex::X0Y0Z1, false),
        (SubNodeIndex::X1Y0Z1, false),
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
        (SubNodeIndex::X0Y0Z1, false),
        (SubNodeIndex::X1Y0Z1, false),
        (SubNodeIndex::X0Y1Z1, false),
        (SubNodeIndex::X1Y1Z1, false),
        (SubNodeIndex::X0Y0Z0, true),
        (SubNodeIndex::X1Y0Z0, true),
        (SubNodeIndex::X0Y1Z0, true),
        (SubNodeIndex::X1Y1Z0, true),
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
        (SubNodeIndex::X0Y0Z1, true),
        (SubNodeIndex::X1Y0Z1, true),
        (SubNodeIndex::X0Y1Z1, true),
        (SubNodeIndex::X1Y1Z1, true),
        (SubNodeIndex::X0Y0Z0, false),
        (SubNodeIndex::X1Y0Z0, false),
        (SubNodeIndex::X0Y1Z0, false),
        (SubNodeIndex::X1Y1Z0, false),
    ],
];

pub const FACE_TO_SUB_EDGES_AXIS_TYPE: [[AxisType; 2]; AxisType::COUNT] = [
    // face is x axis
    [AxisType::YAxis, AxisType::ZAxis],
    // face is y axis
    [AxisType::XAxis, AxisType::ZAxis],
    // face is z axis
    [AxisType::XAxis, AxisType::YAxis],
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
// 垂直于轴的面上的四个边所相邻的SubNodeIndex
// face axis => 边的轴向 =》边的轴值 => 共享边的SubNodeIndex, 以及SubNodeIndex属于Face左侧还是右侧的Node。
// 需要注意，因为Face是连接两个Node的，所以AxisValue::Zero表示在右侧的Node，AxisValue::One表示在左侧的Node, 与正常值相反。
#[allow(clippy::type_complexity)]
pub const FACES_TO_SUB_EDGES_NODES: [[[[(SubNodeIndex, AxisValue); 4]; 2]; 2]; AxisType::COUNT] = [
    // 面是 x axis
    [
        // 边是 y axis
        [
            [
                // 1 0 5 4
                (SubNodeIndex::X1Y0Z0, AxisValue::Zero),
                (SubNodeIndex::X0Y0Z0, AxisValue::One),
                (SubNodeIndex::X1Y0Z1, AxisValue::Zero),
                (SubNodeIndex::X0Y0Z1, AxisValue::One),
            ],
            [
                (SubNodeIndex::X1Y1Z0, AxisValue::Zero),
                (SubNodeIndex::X0Y1Z0, AxisValue::One),
                (SubNodeIndex::X1Y1Z1, AxisValue::Zero),
                (SubNodeIndex::X0Y1Z1, AxisValue::One),
            ],
        ],
        // 边是 z axis
        [
            [
                // 0 1 2 3
                (SubNodeIndex::X0Y0Z0, AxisValue::One),
                (SubNodeIndex::X1Y0Z0, AxisValue::Zero),
                (SubNodeIndex::X0Y1Z0, AxisValue::One),
                (SubNodeIndex::X1Y1Z0, AxisValue::Zero),
            ],
            [
                (SubNodeIndex::X0Y0Z1, AxisValue::One),
                (SubNodeIndex::X1Y0Z1, AxisValue::Zero),
                (SubNodeIndex::X0Y1Z1, AxisValue::One),
                (SubNodeIndex::X1Y1Z1, AxisValue::Zero),
            ],
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
    //面是  y axis
    [
        //边是  x axis
        [
            [
                // 2 6 0 4
                (SubNodeIndex::X0Y1Z0, AxisValue::Zero),
                (SubNodeIndex::X0Y1Z1, AxisValue::Zero),
                (SubNodeIndex::X0Y0Z0, AxisValue::One),
                (SubNodeIndex::X0Y0Z1, AxisValue::One),
            ],
            [
                (SubNodeIndex::X1Y1Z0, AxisValue::Zero),
                (SubNodeIndex::X1Y1Z1, AxisValue::Zero),
                (SubNodeIndex::X1Y0Z0, AxisValue::One),
                (SubNodeIndex::X1Y0Z1, AxisValue::One),
            ],
        ],
        // 边是 z axis
        [
            [
                // 3 2 1 0
                (SubNodeIndex::X1Y1Z0, AxisValue::Zero),
                (SubNodeIndex::X0Y1Z0, AxisValue::Zero),
                (SubNodeIndex::X1Y0Z0, AxisValue::One),
                (SubNodeIndex::X0Y0Z0, AxisValue::One),
            ],
            [
                (SubNodeIndex::X1Y1Z1, AxisValue::Zero),
                (SubNodeIndex::X0Y1Z1, AxisValue::Zero),
                (SubNodeIndex::X1Y0Z1, AxisValue::One),
                (SubNodeIndex::X0Y0Z1, AxisValue::One),
            ],
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
    //面是 z axis
    [
        // 边是 x axis
        [
            [
                // 4 0 6 2
                (SubNodeIndex::X0Y0Z1, AxisValue::Zero),
                (SubNodeIndex::X0Y0Z0, AxisValue::One),
                (SubNodeIndex::X0Y1Z1, AxisValue::Zero),
                (SubNodeIndex::X0Y1Z0, AxisValue::One),
            ],
            [
                (SubNodeIndex::X1Y0Z1, AxisValue::Zero),
                (SubNodeIndex::X1Y0Z0, AxisValue::One),
                (SubNodeIndex::X1Y1Z1, AxisValue::Zero),
                (SubNodeIndex::X1Y1Z0, AxisValue::One),
            ],
        ],
        // 边是 y axis
        [
            [
                //  4 5 0 1
                (SubNodeIndex::X0Y0Z1, AxisValue::Zero),
                (SubNodeIndex::X1Y0Z1, AxisValue::Zero),
                (SubNodeIndex::X0Y0Z0, AxisValue::One),
                (SubNodeIndex::X1Y0Z0, AxisValue::One),
            ],
            [
                (SubNodeIndex::X0Y1Z1, AxisValue::Zero),
                (SubNodeIndex::X1Y1Z1, AxisValue::Zero),
                (SubNodeIndex::X0Y1Z0, AxisValue::One),
                (SubNodeIndex::X1Y1Z0, AxisValue::One),
            ],
        ],
    ],
];

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
// four edges axis in a face axis
pub const FACE_TO_EDGE_AXIS: [[AxisType; 4]; AxisType::COUNT] = [
    // x axis
    [
        AxisType::YAxis,
        AxisType::YAxis,
        AxisType::ZAxis,
        AxisType::ZAxis,
    ],
    // y axis
    [
        AxisType::XAxis,
        AxisType::XAxis,
        AxisType::ZAxis,
        AxisType::ZAxis,
    ],
    // z axis
    [
        AxisType::XAxis,
        AxisType::XAxis,
        AxisType::YAxis,
        AxisType::YAxis,
    ],
];

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
// 不同方向的边，切割为两个子边，每个子边的4个SubNodeIndex,
// 因为边是跨越4个Node的，所以SubNodeIndex是4个不同的Node的。
pub const SUBEDGE_NODES: [[[SubNodeIndex; 4]; 2]; AxisType::COUNT] = [
    // edge is x axis
    [
        // 负半轴
        [
            // 6 2 4 0
            SubNodeIndex::X0Y1Z1,
            SubNodeIndex::X0Y1Z0,
            SubNodeIndex::X0Y0Z1,
            SubNodeIndex::X0Y0Z0,
        ],
        // 正半轴
        [
            SubNodeIndex::X1Y1Z1,
            SubNodeIndex::X1Y1Z0,
            SubNodeIndex::X1Y0Z1,
            SubNodeIndex::X1Y0Z0,
        ],
    ],
    // edge is y axis
    [
        // 5 4 1 0
        // 负半轴
        [
            SubNodeIndex::X1Y0Z1,
            SubNodeIndex::X0Y0Z1,
            SubNodeIndex::X1Y0Z0,
            SubNodeIndex::X0Y0Z0,
        ],
        // 正半轴
        [
            SubNodeIndex::X1Y1Z1,
            SubNodeIndex::X0Y1Z1,
            SubNodeIndex::X1Y1Z0,
            SubNodeIndex::X0Y1Z0,
        ],
    ],
    // edge is z axis
    [
        // 2 3 0 1
        // 负半轴
        [
            SubNodeIndex::X0Y1Z0,
            SubNodeIndex::X1Y1Z0,
            SubNodeIndex::X0Y0Z0,
            SubNodeIndex::X1Y0Z0,
        ],
        // 正半轴
        [
            SubNodeIndex::X0Y1Z1,
            SubNodeIndex::X1Y1Z1,
            SubNodeIndex::X0Y0Z1,
            SubNodeIndex::X1Y0Z1,
        ],
    ],
];

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
// 因为边是跨越4个Node的，所以SubNodeIndex是4个不同的Node的。
pub const EDGE_NODES_VERTICES: [[[SubNodeIndex; 2]; 4]; AxisType::COUNT] = [
    // edge is x axis
    [
        // 负方向到正方向
        // 6 7
        [SubNodeIndex::X0Y1Z1, SubNodeIndex::X1Y1Z1],
        // 2 3
        [SubNodeIndex::X0Y1Z0, SubNodeIndex::X1Y1Z0],
        // 4 5
        [SubNodeIndex::X0Y0Z1, SubNodeIndex::X1Y0Z1],
        // 0 1
        [SubNodeIndex::X0Y0Z0, SubNodeIndex::X1Y0Z0],
    ],
    // edge is y axis
    [
        // 5 7
        [SubNodeIndex::X1Y0Z1, SubNodeIndex::X1Y1Z1],
        // 4 6
        [SubNodeIndex::X0Y0Z1, SubNodeIndex::X0Y1Z1],
        // 1 3
        [SubNodeIndex::X1Y0Z0, SubNodeIndex::X1Y1Z0],
        // 0 2
        [SubNodeIndex::X0Y0Z0, SubNodeIndex::X0Y1Z0],
    ],
    // edge is z axis
    [
        // 2 6
        [SubNodeIndex::X0Y1Z0, SubNodeIndex::X0Y1Z1],
        // 3 7
        [SubNodeIndex::X1Y1Z0, SubNodeIndex::X1Y1Z1],
        // 0 4
        [SubNodeIndex::X0Y0Z0, SubNodeIndex::X0Y0Z1],
        // 1 5
        [SubNodeIndex::X1Y0Z0, SubNodeIndex::X1Y0Z1],
    ],
];
