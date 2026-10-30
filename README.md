# CC30 Nova

CC30 Nova 是使用 Rust、Tauri 2 与 Vue 3 重实现的 CLEVO Control Center 3.0
硬件管理界面。逆向分析与验证基于 `ControlCenter 3.0 Package v6.040`、
THUNDERBOT 911MT / CLEVO `V158RNXW-HL` 和 Insyde BIOS `1.07.02`。

原程序是硬件管理软件，并非 BIOS 刷写工具。因此 CC30 Nova 不提供固件刷写、
任意 EC 访问或其他未经硬件验证的底层入口。

## 已实现

- 实时显示 CPU、GPU 温度、风扇转速与占空比。
- 按固件功能包动态显示并写入可用的电源配置。当前实机支持静音与性能。
- 风扇模式：自动、最大。
- 0% 至 100% 的风扇曲线偏移。
- 只读显示出厂风扇曲线。
- 读取 EC 版本、BIOS 功能包版本、风扇通道数和 RGB 分区能力。
- 只读显示 GPU 当前工作模式及固件允许的切换选项。
- 设置 FnLock、Windows 键与 Win/Fn 交换。
- 设置 FlexiCharger 开关及固件提供的充电启停阈值。
- 设置低刷新率策略与 1.0× 至 2.0× 蓝光过滤强度。
- 只读显示 PgUp/PgDn 功能键状态、键盘灯效能力与电源模式锁定状态。
- 只读显示 DTT、Turbo Fan、Slow Fan 等固件热策略运行标记。
- 读取 Windows 电池设计容量、满充容量、循环次数、健康度与供电状态。
- 读取 SMBIOS 设备身份、CPU 拓扑、内存拓扑、GPU 与操作系统信息。
- 原生桥接及固件能力诊断。
- “设备功能”页集中显示可设置、只读、待接入、安全禁用与不支持的功能。
- 原生桥接不可用时自动进入标识明确的只读模拟模式。
- 适配桌面与窄屏窗口的响应式界面。

## 有意不支持

- BIOS 刷写或固件更新。
- 任意 DCHU / EC 命令执行。
- CPU 或 GPU 超频。
- 键盘 RGB、单键灯效与 Flexikey。
- 电池校准、Flexikey、Fn 键重映射及厂商专属节能功能的写入。
- PgUp/PgDn、DTT、Turbo Fan、Slow Fan 与电源模式锁定的写入。
- 自定义风扇曲线写入。协议虽已完成映射，但开放任意曲线写入仍需针对具体机型
  完成热安全验证。

## 安全边界

Tauri 命令面仅包含：

- `get_snapshot`
- `get_system_info`
- `get_diagnostics`
- `set_power_mode`
- `set_fan_mode`
- `set_fan_offset`
- `set_keyboard_settings`
- `set_charging_threshold`
- `set_low_refresh_rate`
- `set_blue_light_factor`

命令面共十项，其中七项为写入。所有写入均使用固定命令并校验输入范围。后端会在写入前检查固件能力位，并在写入后
回读确认目标值。程序不提供通用“发送命令”接口。

## 原生桥接

程序动态加载 `InsydeDCHU.dll`，不会静态链接到主程序。查找顺序如下：

1. `CC30_DCHU_PATH`
2. Tauri 资源目录
3. 应用程序可执行文件目录
4. 开发环境 `.tools` 路径
5. `Program Files` 下的标准 Control Center 安装目录

若 DLL 无法加载，CC30 Nova 会明确显示模拟模式，并停止尝试硬件写入。
原厂 DLL 属于第三方运行组件，不包含在本仓库中。使用真实硬件开发时，请将
合法取得的 `InsydeDCHU.dll` 放入 `src-tauri/resources/native/`，或通过
`CC30_DCHU_PATH` 指定其绝对路径。

## 逆向协议

电源模式：

```text
读取: ReadAppSettings(1, 1, 1)
能力: Feature package 偏移 17 的低 4 位
写入: OutLog 事件 "clevocc30^101^<mode>"
等待: 最多 5 秒轮询第 1 页偏移 1 的回读值
```

