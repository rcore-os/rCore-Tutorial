# rCore-Tutorial V3（开发中）

[![Actions Status](https://github.com/rcore-os/rCore-Tutorial/workflows/CI/badge.svg?branch=master)](https://github.com/rcore-os/rCore-Tutorial/actions)

[本教学仓库](https://github.com/rcore-os/rCore-Tutorial)是继 [rCore_tutorial V2](https://rcore-os.github.io/rCore_tutorial_doc/) 后重构的 V3 版本。

本文档的目标主要针对「做实验的同学」，我们会对每章结束后提供完成的代码，你的练习题只需要基于我们给出的版本上增量实现即可，不需要重新按照教程写一遍。

而对想完整实现一个 rCore 的同学来说，我们的文档可能不太友好。因为在编写教程过程中，我们需要对清晰和全面做很多的权衡和考虑、需要省略掉大量 Rust 语法层面和 OS 无关的代码以带来更好的可读性和精简性，所以想参考本文档并完整实现的同学可能不会有从头复制到尾的流畅（这样的做法也不是学习的初衷），可能需要自己有一些完整的认识和思考。

另外，如果你觉得字体大小和样式不舒服，可以通过 GitBook 上方的按钮调节。

## 仓库目录

- `docs/`：教学实验指导分实验内容和开发规范
- `notes/`：开题报告和若干讨论
- `os/`：操作系统代码
- `user/`：用户态代码
- `SUMMARY.md`：GitBook 目录页
- `book.json`：GitBook 配置文件
- `rust-toolchain`：限定 Rust 工具链版本
- `deploy.sh`：自动部署脚本
<!-- Rust 工具链版本需要根据时间更新 -->

## 实验指导

基于 GitBook，目前已经部署到了 [GitHub Pages](https://rcore-os.github.io/rCore-Tutorial-deploy/) 上面。

### 文档本地使用方法

<!-- 下面的代码不再使用标签，因为也同时会渲染到 GitHub 的项目首页 -->
```bash
npm install -g gitbook-cli
gitbook install
gitbook serve
```

## 代码

### 操作系统代码
本项目基于 cargo 和 make 等工具，在根目录通过 `make run` 命令即可运行代码，更具体的细节请参见 `Makefile`、`os/Makefile` 以及 `user/Makefile`。

### 参考和感谢

本文档和代码部分参考了：
- [rCore](https://github.com/rcore-os/rCore)
- [zCore](https://github.com/rcore-os/zCore)
- [rCore_tutorial V2](https://rcore-os.github.io/rCore_tutorial_doc/)
- [使用Rust编写操作系统](https://github.com/rustcc/writing-an-os-in-rust)

在此对仓库的开发和维护者表示感谢，同时也感谢很多在本项目开发中一起讨论和勘误的老师和同学们。

<!-- TODO LICENSE -->
