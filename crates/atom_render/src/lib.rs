#![deny(missing_docs)]

//! GPU 缓冲区工具库：提供共享（多数据共用一块 buffer）与 staged（CPU 回读）两类
//! buffer 封装，减少 buffer 申请次数并支持 GPU → CPU 数据回读。

/// 共享 buffer 封装：多组数据共用一块 storage/uniform buffer，支持 stride 对齐。
pub mod shared_buffer;
/// staged buffer 封装：GPU buffer + 可映射 CPU buffer 配对，支持 GPU → CPU 数据回读。
pub mod staged_buffer;
