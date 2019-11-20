## Flare Agent 常见问题

#### 1. 在Linux上启动Agent失败
如果提示以下错误，可能是执行注入脚本的用户身份与Java进程用户不同导致：
```bash
#启动注入脚本
>cd /pathflare-profiler/agent/
>./bin/start-trace-agent.sh 15110
PROJECT_PATH:/opt/apps/flare-profiler/agent
AGENT_PATH: /opt/apps/flare-profiler/agent/lib/libflareagent.so
AGENT_OPTS: trace=on,interval=5
TARGET_PID: 15110
[INFO] agentPath: /opt/apps/flare-profiler/agent/lib/libflareagent.so
[INFO] options: trace=on,interval=5

Exception in thread "main" com.sun.tools.attach.AttachNotSupportedException: Unable to open socket file: target process not responding or HotSpot VM not loaded
	at sun.tools.attach.LinuxVirtualMachine.<init>(LinuxVirtualMachine.java:106)
	at sun.tools.attach.LinuxAttachProvider.attachVirtualMachine(LinuxAttachProvider.java:78)
	at com.sun.tools.attach.VirtualMachine.attach(VirtualMachine.java:250)
	at com.kylixs.jvmti.attacher.AgentAttacher.attachAgent(AgentAttacher.java:27)
	at com.kylixs.jvmti.attacher.AgentAttacher.main(AgentAttacher.java:72)
```
解决办法：  
(a) su - <user> 切换到Java进程相同用户身份，再执行脚本，但可能出现该用户禁止登录的问题  
(b) 使用/usr/bin/su指定执行用户身份
```
/usr/bin/su - <username>  -s /bin/sh -c "/path/flare-profiler/agent/bin/start-trace-agent.sh 15110 & "
```
(c) 使用封装好的run-agent-as-user.sh脚本：
```
>cd /pathflare-profiler/agent/
>./bin/run-agent-as-user.sh <username> start 15110

```

#### 2. 在Windows上注入以服务运行的Java/Tomcat进程失败  
使用注入服务的脚本  
```
  cd flare-profiler/agent/bin
  start-agent-as-service.bat <pid>
```
停止Agent执行下面的脚本   
```
  cd flare-profiler/agent/bin
  stop-agent-as-service.bat <pid>
```


