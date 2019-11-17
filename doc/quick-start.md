## Java Flare Profiler 入门指南

#### 1. 启动Java应用
启动待分析的Java应用，如Tomcat应用、SpringBoot应用、ElasticSearch、Spark。  
建议启动一个可以压测的Web应用，或者有性能瓶颈的应用。

#### 2. 启动Flare Profiler
##### 1) 启动Flare Server
```shell script
wget https://xxx/flare-profiler-linux.tar.gz
tar -xvf flare-profiler-linux.tar.gz
cd flare-profiler-linux
./startup.sh
```

##### 2) 启动Flare Agent
通过命令找到第1步启动的待分析Java应用的PID
```shell script
ps -ef | grep java
```
##### 3) 连接Flare Agent

#### 3. 查看会话列表及历史存档

#### 4. 查看Dashboard

#### 5. 查看线程CPU时间统计图

#### 6. 分析线程方法调用栈

#### 7. 停止Flare Profiler
##### 1) 关闭会话

##### 2) 停止Flare Agent

##### 3) 停止Flare Server


