# POE2 每日通货收益追踪器设计

## 目标

构建一个仅面向 Windows 的本地桌面应用，帮助《流放之路 2》玩家通过人工校正的通货快照，统计每日各通货的数量变化、已解释的收支，以及未归因的余额变化。

## 范围

### 第一版包含

- Tauri 2 + Vue 3 + TypeScript 的 Windows 桌面应用。
- 本地 SQLite 数据库，默认离线且没有账号、云端或遥测。
- 今日概览、创建快照、通货明细、历史日账本四个页面。
- 手动创建和人工校正的通货快照。
- 相邻有效快照产生的日通货净变化。
- 用户记录的交易收入、交易支出、货币兑换和制作消耗。
- 可替换的截图/OCR 与 `Client.txt` 会话事件接口，但不实施自动读取或识别。
- Rust 单元测试、SQLite 集成测试和关键 Vue 组件测试。

### 第一版不包含

- 游戏内存读取、客户端注入、自动点击、自动操作或未公开 API。
- 账号注册、登录、同步、云端上传或协作。
- 自动 OCR、日志监听、交易成交判定、通货实时价格或装备估值。
- macOS、Linux 和移动端。

## 架构

应用分为 Vue 前端和 Tauri/Rust 后端。Vue 仅渲染 UI、处理表单状态和调用明确的 Tauri commands；它不直接读取磁盘和数据库。Rust 后端负责 SQLite 存储、校验、业务计算与将来本地采集适配器。

业务规则位于无 I/O 的 Rust 领域层中，便于独立测试。数据库层只负责持久化领域对象。前端通过命令获取 DTO，不理解 SQL 或数据库迁移细节。

```
Vue 3 UI
  -> Tauri commands
    -> application service
      -> domain rules / SQLite repository
      -> future: screenshot OCR adapter / Client.txt adapter
```

## 数据模型

### currency_definition

- `id`: 稳定主键。
- `game_key`: 游戏内稳定标识或预设键。
- `name`: 展示名称。
- `category`: 通货分类。
- `is_valued`: 是否可在未来参与估值。

### snapshot

- `id`: 稳定主键。
- `captured_at`: 用户确认的采集时间。
- `source`: `manual` 或未来来源标识。
- `note`: 可选备注。
- `status`: `valid` 或 `invalid`。
- `created_at`: 写入本机的时间。

快照是不可变记录。若内容有误，用户创建修正快照，或将错误快照标记为 `invalid`，而非覆盖原始数据。

### snapshot_entry

- `snapshot_id`: 所属快照。
- `currency_id`: 通货定义。
- `quantity`: 非负整数数量。

每份快照内，每种通货最多一条记录。

### ledger_adjustment

- `id`: 稳定主键。
- `occurred_at`: 收支发生时间。
- `currency_id`: 影响的通货。
- `quantity`: 正整数数量。
- `direction`: `inflow` 或 `outflow`。
- `kind`: `trade`、`exchange`、`crafting` 或 `other`。
- `note`: 可选备注。

日账本为派生视图，不保存为唯一事实来源，允许在快照失效或调整变更后重建。

## 统计口径

每个自然日、每种通货使用当天最早和最晚的有效快照。

```
net_change = last_valid_snapshot_quantity - first_valid_snapshot_quantity
explained_change = sum(inflow adjustments) - sum(outflow adjustments)
unattributed_change = net_change - explained_change
```

只有同日存在至少两份有效快照时，才计算该日的 `net_change`。没有足够快照的日期不会被当作零收益。

## 用户流程

1. 玩家在“创建快照”中录入或粘贴各类通货数量，确认采集时间后保存。
2. 玩家在“通货明细”中添加交易、兑换或制作相关的调整记录。
3. “今日概览”展示各通货净变化、已解释变化和未归因变化。
4. “历史日账本”按日期展示可计算的记录；错误快照被标记失效后，账本立即重建。

## 可靠性与隐私

- 仅在用户明确选择文件或主动提交表单时处理数据。
- 数量必须是非负整数；快照时间和通货必须存在；同一快照不可出现重复通货。
- OCR 和日志适配器返回候选数据，必须经过用户确认才能形成有效快照或调整记录。
- 应用不读取游戏内存、不修改客户端、不自动触发输入。
- 未来账号体系将新增独立同步服务。本地数据使用可迁移 `profile_id` 和 `schema_version`；第一版不保存身份凭据。

## 测试策略

- Rust 领域单测覆盖：快照差分、调整汇总、失效快照排除、数量校验、同日快照边界。
- SQLite 集成测试覆盖：迁移、保存、读取、唯一约束和重建查询。
- Vue 组件测试覆盖：快照表单校验、调整录入和今日汇总的空数据/已计算状态。
- 每一个业务功能均遵循测试先行：先增加失败测试，再实现最小代码使其通过。

## 验收标准

- 用户可以离线创建并校正至少两份同日通货快照。
- 用户可以为任意已定义通货新增收入或支出调整记录。
- 今日页能按通货展示净变化、已解释变化和未归因变化，且计算不含失效快照。
- 应用重启后数据仍存在于本地 SQLite。
- 业务规则、数据库读写和关键 UI 交互均有自动化测试。
- 项目中不存在账号请求、网络上传、游戏内存访问或自动输入功能。
