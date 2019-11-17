# Flare Profiler
一个基于JVMTI技术的Java诊断工具，支持快速分析JVM线程CPU时间及方法调用栈。


### 关键特性
- 支持带时序的方法调用栈分析，直观准确反映任意一次请求执行的过程
- 整合了Chrome浏览器开发工具的火焰图组件，支持鼠标缩放、拖动查看方法调用栈，操作流畅，高效快捷
- 支持实时分析和离线分析，自动分卷保存取样结果
- 支持多会话，可多人同时使用，可同时接入多个Agent或打开存档
- 使用Rust语言编写，运行稳定，资源占用低，只需不到20MB内存，适合在容器内或者开发环境使用
- 支持多平台（Windows, Linux and macOS）

### 原理介绍

Flare Profiler 系统交互图如下：  
![系统交互图](doc/design/flare-profiler-demployment.png)  
  

##### 主要功能模块  
- flare-agent: 基于JVMTI技术注入JVM进程中运行，负责进行周期采样获取线程CPU时间及线程方法调用栈等数据。后续会增加对象分配、锁检查、GC活动等功能。
- flare-server: 查询分析服务，负责收集采样数据并保存到本地文件，提供查询分析服务接口，如获取线程CPU时间统计图，获取某个时间范围的线程方法调用栈统计数据等。
- flare-ui: 诊断分析交互界面，主要包括展示线程CPU时间图表、方法调用栈火焰图等。
  
### 功能对比
以下对比常见工具的JVM CPU 诊断功能：   
   
| 项目      | JProfiler |  Async Profiler | Flare Profiler |
| -------- | --------: | :-------------: |:------------:  |
| 内存占用   | 大于512MB   |   ？     |  小于20MB     |
| 火焰图     |   不支持 | 支持，静态SVG | 支持，动态缩放、拖动查看   |
| 方法调用时序 |  不支持 |  不支持 |  支持直观查看方法执行过程  |
| 实时/离线分析 |  支持实时查看，Agent断开后不能分析 |  不支持实时查看 |  支持实时和离线分析  |
| 稳定性 | 压测时JVM可能Crash | ？ | 压测时仍然稳定运行 |
| 兼容性 | 支持多种平台 | 不支持Windows | 支持Windows、Linux、macOS |


### 常见问题
1、可能超出最大可打开的文件数量
save summary info failed: Too many open files (os error 24)  
原因：  
   Flare Profiler为每个JVM线程创建两个文件，如果JVM线程太多，将导致打开或者保存文件失败。  
解决办法：  
   ulimit -a #查看系统的文件句柄数量设置，如果太少，请改大一点  
   macOS参考命令:  
   ```bash
   sudo launchctl limit maxfiles 999999999 999999999  
   sudo ulimit -n 65535  
   ulimit -n 65535  
   ```
   
2、Call Graph标签页加载异常  
火焰图组件来源于Chrome浏览器调试工具项目（devtools），仅支持在Chrome浏览器77以上版本运行。


Links:  
[v0.1.0-alpha 版本介绍](https://github.com/kylixs/kylixs.github.io/blob/master/flare-profiler-v0.1.0-alpha-demo.md)
