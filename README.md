# POE2 每日通货收益

面向《流放之路 2》的 Windows 本地账本工具。它用手动快照和收支调整记录每日通货变化，区分国服与国际服经济数据。

## 当前能力

- 新资料默认国服；可切换国际服，两个区服的快照、调整和价格不会混算。
- 手动快照、交易/兑换/制作收支调整、按日与按周账本。
- 手动价格快照与 CSV 导入；同一时点的手动确认价优先于自动价。
- 国际服行情状态预留 poe.ninja 适配器；国服在验证数据源前明确显示“未配置”。
- OCR/日志只可生成本地候选；只有确认后才创建快照。

## 本地运行

```bash
pnpm install
pnpm tauri dev
```

测试与构建：

```bash
pnpm exec vitest run
cargo test --manifest-path src-tauri/Cargo.toml
pnpm tauri build
```

## 手动价格 CSV

在当前选择的区服导入，表头必须完全为：

```csv
currency_id,value,quoted_in,captured_at
exalted,12,chaos,2026-09-03T12:00:00+08:00
```

`value` 必须是正整数。导入行会作为手动确认价格保存；不会导入或猜测其它区服的价格。

## 隐私与安全

账本 SQLite 数据库仅保存在本机。应用不读取游戏内存、不注入或自动操作游戏客户端，也不上传快照、候选或账本数据。自动行情失败不会阻断本地记账。
