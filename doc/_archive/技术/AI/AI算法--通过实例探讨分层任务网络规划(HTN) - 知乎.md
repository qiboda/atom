一篇文章搞懂hierarchical task network(HTN)-通过实例探讨分层任务网络规划（译）

Game AI Pro  
Exploring HTN Planners through Example by Troy Humphreys

**Introduction**

**介绍**

作为程序员，我们可能会发现自己总是在寻找“更好的解决方案”来解决我们遇到的任何问题，这个解决方案可能是更好的性能、可维护性或可用性。而且只有在我们实现这些解决方案之后，我们才能理解其中的一些细微差别。通常，这些细微差别可能是决定我们采用何种解决方案的决定性因素。

在AI开发中，一个普遍需要解决的问题是行为选择。这个问题有很多解决方案，比如FSM有限状态机、行为树、基于效用的选择、神经网络和规划器。这篇文章通过利用在开发过程中可能遇到的现实世界中的项目例子来探究一个类型规划器的细微差别，这类规划器称为分层任务网络(HTN)规划器。

像HTN这样的规划器架构，是将一个问题作为输入，并提供一系列解决问题的步骤。在HTN术语中，这一系列步骤称为一个计划(plan)。HTN相对于其他规划器的独特之处在于，它允许我们把问题表现为一个非常高级（抽象）的计划(plan)，通过它的规划过程，递归地把这个任务分解成更小的任务。当这个过程完成时，剩下生成的是一组表示计划(plan)的原子任务。

把高层次（抽象）的任务分解成更小的任务是解决很多问题的一种很自然的方法。在我们的例子中，问题只是“弄清楚要去做什么”。HTNs具有高度的模块化和快速的运行时执行，是一种很有吸引力的解决方案。对于那些熟悉行为树的人来说，这些好处也很熟悉。可是与行为树不同的是，HTN的规划器能够对可能的行为产生的影响进行推理。这种对未来推理的能力让HTN的规划器在描述行为时具有令人难以置信的表现力。

有许多不同的系统用HTN来规划\[Erol 95\]。我们将探究的是我们在《变形金刚:塞伯坦之秋》(HighMoon 12)中使用过的系统，它是基于一个总顺序正向的分解规划器。 下面的示例将通过使用一个简化和虚构的例子，介绍我们在开发过程中遇到的一些挑战和获得的好处。

在我们的例子中，我们将使用一个叫作Trunk Thumper的巨魔NPC。设计师最初的描述是，他是一个巨大的、令人讨厌的、笨拙的巨魔，它在众多的桥梁上巡逻，并用一根巨大的树干攻击路过的敌人。就像通常会遇到的情况一样，这种设计一定会被改动。

**Building Blocks of HTN**

**HTN的构建模块**runner. The world state is updated by the NPC’s sensors and by the

在构建Trunk Thumper的行为(behavior)之前，有必要重温一下HTN的基本构建模块，这样您就可以了解它是如何工作的。在我们的例子中Trunk Thumper NPC拥有一个规划器，这个规划器使用一个域(domain)和世界状态(world state)构建称为计划(plan)的任务序列。这个计划将由Trunk Thumper的计划执行器(plan runner)执行。世界状态(world state)由NPC的传感器(sensors)和被计划执行器(plan runner)成功执行的任务来更新。系统示意图如图12.1所示。

**The World State**

**世界状态**

与任何类型的行为算法一样，HTN同样需要某种类型的知识表示来描述当前的问题空间。在我们的Trunk Thumper例子中，这个表示就是描述他对这个世界和他自己的了解。其他类型的行为算法可能查询世界中不同对象的实际状态。例如，查询对象的位置或生命值。但使用HTN，这些信息需要被编码成它能理解的东西，称为世界状态(world state)。世界状态本质上是一个用来描述HTN将会推理出来的属性向量集合。下面是一些简单的伪代码。

```
enum EHtnWorldStateProperties
{
WsEnemyRange,
WsHealth,
WsIsTired,
…
}
enum EEnemyRange
{
MeleeRange,
ViewRange,
OutOfRange,
…
}
vector<byte> CurrentWorldState;
EEnemyRange currentRange = CurrentWorldState[WsEnemyRange];
CurrentWorldState[WsEnemyRange] = MeleeRange;
```

从伪代码中可以看到，世界状态(world state)可以是一个数组或由枚举索引的向量，比如EhtnWorldStateProperties

世界状态(world state)中的每个字段都可以有自己的一组值。就WsIsTired字段来说，字节可以表示布尔值0和1。使用WsEnemyRange字段时，使用枚举EEnemyRange里面的值。需要注意的是，世界状态(world state)只需要表示出HTN做出决策所需要的字段。这就是为何WsEnemyRange字段使用抽象值而不是实际范围值表示的原因。（因为通过抽象值做出决策，如果需要的是范围值来做出决策，改造byte\[\]为数据的world state ，是否可以提供使用范围值？）世界状态(world state)的目标并不是代表游戏中所有可能对象的所有可能状态。它只需要表示出满足我们的规划器(planner)做出决策的问题域。当然，对于我们的示例来说，这意味着它只需要表示 Trunk Thumper 做出决策所需的内容。

![[atom/技术/AI/attachmen/137312a5a05b80eb37f9a5daed6fba2a_MD5.jpg]]

图 12.1 HTN系统概述

**Sensors**

**传感器(Sensors)**

如果您还记得的话，HTN输出一个计划(plan)或任务(task)序列。这些任务(task)将在在执行过程中对世界状态(world state)产生影响。不过，也有一些外部的影响会影响世界状态(world state)比如玩家或其他npc 。例如，敌人和巨魔都可以影响世界状态(world state)的WsEnemyRange属性。巨魔执行移动的任务可以更新此属性。不过HTN规划器(planner)中没有什么可以处理敌人移动产生的变化。

这些变化可以通过许多不同的方式转化为世界状态(world state)。一种比较好的方法是使用简单传感器系统(simple sensor system)来管理一组时间切片传感器。(把传感器的更新计算分散到多个帧来完成？降低每一帧的运算量)。每个传感器(sensor)可以管理不同的世界状态(world state)属性。一些不同传感器的例子包括视觉、听觉、范围和生命值传感器。这些传感器的工作原理与任何其他人工智能系统相同，只是增加了一个步骤，即把它们的信息编码成我们的HTN能够理解的世界状态(world state)。

**Primitive Tasks**

**原子任务**