功能包偏移 17 的 bit0 至 bit3 分别对应静音、节能、性能与娱乐。界面仅显示
固件声明支持的选项，写入前也会再次校验对应能力位。部分 BIOS 在 WMI 16
声明电源模式能力，却将功能包位图返回为 0；此时仅回退到 911MT 实机已验证的
静音与性能模式，不启用未验证配置。

风扇模式：

```text
SetDCHU_Data(121, subcommand=1, mode)
WriteAppSettings(4, 5, 1, [mode])
```

风扇偏移：

```text
raw = round(255 * offset / 100)
SetDCHU_Data(121, subcommand=14, raw)
WriteAppSettings(4, 7, 1, [offset])
```

功能键：

```text
读取: SetDCHU_DataEx(4, [24])
写入: SetDCHU_DataEx(4, [25, fnLockMode, winKeyMode, winFnMode])
持久化: WriteAppSettings(1, 36, 3, [fnLockMode, winKeyMode, winFnMode])
```

FlexiCharger：

```text
读取: SetDCHU_DataEx(4, [30])
写入: SetDCHU_DataEx(4, [31, enabled, startPercent, stopPercent])
```

低刷新率：

```text
支持: ReadAppSettings(1, 27, 9)[0] == 1
当前: ReadAppSettings(1, 27, 9)[1] == 1
写入: WriteAppSettings(1, 28, 1, [0|1])
```

蓝光过滤：

```text
支持: WMI 16 & 0x0800
读取: ReadAppSettings(1, 27, 9)[7]
写入: WriteAppSettings(1, 34, 1, [round((factor - 1.0) * 10)])
```

WMI 12 遥测偏移：

| 数据 | 偏移 | 编码 |
| --- | ---: | --- |
| CPU / GPU1 / GPU2 转速计 | 2 / 4 / 6 | 大端 `u16` |
| CPU / GPU1 / GPU2 占空比 | 16 / 19 / 22 | `round(raw / 255 * 100)` |
| CPU / GPU1 / GPU2 温度 | 18 / 21 / 24 | 摄氏度 |
| RPM | - | `round(2,156,250 / raw)` |

## 功能边界

`InsydeDCHU.dll` 只导出 6 个底层函数，CC30 Nova 已全部加载：
`GetDCHU_Data_Integer`、`GetDCHU_Data_Buffer`、`ReadAppSettings`、
`WriteAppSettings`、`SetDCHU_Data` 和 `SetDCHU_DataEx`。界面功能是在这些
底层函数之上实现经过验证的固定协议，不提供任意命令入口。

| 状态 | 功能 |
| --- | --- |
| 已开放写入 | 电源模式、自动/最大风扇、风扇偏移、键盘设置、FlexiCharger、低刷新率、蓝光过滤 |
| 已开放只读 | 实时温度与转速、风扇曲线、GPU 模式、电池固件、热策略遥测、原生 API 诊断 |
| 外部厂商组件 | 键盘 RGB、Flexikey、Battery Utility、Fan Speed Setting 由 CLEVO UWP 应用实现，不在 DCHU 导出协议内 |
| 当前实机未声明 | 风扇除尘、DTT、Turbo Fan、Slow Fan、Max-Q、Fanless、PgUp/PgDn、单键 RGB |
| 暂不开放写入 | GPU 模式切换需要重启并完成整机显示链路验证 |
| 安全禁用 | CPU/GPU 超频、AMD 手动调频、电池校准 |

## 开发环境

要求：

- Windows 11 x64
- Rust stable，MSVC 目标
- Node.js 22 或更高版本
- Tauri 2 依赖与 Microsoft Edge WebView2

安装依赖并启动：

```powershell
npm install
npm run tauri dev
```

仅预览网页界面：

```powershell
npm run dev
```

网页预览无法调用 Tauri 命令，因此会自动使用模拟数据。

验证：

```powershell
npm run build
Set-Location src-tauri
cargo fmt --check
cargo test
cargo check
```

构建发行包：

```powershell
npm run tauri build
```

安装包默认生成在 `src-tauri/target/release/bundle`。

## 隐私

SMBIOS 标识与硬件信息仅在本地读取并展示。CC30 Nova 不包含遥测、网络上传或
远程控制路径。
