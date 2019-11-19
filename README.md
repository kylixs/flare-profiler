# Flare Profiler
[English Document](README_EN.md)    

一个基于JVMTI技术的Java诊断工具，支持快速分析JVM线程CPU时间及方法调用栈，集成了灵活好用的动态火焰图组件，Java服务性能优化利器。  

当你遇到以下类似的问题时，Flare Profiler可以帮助你解决问题： 
- Java服务在高并发时存在瓶颈，吞吐量较低，无从下手
- 某些计算任务耗时很长，处理逻辑非常复杂，难以分析
- 线上服务突然变慢，不知道是什么原因导致
- 某些比较复杂的开源项目或者框架代码分支太多，通过调试来分析比较麻烦
- 作为架构师，想将性能调优工作下放到业务组，缺少一个简单好用的工具



Flare Profiler 0.1.0 只是一个雏形，有很多好点子还没有细化下来，想收集了解大家在Java性能调优过程遇到的一些疑难问题。  
接下来可能做的功能：  
- 完整的带时序的方法调用树，面向学习分析开源项目（区别于通过定时取样获取的部分方法调用栈，而是通过监控方法进入退出获取到完整的带时序的方法调用树，方便回溯分析任意代码分支）
- 某个时刻的线程调用栈切面，方便分析资源竞争问题
- 分析线程锁等待时间
- 分析对象分配情况（计划参考Yourkit Java Profiler的功能，关联创建对象到发生的调用栈上，这个功能非常实用）
  
**关于github下载慢的问题，建议加QQ群（837682428），从群共享文件下载。**

使用文档：  
[入门指南 Quick Start](doc/quick-start.md)


### 关键特性
- 支持带时序的方法调用栈分析，直观准确反映任意一次请求执行的过程
- 整合了Chrome浏览器开发工具的火焰图组件，支持鼠标缩放、拖动查看方法调用栈，操作流畅，高效快捷
- 支持实时分析和离线分析，自动分卷保存取样结果
- 支持多会话，可多人同时使用，可同时接入多个Agent或打开存档
- 使用Rust语言编写，运行稳定，资源占用低，只需不到20MB内存，适合在容器内或者开发环境使用
- 支持多平台（Windows, Linux and macOS）

### 原理介绍

Flare Profiler 系统交互图如下：  
![系统交互图](doc/design/flare-profiler-interaction.png)  
  

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
~~1、可能超出最大可打开的文件数量~~(**此问题已经解决**)  
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

   
2、方法调用栈火焰图（Call Graph标签页）加载异常  
火焰图组件来源于Chrome浏览器调试工具项目（devtools），仅支持在Chrome浏览器77以上版本运行，如果出现显示异常请升级Chrome到最新版本或者使用Protable版本。


Links:  
- [入门指南 Quick Start](doc/quick-start.md)  
- [v0.1.0-alpha 版本介绍](https://github.com/kylixs/kylixs.github.io/blob/master/flare-profiler-v0.1.0-alpha-demo.md)

**Flare Profiler 开源交流QQ群： 837682428，欢迎加群一起探讨学习Java & Rust !**  
![Flare Profiler 开源交流QQ群： 837682428](doc/flare-profiler-qq-group.png)  