正如我们之前提到的，一个HTN是由一组任务(task)组成的。用于构建HTN的任务(task)有两种类型，称为复合任务(compound task)和原子任务(primitive task)。原子任务(primitive task)表示可以由我们的NPC执行的单个步骤。在我们的Trunk Thumper例子中，拔起一棵树或用树干重击进行攻击都是原子任务(primitive task)的例子。一组原子任务(primitive task)是我们最终从HTN中得到的计划(plan)。原子任务(primitive task)由一个操作符(operator)和一组影响(effect)和条件(condition)组成。

为了执行一个原子任务(primitive task)，它的条件集必须是合法的。这允许任务的实现者确保任务的运行满足了正确的条件。需要注意的是，原子任务(primitive task)的条件不是HTN实现的必要条件。但是，建议使用这些条件来减少在HTN高层次中的冗余检查。此外，这样做可以避免在一些不得不做检查的地方可能存在潜在的bug。

原子任务(primitive task)的影响(effect)描述了任务的成功将如何影响NPC的世界状态(world state)。例如，任务DoTrunkSlam执行巨魔的树干近战攻击，导致巨魔变得疲惫。任务DoTrunkSlam的影响(effect)就是我们描述这个结果的方式。这允许HTN推理出前面提到的“未来”。由于“变得疲惫”的影响被表现出来，我们的Trunk Thumper能够更好地决定在任务DoTrunkSlam之后该做什么，或者是否值得这样做。

操作符(operator)表示一个NPC可以执行的原子操作。这可能听起来完全像原子任务(primitive task)本身。不同之处在于，原子任务(primitive task)及它的影响(effect)和条件(condition)描述了操作符(operator)对我们构建的HTN的意义。

我们以SprintToEnemy和WalkToNextBridge这两个任务为例。这两个任务都使用MoveTo操作符(operator)，但是这两个任务以不同的方式更改NPC的状态。当SprintToEnemy任务成功完成时，我们的NPC会攻击敌人并感到疲劳，这由任务的影响(effect)指定。WalkToNextBridge任务的影响(effect)会将NPC的位置设置为桥的位置，同时他就会有点无聊。正如你所看到的，我们可以使用同一个操作符(operator)，但在我们的网络中描述了它的两种不同用途。下面是SprintToEnemy任务和WalkToNextBridge任务作为例子用来描述一个原子任务(primitive task)的记法。

```
Primitive Task [TaskName(term1, term2,...)]
Preconditions [Condition1, Condition2, …]//optional
Operator [OperatorName(term1, term2,...)]
 Effects [WorldState op value, WorldState = value, WorldState += value]//optional
----------------------------------
Primitive Task [SprintToEnemy]
Preconditions [WsHasEnemy == true]
Operator [NavigateTo(EnemyLoc, Speed_Fast)]
 Effects [WsLocation = EnemyLoc, WsIsTired = true]
------------------------------
Primitive Task [WalkToNextBridge]
Operator [NavigateTo(BridgeLoc, Speed_Slow)]
 Effects [WsLocation = BridgeLoc, WsBored += 1]
------------------------------
```

**Compound Tasks**

**复合任务**

复合任务(Compound task)是HTN具有“分层”特性的地方。你可以把复合任务(Compound task)看作是一个高级任务，它有多种方式来完成。以Trunk Thumper为例，他可能有AttackEnemy任务。我们的Thumper可能有不同的方式来完成这项任务。如果他接近一根树干，他可以跑向他的目标，用它作为近战武器“thump”他的敌人。如果没有树干，他可以从地上拿起大石头，扔向我们的敌人。如果条件满足，他可能有许多其他方法。

为了确定我们采用哪种方法来完成一个复合任务，我们需要选择正确的方法(method)。方法(method)由一组条件和任务组成。为了使该方法成为可选择的方法，结合世界状态(world state)对条件(condition)进行了验证。任务集或子任务表示该方法(method)的处理方式。这个子任务集可以由基本任务和复合任务组成。将复合任务转化为其他复合任务的方法的能力是HTN具有分层特性的地方。下面是一个例子，我们将使用它来描述一个复合任务。

```
Compound Task [TaskName(term1, term2,...)]
Method 0 [Condition1, Condition2,...]
 Subtasks [task1(term1, term2,...). task2(term1, term2,...),...]
Method 1 [Condition1, Condition2,...]
 Subtasks [task1(term1, term2,...). task2(term1, term2,...),...]
```

在前面的例子中，使用树干作为近战武器和投掷石块都是攻击敌人复合任务的方法。我们决定使用哪种方法的条件取决于巨魔是否有树干。下面是一个使用上面的记法来标记AttackEnemy任务的例子。

```
Compound Task [AttackEnemy]
Method 0 [WsHasTreeTrunk == true]
 Subtasks [NavigateTo(EnemyLoc). DoTrunkSlam()]
Method 1 [WsHasTreeTrunk == false]
 Subtasks [LiftBoulderFromGround(). ThrowBoulderAt(EnemyLoc)]
```

通过理解复合任务是如何工作的，就很容易想象我们是如何拥有一个大的层次结构的，它可以从一个BeTrunkThumper复合任务开始，然后分解成一组更小的任务——每个任务再分解成更小的任务，以此类推。这就是HTN如何形成描述巨魔NPC行为的层次结构。

必须理解的是，复合任务实际上只是一组方法(method)的容器，这些方法(method)每一个都是用来表示完成某个高级任务的不同方式。在计划(plan)执行期间没有运行复合任务代码。

**Putting Together an HTN Domain**

**组合成一个HTN域**

现在我们已经对HTN的主要构建块有了一个概述，我们可以为Trunk Thumper构建一个简单的域来说明它是如何工作的。域(domain)是用来描述整个任务层次结构的术语。正如我们之前提到的，我们的巨魔有许多桥，他积极地巡逻和用一根大树干攻击敌人。我们从一个名为BeTrunkThumper的复合任务开始。这个根任务封装了作为一个Trunk Thumper的“主要思想”。

```
Compound Task [BeTrunkThumper]
Method [WsCanSeeEnemy == true]
 Subtasks [NavigateToEnemy(), DoTrunkSlam()]
Method [true]
 Subtasks [ChooseBridgeToCheck(), NavigateToBridge(), CheckBridge()]
```

从这个根复合任务(root compound task)中可以看到，第一个方法定义了巨魔的最高优先级。如果他能看到敌人，他会使用NavigateToEnemy任务导航，并使用DoTrunkSlam任务攻击敌人。否则，他就会采用下一种方法。下一个方法将运行三个任务;选择下一个要检查的桥，导航到那个桥，检查桥上有没有敌人。让我们看看组成这些方法和域的其余部分的基本任务。

