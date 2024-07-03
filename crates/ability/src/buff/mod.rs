/// buff
/// buff 会有一个时长。如果是瞬发的呢？
/// 开始和结束的时候，执行effect graph
/// looper 的时候, 也触发effect graph
///
/// buff 有一个特定的Entry。接受各种特定的事件。执行对应的逻辑。
/// 所以，buff 只会有一个Effect Graph.
///
/// 如此，与技能十分类似，仅仅是多了一个Buff Time和Entry不同。
///
pub mod bundle;
pub mod state;
pub mod event;
pub mod layertag;
pub mod node;
pub mod plugin;
pub mod timer;
pub mod layer;
