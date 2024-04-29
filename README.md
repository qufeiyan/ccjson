# ccjson 

![license](https://img.shields.io/badge/license-MIT-orange)
![prs](https://img.shields.io/badge/PRs-welcome-brightgreen)
![poweredby](https://img.shields.io/badge/powered%20by-qufeiyan-red)

> 实现一个命令行工具，以生成 compile_commands.json, 便于 `clangd` 识别, 搭配 `vscode` | `vim` 等编辑器以提供现代 `ide` 的智能提示、代码补全等服务。

原项目由 [python3](https://github.com/qufeiyan/compilecmds) 实现, 虽然能满足自用, 且完美解决使用非标准 `make` 构建的 `c` 项目无法使用 `clangd` 的问题, 但是在离线环境下安装对应的 `python` 包依赖对不熟悉 `python` 的人来说并不是那么容易。本次实现的 `ccjson` 以 `rust` 重构, 直接生成二进制可执行程序，以解决工具安装问题。

### 需求

- 方便易用，支持解析编译命令以正确生成 `compile_commands.json` 文件

- 支持以管道、文件方式读入 `build` 日志进行解析

### 实现方案

数据流为:

```bash
reader --> parser --> writer 
```

### 安装

构建 `linux` 上可执行文件

1. 使用 `rustup` 添加 `x86_64` 架构 `linux`（基于 `musl c` 库）的目标平台

```bash
rustup target add x86_64-unknown-linux-musl
```
2.  执行以下命令进行编译，生成适用于 `linux` 的 `release` 版本可执行文件

```bash
cargo build --release --target x86_64-unknown-linux-musl
```

注意: 如果使用的是 `macos`, 还需要指定 `musl c` 的链接器，先安装交叉编译工具链

```bash
brew install FiloSottile/musl-cross/musl-cross
```

并在项目的 `.cargo/config.toml` 文件中指定 `musl` 链接器路径

```bash
[target.x86_64-unknown-linux-musl]
linker = "x86_64-linux-musl-gcc"
```

### 使用

安装后，使用 `ccjson` 命令输出`compile_commands.json` 文件

1. 管道方式, 一边编译，一边生成

```bash
$(make_script) | ccjson 
```

2. 读取编译日志

```bash
ccjson -p $(build.log) -d $(build_dir)
```