```
Primitive Task [DoTrunkSlam]
Operator [AnimatedAttackOperator(TrunkSlamAnimName)]
Primitive Task [NavigateToEnemy]
Operator [NavigateToOperator(EnemyLocRef)]
 Effects [WsLocation = EnemyLocRef]
Primitive Task [ChooseBridgeToCheck]
Operator [ChooseBridgeToCheckOperator]
Primitive Task [NavigateToBridge]
Operator [NavigateToOperator(NextBridgeLocRef)]
 Effects [WsLocation = NextBridgeLocRef]
Primitive Task [CheckBridge]
Operator [CheckBridgeOperator(SearchAnimName)]
```

第一个任务DoTrunkSlam是一个示例，说明了原子任务(primitive task)在HTN域中如何描述一个操作符(operator)。在这里，任务实际上是执行一个动画攻击操作符(operator)，动画名称作为一个术语传递进来。下一个任务NavigateToEnemy也是这样的一个例子，但在成功完成此任务时，通过原语任务的效果将世界状态(world state)的WsLocation字段设置为EnemyLocRef。

**Finding a Plan**

**找到一个计划**

对于由复合任务(compound task)和原子任务(primitive task)组成的域(domain)，我们从如何将这些任务组合在一起表示一个NPC的形象开始。结合世界状态(world state)，我们可以探讨下HTN的主体部分，规划器(planner)。_有三个情况会强制规划器(planner)去寻找新的计划(plan):_

_1.NPC完成或失败当前的计划；_

_2.NPC没有计划；_

_3.NPC的世界状态(world state)通过传感器(sensor)改变。_

_如果出现上述任何一种情况，规划器(planner)将尝试生成计划(plan)。_要做到这一点，规划器(planner)从一个根复合任务(root compound task)开始，该任务表示我们正在努力规划的问题领域。以前面的示例来说，这个根任务是BeTrunkThumper任务。这个根任务被推到TasksToProcess堆栈上。接下来，规划器(planner)创建世界状态(world state)的副本。规划器(planner)将修改这个世界状态(world state)的副本，“模拟”执行任务时会对世界状态(world state)发生的改变。

在执行了这些初始化步骤之后，规划器(planner)开始对要处理的任务进行迭代。在每次迭代中，规划器(planner)从TasksToProcess堆栈中弹出下一个任务。如果它是一个复合任务，计划器会尝试分解它——首先，通过搜索它的方法(method)来寻找第一组有效的条件。如果找到一个方法(method)，该方法(method)的子任务(subtask)将添加到TaskToProcess堆栈中。如果没有找到有效的方法(method)，规划器(planner)的状态将回滚到分解的最后一个复合任务。稍后我们将详细介绍如何恢复规划器(planner)的状态。

如果下一个任务是原子任务(primitive task)，我们需要根据世界状态(world state)检查它的先决条件(precondition)。如果满足条件，则将任务添加到最终计划(final plan)，并将其影响(effect)应用到当前世界状态(world state)的副本。应用这些影响(effect)是因为规划器(planner)假定任务将会成功。这使得将来的方法(method)可以考虑新的状态。如果原子任务(primitive task)的条件不满足，规划器(planner)的状态将回滚，就像对复合任务(compound task)所做的那样。此迭代过程将继续，直到TasksToProcess堆栈为空。在完成时，规划器(planner)将以原子任务(primitive task)的列表作为结果而结束，或者规划器(planner)将回滚到足够靠前以导致TasksToProcess堆栈没有可迭代的任务。下面是演示此过程的示例伪代码。

```
WorkingWS = CurrentWorldState
TasksToProcess.Push(RootTask)
while TasksToProcess.NotEmpty
{
CurrentTask = TasksToProcess.Pop()
if CurrentTask.Type == CompoundTask
{
 SatisfiedMethod = CurrentTask.FindSatisfiedMethod(WorkingWS)
 if SatisfiedMethod != null
 {
 RecordDecompositionOfTask(CurrentTask, FinalPlan, DecompHistory)
 TasksToProcess.InsertTop(SatisfiedMethod.SubTasks)
 }
 else
 {
 RestoreToLastDecomposedTask()
 }
}
else//Primitive Task
{
 if PrimitiveConditionMet(CurrentTask)
 {
 WorkingWS.ApplyEffects(CurrentTask.Effects)
 FinalPlan.PushBack(CurrentTask)
 }
 else
 {
 RestoreToLastDecomposedTask()
 }
}
}
```

RecordDepositionOfTask和RestoreToLastDecomposedTask函数中发生了一些奇妙的事情，需要更详细地解释。record函数将规划器(planner)的状态记录到DecompHistory堆栈中。这包括TasksToProcess和FinalPlan容器，以及为分解及其所属复合任务所选择的方法(method)。通过restore函数将记录的状态弹出到规划器(planner)，规划器(planner)可以在不能分解复合任务或者原子任务的条件不能满足时回溯。

您可能已经意识到，规划器(planner)使用深度优先搜索来查找有效的计划。这意味着您可能必须探索整个领域才能找到一个有效的计划。但是，务必记住，您正在遍历一个任务层次结构。这个层次结构允许规划器(planner)通过复合任务的方法剔除网络中的大部分。因为我们没有使用启发式或代价(比如A\*和Dijkstra)搜索，所以我们可以跳过任何类型的排序。这些特性使得《变形金刚:塞伯坦的秋天》中的HTN规划系统比《变形金刚:塞伯坦的战争》(HighMoon10)中的GOAP系统要快得多。

现在已经解释了规划器(planner)，我们可以扩展示例，看看Trunk Thumper的域的修改版本是如何分解的(图12.2)。这个域的根任务仍然是BeTrunkThumper，但是DoTrunkSlam现在是一个复合任务。DoTrunkSlam有两个方法—每个方法执行不同型式的trunk slam任务。为了简单起见，省略了这两个复合任务的方法条件。在域的下面，您可以看到规划器(planner)的迭代从上到下。对于每个迭代，您可以在TasksToProcess中看到最左边的任务  
堆栈被处理。

**Running the Plan**

**运行计划**

运行HTN计划非常简单。NPC的plan runner将尝试依次执行每个原子任务(primitive task)的操作符(operator)。当成功完成每个任务时，计划器将任务的效果应用到世界状态(world state)。如果任务运行它自身指定的操作符失败，那么计划也会失败并强制重新计划。

