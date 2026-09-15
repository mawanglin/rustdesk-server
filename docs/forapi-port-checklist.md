# forapi 功能移植：官方 1.1.17 补丁清单

日期：2026-09-15
状态：分析完成，待执行
分支：feat/align-client-1.4（异步官方 master 1.1.17 base a7736be）

## 结论先行

官方 1.1.17 已在 1.1.15+ **原生吸收 forapi 的绝大多数 register_pk 逻辑**
（handle_udp RegisteredPk 分支几乎同 forapi 的 handle_register_pk）。因此真正要补的
forapi 独有功能只有 4 处。

## 移植清单（3 个必需 + 1 个决策）

### 1. must_login 支持（必需）
- 位置：`start_with_bind` 开头（1.1.17 无此逻辑，需从 forapi 移植）
- forapi 逻辑：
  ```rust
  let must_login = get_arg("must-login");
  if must_login.to_uppercase() == "Y"
      || (must_login == "" && std::env::var("MUST_LOGIN").unwrap_or_default().to_uppercase() == "Y")
  ```
- 作用：决定是否强制客户端带 token 登录。读到后存到 `Inner`（后续校验时判断）。

### 2. peers_online_state（必需）
- 1.1.17 无此函数。从 forapi 移植（纯新增，依赖 `REG_TIMEOUT`/`pm.get_in_memory` 均在）。
- 调用点在 register_peer 后：官方 1.1.17 的 handle_udp RegisterPeerResponse 分支已存在
  （`if !register_peer_response.teleport_enabled()` 附近），需对照 forapi 调用点插入。

### 3. PunchHoleSent / PunchHole 的 token 登录校验（必需）
- forapi 在 handle_tcp 的 PunchHoleSent 分支 + handle_punch_hole_request 里校验 `ph.token`
  （非空则 jwt::verify_token，失败拒绝）。
- 官方 1.1.17 已实现 PunchHoleSent/PunchHole 但**无 token 字段校验**，需叠加。
- 注意：官方 1.1.17 PunchHoleSent 的 proto 字段 `token` 是否存在需查 hbb_common；
  若不含该字段需同步改 libs/hbb_common（子模块）——这会影响升级，需谨慎决策。

### 4. update_addr 改签名（决策点）
- forapi 把 update_addr 改为 `-> bool`（只返回 request_pk，不在函数内发响应），
  调用处（register_peer 分支）补 set_register_peer_response。
- 官方 1.1.17 的 update_addr 仍发响应（`-> ResultType<()>`）。**建议保留 1.1.17 原样**
  除非必须与 forapi 的调用点结构对齐。倾向：保留官方实现 + 只补 peers_online_state。

## 关键开放问题（需查证后决定）

- `libs/hbb_common` 的 PunchHoleSent / PunchHole proto 是否含 `token` 字段？
  - 若含：直接叠加校验，纯 hbbs 侧。
  - 若不含：需 patch 子模块，抬高升级成本 → 与 API 端 alignment 方案一起权衡。

## 验证

- `cargo check --bin hbbs` 通过。
- `hbbs --help` 显示 `--must-login` 参数。
- 无 token / 错 token 的 PunchHole 被拒（日志可见）。
- 最终经 fpk build 管道（musl 交叉编译）编译通过。