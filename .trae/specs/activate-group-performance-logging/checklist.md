# activate_group 性能耗时日志 - 验证清单

- [ ] Checkpoint 1: `Cargo.toml` 中 `[dependencies]` 末尾新增了 `log` 与 `fern` 两行，版本号合理（log 0.4.x，fern 0.6.x）
- [ ] Checkpoint 2: `state.rs` 的 `AppState` 中新增 `log_path: PathBuf` 字段，`AppState::new` 签名增加第三个参数
- [ ] Checkpoint 3: `lib.rs` 的 `setup` 中在 `app_data_dir` 解析后初始化 fern logger：(a) 拼接 `performance.log` 路径；(b) 配置 chrono 时间戳格式；(c) 同时 chain 到 stdout 和文件；(d) `apply()` 或等效调用成功
- [ ] Checkpoint 4: `lib.rs` 的 `setup` 中 `AppState::new` 调用时把 `log_path` 传入，编译无参数不匹配错误
- [ ] Checkpoint 5: `env_service.rs::activate_group` 函数开头有 `total_start`，并有入口 `log::info!` 消息
- [ ] Checkpoint 6: `activate_group` 步骤 1~6 每个步骤后都有一条 `log::info!` 输出，格式为 `[activate_group] [group_id=xxx] 步骤N: <描述>: <N.NN>ms`
- [ ] Checkpoint 7: `activate_group` 在所有 return 分支前（含冲突 early return、正常成功、错误 `?` 路径之后）都有「总耗时」日志
- [ ] Checkpoint 8: `env_service.rs::deactivate_group` 同样有入口日志、4 个步骤日志、总耗时日志，tag 为 `[deactivate_group]`
- [ ] Checkpoint 9: 所有日志格式使用毫秒单位，精度为 2 位小数（`{:.2}ms`）
- [ ] Checkpoint 10: `cargo check --package switchenv_lib` 无编译错误（exit code 0）
- [ ] Checkpoint 11: 引入的新 unsafe 块数量为 0（不得新增 unsafe）
- [ ] Checkpoint 12: 日志文件路径为 `app_data_dir/performance.log`，与 `SwitchEnv.db` 同目录
- [ ] Checkpoint 13: 日志文件使用 `append(true)` + `create(true)` 打开，应用重启后不覆盖旧日志
- [ ] Checkpoint 14: 日志写入无 panic 路径，初始化失败使用 `expect` 或 `?` 处理，不会 silent swallow
- [ ] Checkpoint 15: `env_service.rs` 顶部 `use std::time::Instant;` 或等价的路径写法存在，避免未导入导致编译错误