如果当前或剩余任务的任何条件无效，该计划也会失败。plan runner以运行中世界状态(world state)为参照来监控这些任务的先决条件(precondition)能否成立，就像计划人员一样。当它确认每个任务的前提条件时，就会应用它的影响(effect)。

![[atom/技术/AI/attachmen/11215219abed50042172fecaf02dca42_MD5.jpg]]

图12.2

图12.2 Decomposition of the Trunk Thumper domain, showing the resulting plan if BeTrunkThumper. Method0 and DoTrunkSlam.Method.1 were chosen.

图12.2 Trunk Thumper域的分解，显示BeTrunkThumper任务分解方案。方法Method0和方法DoTrunkSlam.Method.1 被选择。

运行中的世界状态(world state)。应用这些影响(effect)是很重要的，因为后续任务的先决条件(precondition)可能依赖于应用这些影响(effect)才能有效。这个计划的验证使HTN域能够更有表现力，并对世界状态(world state)的变化做出反应。

**Using Recursion for Greater Expressiveness**

**利用递归展现更强大的表现力**

在我们的巨魔游戏中，设计师认为树干攻击有点太过于压制性。他们认为应该让树干在三次攻击后断裂，迫使巨魔寻找另一个。首先，我们可以将属性WsTrunkHealth添加到世界状态(world state)。通过将攻击方法封装到它自己的复合任务中，并添加一点递归，我们将能够修改巨魔的攻击行为。更改后的域现在是:

```
Compound Task [BeTrunkThumper]
Method [ WsCanSeeEnemy == true]
 Subtasks [AttackEnemy()]// using the new compound task
Method [true]
 Subtasks [ChooseBridgeToCheck(), NavigateToBridge(), CheckBridge()]
Compound Task [AttackEnemy]//new compound task
Method [WsTrunkHealth > 0]
 Subtasks [NavigateToEnemy(), DoTrunkSlam()]
Method [true]
 Subtasks [FindTrunk(), NavigateToTrunk(), UprootTrunk(), AttackEnemy()]
Primitive Task [DoTrunkSlam]
Operator [DoTrunkSlamOperator]
 Effects [WsTrunkHealth += -1]
Primitive Task [UprootTrunk]
Operator [UprootTrunkOperator]
 Effects [WsTrunkHealth = 3]
Primitive Task [NavigateToTrunk]
Operator [NavigateToOperator(FoundTrunk)]
 Effects [WsLocation = FoundTrunk]
```

当我们的巨魔看到敌人时，它会像以前一样攻击，只是现在，这个行为被包裹在一个新的复合任务中，叫做“AttackEnemy”。该任务的高优先级方法像开始的域一样执行navigate和slam，但现在的条件是树干具有一定的生命值。对DoTrunkSlam任务的更改将在每次成功攻击时降低树干的生命值。这使规划器(planner)在应对树干损坏状况时能够使用较低优先级的方法。

第二种AttackEnemy的方法(method)是获得一个新的树干。它首先选择一棵新树，导航到那棵树，拔掉它，然后它就可以AttackEnemy了。这就是递归的作用。当规划器(planner)再次分解攻击敌人的任务时，它现在可以再次考虑这些方法。如果树干的生命值仍然为零，这将导致规划器(planner)进行无限循环。但是新的UprootTrunk任务的效果将WsTrunkHealth设置为3，允许我们生成一个计划FindTrunk→NavigateToTrunk→UprootTrunk→NavigateToEnemy→DoTrunkSlam。这个新域允许我们重用域中已经存在的方法，以使巨魔重新攻击。

**Planning for World State Changes not Controlled by Tasks**

**规划不受任务控制的世界状态变化而产生的计划**

到目前为止，我们一直在构建的所有计划都依赖于原子任务(primitive task)改变世界状态(primitive task)的效果。但是，当世界状态在原子任务(primitive task)控制之外发生改变时，会发生什么呢?为了探究这一点，让我们再次修改我们的示例。让我们假设一个设计师注意到当巨魔看不到敌人的时候，他会回到桥上巡逻。设计师要求你实施一种追赶敌人的行为，并在再次看到敌人时做出反应。让我们看看我们可以对域进行哪些更改来处理这个问题。

```
Compound Task [BeTrunkThumper]
Method [ WsCanSeeEnemy == true]
 Subtasks [AttackEnemy()]
Method [ WsHasSeenEnemyRecently == true]//New method
 Subtasks [NavToLastEnemyLoc(), RegainLOSRoar()]
Method [true]
 Subtasks [ChooseBridgeToCheck(), NavigateToBridge(), CheckBridge()]
Primitive Task [NavToLastEnemyLoc]
Operator [NavigateToOperator(LastEnemyLocation)]
 Effects [WsLocation = LastEnemyLocation]
Primitive Task [RegainLOSRoar]
Preconditions[WsCanSeeEnemy == true]
Operator [RegainLOSRoar()]
```

通过这种重新设计，如果Trunk Thumper看不到敌人，规划器(planner)就会向下移动到新方法，这个方法依赖世界状态(world state)里的WsHasSeenEnemyRecently属性。这个方法的任务是navigate到最后一次看到敌人的地方，如果他再次看到敌人，就会执行一个“怒吼”的动画。这里的问题是，RegainLOSRoar任务的先决条件是WsCanSeeEnemy为真。这个世界状态(world state)是由巨魔的视觉传感器(sensor)处理的。当规划器(planner)将RegainLOSRoar任务放到最终任务(final plan)列表中时，它的先决条件检查将失败，因为域中没有任何东西表示navigate完成时预期的世界状态(world state)能满足条件。

为了解决这个问题，我们将引入预期影响(expected effect)的概念。预期影响(expected effect)是只在规划期间和计划验证期间应用到世界状态的效果。这里的思想是，您可以表达世界状态中的变化，这些变化应该基于正在执行的任务而发生。这使得规划器(planner)能够根据自己认为在这一过程中会完成的事情，对未来进行更深入的规划。记住，规划器(planner)在做决策时的一个关键优势是他们可以对未来进行推理，帮助他们更好地决定下一步该做什么。为了适应这一点，我们可以将域内的NavToLastEnemyLoc改为:

```
Primitive Task [NavToLastEnemyLoc]
Operator [NavigateToOperator(LastEnemyLocation)]
 Effects [WsLocation = LastEnemyLocation]
 ExpectedEffects [WsCanSeeEnemy = true]
```

现在，当这个任务从分解列表(decomposition list)中弹出时，当前的世界状态(world state)将被更新为预期的效果，这时RegainLOSRoar任务将被允许继续向最终任务的链中添加任务。这种简单的行为可以用多种不同的方式实现，但在《变形金刚:塞伯坦的秋天》的制作过程中，预期的效果多次派上了用场。它们是在HTN域中让表达性更强的一种简单方法。

**How to Handle Higher Priority Plans**

**如何处理优先级更高的计划**

至此，我们已经根据任务方法的顺序分解了复合任务。这是一种很自然的搜索方式，但是必须考虑到这些攻击会对 Trunk Thumper 域的改变。

```
Compound Task [AttackEnemy]
Method [WsTrunkHealth > 0, AttackedRecently == false,
CanNavigateToEnemy == true]
 Subtasks [NavigateToEnemy(), DoTrunkSlam(), RecoveryRoar()]
Method [WsTrunkHealth == 0]
 Subtasks [FindTrunk(), NavigateToTrunk(), UprootTrunk(), AttackEnemy()]
Method [true]
 Subtasks [PickupBoulder(), ThrowBoulder()]
Primitive Task [DoTrunkSlam]
Operator [DoTrunkSlamOperator]
 Effects [WsTrunkHealth += -1, AttackedRecently = true]
Primitive Task [RecoveryRoar]
Operator [PlayAnimation(TrunkSlamRecoverAnim)]
Primitive Task [PickupBoulder]
Operator [PickupBoulder()]
Primitive Task [ThrowBoulder]
Operator [ThrowBoulder()]
```

在一些游戏测试后，我们的设计师觉得我们的巨魔是过于强力攻击。只有当它去抓另一个树干时，它才会放松对玩家的攻击。设计师建议在trunk slam后加入一个回复动画，并在巨魔最近攻击后设置一个不允许slam攻击的新条件。我们的设计师也注意到，如果我们的巨魔不能导航到他的敌人那里，他的行为就会很奇怪(例如，由于障碍)。他决定采取低优先级攻击，如果发生这种情况，就扔一块大石头。

关于这些行为变化的一切似乎都相当简单，但是我们需要仔细看看在运行trunk slam计划时会发生什么。在实际slam动作之后，我们开始运行RecoveryRoar任务。如果在执行此任务时，世界状态(world state)发生改变并导致重新规划，RecoveryRoar任务将被中止。原因是，当规划器(planner)运行到处理slam方法时，世界状态(world state)中的AttackRecently将被设置为true，因为DoTrunkSlam任务成功完成，它的影响(effect)应用到了世界状态。这将导致规划器(planner)跳过“slam”方法任务，转而采用新的“throw boulder”方法，从而产生新的计划。这将导致RecoveryRoar任务在执行过程中被中止，即使当前运

在这种情况下，我们需要一种方法来确定运行计划的“优先级”。有几种方法可以解决这个问题。由于HTN是一个图数据结构，我们可以使用某种形式的基于成本的搜索，例如A\*或Dijkstra。这将涉及到将某种成本绑定到我们的任务甚至方法上。不幸的是，在实践中调优这些成本非常棘手。不仅如此，我们现在还必须在规划器(planner)中添加排序，这将减慢它的执行速度。

相反，我们希望保持方法的“顺序优先级”的简单性和可读性。问题是计划(plan)不知道规划器(planner)为达成计划而进行的复合任务的分解顺序——它只执行原子任务(primitive task)的操作符。

![[atom/技术/AI/attachmen/e17405e4a060eca65207f383225bb0d8_MD5.jpg]]

图12.3

图12.3 All possible plans with the Trunk Thumper domain and the Method Traversal Record for each plan, sorted by priority

图12.3 Trunk Thumper域里所有可能的计划和每个计划的方法遍历记录，按优先级排序

我们希望用复合任务的方法(method)的顺序来定义优先级——但是计划并不知道复合任务是什么。为了解决这个问题，我们可以在搜索计划中对HTN域的遍历时进行编码。方法(method)遍历记录(MTR)仅存储为创建计划而分解的每个复合任务选择的方法指数。现在我们有了MTR，我们可以用两种不同的方式来帮助我们找到更好的计划。最简单的方法是正常规划，并将新发现的计划的MTR与当前运行的计划的MTR进行比较。如果新计划中选择的所有方法索引都具有同等或更高的优先级，那么我们就找到了新计划。图12.3显示了一个示例

当我们分解新的搜索中的复合任务时还可以在规划过程中选择使用当前计划的MTR。我们可以使用MTR来搜索一个有效的方法，只允许具有相等或更高优先级的方法。这允许我们基于当前计划(plan)的MTR去剔除HTN里的的整个分支。第一种方法是两种方法中比较容易的一种，但是如果你发现你在计划中花费了大量的处理时间，第二种方法可以帮助你加快速度。

既然我们有能力为更高优先级的计划中止当前正在运行的计划，那么有一个微妙的实现细节可能会导致npc出现意外行为。如果您设置您的规划器(planner)对世界状态(world stat)变化重新进行规划，那么规划器(planner)将在任务对成功执行的影响(effect)后尝试重新进行规划。思考下面的Trunk Thumper域分段的修改。

```
Compound Task [AttackEnemy]
Method [WsPowerUp = 3]
 Subtasks [DoWhirlwindTrunkAttack(), DoRecovery()]
Method [WsEnemyRange > MeleeRange,]
 Subtasks [DoTrunkSlam(), DoRecovery()]
Primitive Task [DoTrunkSlam]
Operator [AnimatedAttackOperator(TrunkSlamAnimName)]
 Effects [WsPowerUp += 1]
Primitive Task [DoWhirlwindTrunkAttack]
Operator [DoWhirlwindTrunkAttack()]
 Effects [WsPowerUp = 0]
Primitive Task [DoRecover]
Operator [PlayAnimation(TrunkSlamRecoveryAnim)]
```

这个新行为旨在让巨魔在执行DoTrunkSlam三次之后执行DoWhirlwindTrunkAttack任务。这是通过让DoTrunkSlam任务每次执行后的影响(effect)让WsPowerUp属性增加1来实现的。乍一看，这似乎很好，但设计师在你的办公桌上告诉你，巨魔现在每次都用一个trunk slam直接造成whirlwind attack。问题出现在DoTrunkSlam的第三次执行时任务的影响(effect)被应用，规划器(planner)强制重新规划。当WsPowerUp等于3时，规划器(planner)将选择优先级更高的DoWhirlwindTrunkAttack方法。这样就取消了DoRecovery任务，这个任务原本是设计用来打断连续攻击的，让玩家有一些时间做出反应。

通常，whirlwind方法应该能够取消较低优先级的计划。但是当前运行的计划仍然有效，出现此错误的唯一原因是在世界状态(world state)变化时规划器(planner)重新规划了计划，包括通过成功完成原子任务的影响(effect)对世界状态(world state)进行的更改。简单地当世界状态通过应用原子任务的影响(effect)发生变化时，只要不重新规划就可以解决这个问题——这很好，因为无论如何，这个计划都是在考虑世界状态变化的情况下制定的。虽然这是一个很好的改变，但它不是完整的解决方案。规划器(planner)正在执行的任务之外的任何世界状态更改都将迫使重新规划并导致错误重新出现。

这里核心的问题是域(domain)和它目前是如何设置的。这里有几种不同的方法来解决这个问题，你怎么考虑它很重要。如果说恢复动画是攻击的一部分，那将该动画合并到攻击动画中是值得的。这样的话，在slam attack之后总是应该进行恢复。但这损害了域的模块化。比如设计师想要三连slams，然后做一个恢复动画呢?

更好的方法是使用世界状态(world state)来描述需要DoRecovery的原因。考虑下以下的修改:

```
Compound Task [AttackEnemy]
 Method [WsPowerUp = 3]
 Subtasks [DoWhirlwindTrunkAttack(), DoRecovery()]
 Method [WsEnemyRange > MeleeRange,]
 Subtasks [DoTrunkSlam(), DoRecovery()]
Primitive Task [DoTrunkSlam]
 Operator [AnimatedAttackOperator(TrunkSlamAnimName)]
 Effects [WsPowerUp += 1, WsIsTired = true]
Primitive Task [DoWhirlwindTrunkAttack]
 Preconditions [WsIsTired == false]
 Operator [DoWhirlwindTrunkAttack()]
 Effects [WsPowerUp = 0]
Primitive Task [DoRecovery]
 Operator [PlayAnimation(TrunkSlamRecoveryAnim)]
 Effects [WsIsTired = false]
```

使用世界状态(world state)下的WsIsTired，我们可以正确地描述需要DoRecovery任务的原因。DoTrunkSlam任务现在会使Trunk Thumper感到疲劳，直到他有机会恢复，他才能执行DoWhirlwindTrunkAttack任务。现在，当世界状态改变时，DoRecovery任务不会被中断，但是我们保留了DoTrunkSlam和DoRecovery的模块。当按照计划优先级选择执行时，这些细微的细节确实会对您的HTN行为造成困扰。

当你遇到这些类型的行为问题时，问问自己是否正确地表示了这个世界是很重要的。正如我们在本例中看到的，一个简单的世界状态(world state)是所需要的全部。

**Managing Simultaneous Behaviors**

**管理同时发生的行为**

很多不同的行为选择算法都很擅长一次只做一件事，但当同时做两件事时，就会出现复杂的情况。幸运的是，有几种方法可以让HTN处理这个问题。

人的第一反应可能是将多个操作符合并为一个操作符。这是可行的，但有几个缺陷:它丧失了重用我们已经开发的操作符的能力，多个操作符的组合带来了额外的复杂性，损害了可维护性，如果处理不当，对组合操作符的任何变化都可能迫使我们引入重复的代码。您可能会遇到需要同时做多个事情的行为，通常情况下您会想要避免使用这种方法。

更直观的处理方法是构建一个单独的HTN域来处理NPC的不同组件。以我们的巨魔为例，我们可能会有这样的行为，我们需要他导航到他的敌人，但同时保护自己免受攻击。我们可以将其分解为多个operators来控制身体的不同部分，一个导航operator负责下半身，一个防守operator负责上半身。知道了这一点，我们可以建立两个领域，并使用两个规划器(planner)来处理上层和下层主体

您可能在早期就发现这可能很难实现。关键点是你需要同步每个规划器(planner)中的任务。你可以通过确保你在每个规划器(planner)里的知道世界状态(world state)中正在发生的事情来做到这点。在我们的troll示例中，我们世界状态(world state)里有一个Navigating属性，在运行任何下半身navigation任务时将其设置为true。这将允许上半身规划器(planner)根据这些信息做出决策。下面是如何设置这两个域的示例。

```
Compound Task [BeTrunkThumperUpper]//Upper domain
 Method [WsHasEnemy == true, WsEnemyRange <= MeleeRange]
 Subtasks [DoTrunkSlam()]
 Method [Navigating == true, HitByRangedAttack == true]
 Subtasks [GuardFaceWithArm()]
 Method [true]
 Subtasks [Idle()]
Compound Task [BeTrunkThumperLower]//Lower domain
 Method [WsHasEnemy == true, WsEnemyRange > MeleeRange]
 Subtasks [NavigateToEnemy(), BeTrunkThumperLower()]
 Method [true]
 Subtasks [Idle()]
Primitive Task [DoTrunkSlam]
 Operator [DoTrunkSlamOperator]
Primitive Task [GuardFaceWithArm]
 Operator [GuardFaceWithArmOperator]
Primitive Task [NavigateToEnemy]
 Operator [NavigateToOperator(Enemy)]
 Effects [WsLocation = Enemy]
Primitive Task [Idle]
 Operator [IdleOperator]
```

这个很好用，但是有一些小问题。第二个计划器会增加一点性能损失。保持这些域的同步将损害它们的可维护性。最后，相信我，当其他程序员遇到您刚刚用多个规划器(planner)创建的调试问题时，调试将会变得非常困难，您将没任何朋友。

对于我们的巨魔防护例子，还有另一种不需要两个规划器(planner)参与的办法。目前，成功到达目的地后导航任务完成。相反，我们可以让导航任务启动路径跟随并立即完成，因为路径跟随是在后台发生的，而不是作为plan runner中的一个任务。这使我们可以在导航的过程中进行规划其他任务，这样我们就可以举起武器来保护巨魔不受攻击。只要我们在世界状态(world state)有描述我们正在导航和当前距离目的地的属性，这就可以工作。有了它，我们就可以知道什么时候到达目的地，并据此制定计划。下面这个例子是展示域应该的样子。

```
Compound Task [BeTrunkThumper]
 Method [WsHasEnemy == true, WsEnemyRange <= MeleeRange]
 Subtasks [DoTrunkSlam()]
 Method [WsHasEnemy == true, WsEnemyRange > MeleeRange]
 Subtasks [NavigateToEnemy()]
 Method [Navigating == true, HitByRangedAttack == true]
 Subtasks [GuardFaceWithArm()]
 Method [true]
 Subtasks [Idle()]
 Primitive Task [DoTrunkSlam]
 Operator [DoTrunkSlamOperator]
 Primitive Task [GuardFaceWithArm]
 Operator [GuardFaceWithArmOperator]
 Primitive Task [NavigateToEnemy]
 Operator [NavigateToOperator(Enemy)]
 Effects [Navigating = true]
 Primitive Task [Idle]
 Operator [IdleOperator]
```

正如您所看到的，这个域类似于两个域处理方式。这两种方法都依赖于世界状态(world state)才能正确工作。在双域中，世界状态(world state)的Navigating属性被用来保持计划者的同步。在后一种方法中，世界状态(world state)属性用于表示在后台下路径跟随的事件，而不需要两个域和两个规划器(planner)运行。

**Speeding up Planning with Partial Plans**

**通过局部规划加快规划速度**

让我们假设我们已经将Trunk Thumper的域构建为一个相当大的网络。在优化规划器(planner)本身之后，您发现需要减少几毫秒的规划时间。有很多方法可以提高它的性能。正如我们所解释的，HTN通过复合任务中的方法自然地剔除了很大一部分搜索空间。然而，在某些情况下，我们可以添加更多的方法来剔除更多的搜索空间。为了做到这一点，我们需要有恰当地世界状态(world state)表现。

如果这些技术不能让你达到你需要的速度，局部规划可以。局部规划是HTN最强大的功能之一。简单地说，它允许规划器(planner)有能力不完全分解出一个完整的计划(plan)。HTN能够做到这一点是因为它使用前进式分解或前进式搜索来查找计划。也就是说，规划器(planner)从当前的世界状态开始，并从那时开始进行规划。这允许计划者只向前地计划几个步骤。

GOAP和STRIPS规划器的变体，另一方面，使用向后搜索\[Jorkin 04\]。这意味着搜索方式是从期望的目标状态到当前的世界状态。这样搜索意味着规划器(planner)必须完成整个搜索，以便知道第一步要做什么。我们将退回到Trunk Thumper域的一个简单版本，并演示如何将其分解为局部计划域。

```
Compound Task [BeTrunkThumper]
Method [WsCanSeeEnemy == true]
 Subtasks [NavigateToEnemy(), DoTrunkSlam()]
Primitive Task [DoTrunkSlam]
Operator [DoTrunkSlamOperator]
Compound Task [NavigateToEnemy]
Method […]
 Subtasks […]
```

这里，我们有一个方法(method)，如果WsCanSeeEnemy为真，它将同时展开NavigateToEnemy和DoTrunkSlam任务。因为任何由NavigateToEnemy组成的任务都可能需要很长时间，所以把它分成一个局部计划是个不错的选择。因为世界状态随时有可能会改变，迫使我们的巨魔做出不同的决定，所以没有太多的意义去规划太远的未来。我们可以将这一特定计划转换为局部计划:

```
Compound Task [BeTrunkThumper]
Method [WsCanSeeEnemy == true, WsEnemyRange > MeleeRange]
 Subtasks [NavigateToEnemy()]
Method [WsCanSeeEnemy == true]
 Subtasks [DoTrunkSlam()]
Primitive Task [DoTrunkSlam]
Operator [DoTrunkSlamOperator]
Compound Task [NavigateToEnemy]
Method […]
 Subtasks […]
```

这里，我们将前面的方法分解为两个方法。新的高优先级方法判断如果巨魔目前在近战范围之外就导航到敌人。如果巨魔在近战范围之内，他将执行trunk slam attack。导航任务也是局部计划的主要目标，因为它们通常需要很长时间才能完成。重要的是要指出需要有一个世界状态的WsEnemyRange属性可用来区分计划,分离这个计划才是可行的。

这种局部规划的方法要求域的作者自己创建分离。但是有一种方法可以自动化这个过程。通过给原子任务指定出一个“时间”的概念，规划器(planner)可以跟踪它已经计划的未来有多远。然而，结合到域这种方法有几个问题。

```
Compound Task [BeTrunkThumper]
Method [WsCanSeeEnemy == true]
 Subtasks [NavigateToEnemy(), DoTrunkSlam()]
Primitive Task [DoTrunkSlam]
Preconditions[WsStamina > 0]
Operator [DoTrunkSlamOperator]
Compound Task [NavigateToEnemy]
Method […]
 Subtasks […]
```

对于这个域，假设由导航构成的原子任务越过在规划器里设置的时间极限值。这将使巨魔导航到了敌人面前。但是，如果世界状态WsStamina属性为零，由于它的先决条件无法满足巨魔就无法执行DoTrunkSlam。自动化的局部计划分离忽略了正确验证计划的能力。当然，可以编写该方法来包括持久力检查以避免此问题。但既然两种方式都是有效的，最好确保两种方法都能产生相同的结果。如果不这样做，将会导致游戏中出现一些不易察觉的bug。

即使你觉得这不是一个真正的问题,还有一个问题是如何使局部计划的失败继续。我们可以从根任务再重新,但这需要我们以某种方式改变域，以让它理解已经完成了整个计划的第一部分。在我们的例子中，我们必须添加一个更高优先级的方法来检查我们是否在进行近战攻击的范围内。但是如果我们必须这样做，那么自动化部分规划的意义是什么呢?

更好的解决方案是记录未处理列表的状态。这样，我们就可以修改规划器(planner)，使其从任务列表开始，而不是从一个根任务开始。这样我们就可以从中断的地方开始继续搜索。当然，我们不能回滚计划第二部分开始前。遇到这种情况意味着您已经运行了不应该运行的任务。因此，如果用户遇到这种情况，他们就不能使用局部计划，因为计划中稍后的任务需要进行验证，以获得正确的行为。

在《变形金刚:塞伯坦的秋天》中，我们只是将局部计划构建到域中。对于我们来说，在游戏中产生细微bug的几率很高，我们发现在没有必要验证完整计划时，我们会自然地将局部计划放到NPC域中。我们的许多npc使用了12.9节中的最后一个例子来导航，这也是局部规划的一个例子。

**Conclusion**

**结论**

通过创建一个简单的NPC的过程，可以让您对任何行为选择系统的实现所涉及的细节都大开眼界。希望我们已经对HTN进行了足够的探索，以展示其描述行为的自然方法、原子任务的可重用性和模块化。HTN对未来进行推理的能力允许只有规划器才能具备的一种表达方式。我们还试图指出开发人员在实现它时可能遇到的潜在问题。HTN对《变形金刚:塞伯坦的秋天》的AI程序员来说是真正有好处的，我们相信对你来说也一样。

**References**

**引用**

```
[Erol et al. 94] K. Erol, D. Nau, and J. Henler, “HTN planning: Complexity and expressivity.”AAAI-94 Proceedings, 1994.
[Erol et al. 95] K. Erol, J. Henler, and D. Nau. “Semantics for Hierarchical Task-Network Planning.” Technical report TR 95-9. The Institute for Systems Research, 1995.
[Ghallab et al. 04] M. Ghallab, D. Nau, and P. Traverso, Automated Planning. San Francisco,CA: Elsevier, 2004, pp. 229–259.
[HighMoon 10] Transformers: War for Cybertron, High Moon Studios/Activision Publishing,2010.
[HighMoon 12] Transformers: Fall of Cybertron, High Moon Studios/Activision Publishing,2012.
[Jorkin 04] Jeff Orkin. “Applying goal-oriented action planning to games.” In AI Game Programming Wisdom 2, edited by Steve Rabin. Hingham, MA: Charles River Media,2004, pp. 217–227.
```

实例

[https://github.com/Strik3One/SHTNPlanner](https://link.zhihu.com/?target=https%3A//github.com/Strik3One/SHTNPlanner)

Simple Hierarchical Task Network Planner for UE4

This is a very simple implementation of a HTN Planner for Unreal Engine 4.25.

This project serves as a graduation project, with the intended outcome to learn more about HTN Planning for video games. This might be helpful to others so feel free to experiment with it yourself and ask any questions that you might have.

If you are unfamiliar with HTN Planning this is a great article to get familiar with the concept:

Exploring HTN Planners through Example

Table of Contents

Setup

Documentation

World State

Operators

Network

Setup

To use the plugin, simply copy the folder Plugins/SHTNPlanner into your Projects or Engines Plugins folder and enable it in your project.

Documentation

Below you will find documentation for using the plugin in your own project. If you notice something is missing, or have additional question, feel free to shoot me a message.

World State

The WorldState is represented by a Blackboard. There are no restrictions on what kind of variables you can add to it, as it utilizes the standard interface for interacting with Blackboards.

Debug Tool

The plugin comes with a simple debug tool that allows you to see the values of the WorldState at runtime. You can find the tool under Window > Developer Tools > SHTN WorldState Debug

In the window that will open you can select agents that are currently running an HTN Planner and see their respective WorldState values as well as the Current Plan and Task of the selected Agent.

When the Select Agent in PIE box is checked, you will be able to select agents when ejected, the debug view will then automatically switch to this agent.

This tool is very basic and might receive additional functionalities in the future. If you know any way to improve the tool feel welcome to do so.

Operators

Operators are where the behavior logic is defined.

To create an Operator class you must create a new Blueprint class that inherits from SHTNOperator\_BlueprintBase

Events & Functions

Almost every event of the operator comes with two parameters. The first on is the owning AIController and the second one is the Operator Parameter represented as a uint8 or Byte which can be cast back to the expected Enum.

Check Conditions

Will be called both during planning and execution. During planning this will be used in order to check the conditions of a task to decide if it should be added to the plan or not. During execution it will be used to check if the task is still valid for execution - in case any WorldState values changed unexpectedly.

When this function is not implemented, the task return true by default upon condition checking

Get Score

Will be called if the composite task that contains this task is of type Scored.

Receive Initialize Action

This event gets called upon activation of this operator. An operator gets activated each time it appears in the plan. If you require casting the owner to your custom controller class this is a good place to do that.

Receive Execute Action

This event gets called every frame that the Operator is active. Logic defining the behavior should go here.

Finish Execution

This function should be called once the Operator has completed its execution either succesfully or not.

Receive Abort

If the operator gets aborted for any reason, you can use this event in order to clean up things that might have to be cleaned up.

Finish Abort

Is required to be called when Abortion of a task is completed. If Finish Execution return succesfully before this node is called, the effects of the task will still be applied. You can use then when a task gets aborted but you want it to finish it's execution regardless. Whilst aborting the Receive Execute Action event will no longer be called.

Apply Effects

Gets called both during planning, and when a task completes its execution. In case you want to handle these scenarios different a boolean parameter Is Planning will tell you if the operator is planning or executed. Beware that this function will be called before Initialize Action during planning, this means you aren't able to make use of initialized values so keep this in mind.

Changes to the WorldState must be changed through the supplied WorldState object in the event.

Network

The network is where you will build the domain. The domain is a collection of composite and primitive tasks with conditions and effects which are used by the planner in order to produce a plan. The conditions and effects are specified in the tasks operator classes itself.

To create a network you must create a new Blueprint class that inherits from SHTNNetwork\_BlueprintBase. Then you must tell the network which Blackboard assets you want to use as the WorldState. Optionally you can specify the names of Blackboard entries that you want to "ignore", meaning the planner wont replan upon changing these values.

You're also able to set the maximum amount of plan cycles here, meaning the planner will terminate (and fail) upon exceeding the specified amount of cycles. This can be nice to detect if in certain scenarios the planner encounters an infinite loop

Functions

Build HTN Domain

This function needs to be overriden in the network. This is where you will fill your network with tasks. The function does not take any arguments but will need to return a Domain and a boolean (if the domain building was succesfull or not).

You can use Unreal's 'Make ..." nodes to construct the Domain object

Set Default World State

This function is not required to be overriden. It can be used in order to procedurally set the starting WorldState of the network. This will come in handy when the agents needs to know certain things in the world. For example: the amount of ammo pickups present in the world.

Tasks

The HTN network consist of two different types of tasks: Composite and Primitive. Composite tasks contain multiple methods to achieve the task. Each method has a list of tasks (composite and primitive) that will be explored. Primitive tasks contain the Operator class as well as a byte parameter.

Composite Tasks

Composite tasks are stored in a map in the domain object, where the key is the name of the task.

Each composite task contains an array of methods which respectively contain an array of tasks.

You can switch the type of the task to Scored, this will order the methods of this task based on the score returned by the first task of each method. This means that when using the Scored type make sure the first task of each method is a Primitive Task

Tasks are specified with names, so be sure you dont make any typos (don't worry the network will throw an error if there are any tasks specified that don't exist in the network)

Primitive Tasks

Just like composites, primitive tasks are stored in a map in the domain object, where the key is the name of the task.

The primitive tasks themselves require both the Operator Class and Parameter to be specified. The Operator Class is the class in which you specify the behavior for this task and the Parameter can be any value you want it to be, which you can use to have different variations of a task in the same Operator.

Running the network

You can run the Network by calling Run HTN Planner and pass in the AIController on which you want this network to run.