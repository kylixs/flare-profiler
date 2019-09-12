###5.1 取样数据保存格式  

对于每个线程的数据分为两个文件，一个记录时序数据，一个记录具体的调用栈数据。

####1）取样汇总信息文件 (summary)

json格式，记录本次取样的汇总统计信息。

####2）时序数据存储格式

时序数据文件：
```
|头部信息| x|x|x |x...
```
头部信息包含： 开始时间，最后时间，数据类型（固定长度，如int32， long64）

每个时序数据的数据范围最长为1个小时，超过后自动产生新的数据文件，类似日志文件分卷处理。

线程CPU时间精确到微秒，存储的数据类型为u16，最大值为2^16 / 1000 = 65.36ms，大于采样间隔时间（10~50ms），可以满足需要。

假设采用频率20ms，1小时时序数据大小： header len + 2 * 1000/20 * 3600 / 1024 ≈ 352KB

头部信息格式：
```
文件标记（magic 4bytes）|header len (2 bytes) | header fields(n bytes)|
```

header fields 格式：
```
unit type (1byte)| unit size (1bytes) | begin_time(8 bytes) | end_time(8 bytes)| count (4 bytes)|
```



####3）调用栈数据

与时序数据不同，调用栈为不定长数据，采用索引+数据文件的方式存储。索引文件并不是记录每个调用栈数据的偏移位置，只需要每个单位时间记录一个即可。

每隔一个单位时间（如1s，10s）记录一个索引信息： （时间，偏移位置）  =》 修改为 （时序step，偏移位置），避免调用栈时间与cpu时序的时间不一致

读取范围数据：  
 1) 通过二分查找索引记录，定位到选择时间的前一个索引作为开始处理位置。然后读取数据流，从开始位置遍历数据，skip 时间范围之外的数据。
 2) 转换开始时间和结束时间为cpu时序step，读取到开始与结束的数据偏移位置，然后到数据文件中批量读取两个偏移位置的数据。 


索引文件：
```
|头部信息|（时序step，偏移位置）|（时序step，偏移位置）|（时序step，偏移位置）|（时序step，偏移位置）..
```
 
数据文件：
```
|头部信息|调用栈数据|调用栈数据|调用栈数据
```


####4）方法信息数据

调用栈保存的是方法id，具体的方法签名单独保存到方法信息数据文件。

索引文件：
```
|头部信息|（method_id，偏移位置）|（method_id，偏移位置）|（method_id，偏移位置）|（method_id，偏移位置）..
```
 
数据文件：
```
|头部信息|方法信息|方法信息|方法信息
```


###6. Flare UI交互接口
Flare UI 通过WebSocket协议发送查询分析指令到Flare Client， Flare Client根据指令读取相应的数据文件进行统计分析，然后返回结果。
请求及响应都为json格式，通用格式如下：
```json
{
   "cmd": "cmd_name",
   "options" : {
      "sample_instance": "localhost:2233",
       ...
    }
}
```
```json
{
   "result": "success",
   "message": "",
   "data" : {
      "sample_instance": "localhost:2233",
      ...
   }
}
```
注意：
FlareUI支持打开多个取样实例，sample_instance为全局唯一的实例标识符，实例相关的操作都需要指定此参数

####1）列出所有会话
列出当前打开的所有会话
```json
{
   "cmd": "list_sessions",
   "options" : {
    }
}
```
响应结果：
```json
{
   "result": "success",
   "cmd": "list_sessions",
   "data": {
        "sample_sessions": [{
            "session_id": "localhost_2233_01",
            "type": "file"
        },{
            "session_id": "localhost:2233",
            "type": "attach"
        }]
   }
}
```

列出历史取样目录:
```json
{
   "cmd": "history_samples",
   "options" : {
    }
}
```
响应结果：
```json
{
   "result": "success",
   "cmd": "history_samples",
   "data": {
        "history_samples": [{
            "path": "localhost_2233_01",
            "type": "file"
        },{
            "path": "localhost_2233_02",
            "type": "file"
        },{
            "path": "localhost_2233_03",
            "type": "file"
        }]
   }
}
```

####2）打开取样数据
打开指定的取样数据目录，返回创建取样实例ID。
```json
{
   "cmd": "open_sample",
   "options" : {
        "sample_data_dir": "D:/flare-samples/localhost_2233_01"
    }
}
```
响应结果：
```json
{
   "result": "success",
   "cmd": "open_sample",
   "data": {
        "session_id": "localhost_2233_01"
   }
}
```
####3）启动取样，注入目标进程
注入指定Java进程，返回创建取样实例ID。
```json
{
   "cmd": "attach_jvm",
   "options" : {
       "target_pid": 1234,
       "sample_interval_ms": 20,
       "sample_duration_sec": 300
    }
}
```
响应结果：
```json
{
   "result": "success",
   "cmd": "attach_jvm",
   "data": {
      "session_id": "localhost:2233"
   }
}
```


连接指定Flare Agent端口，返回创建取样实例ID。
```json
{
   "cmd": "connect_agent",
   "options" : {
       "agent_addr": "localhost:3344"
    }
}
```
响应结果：
```json
{
   "result": "success",
   "cmd": "connect_agent",
   "data": {
      "session_id": "localhost:3344"
   }
}
```

####4）停止取样，关闭目标Agent端口
```json
{
   "cmd": "stop_sample",
   "options" : {
      "session_id": "localhost:2233"
    }
}
```

####5）获取Dashboard
包含线程列表、JVM信息
```json
{
   "cmd": "dashboard",
   "options" : {
      "session_id": "localhost:2233"
    }
}
```
响应结果：
```json
{
   "result": "success",
   "cmd": "dashboard",
   "data": {
      "session_id": "localhost:2233",
      "time": "20190905 15:41:24",
      "threads": [{
          "id" : 132,
          "name": "DiscoveryClient-1",
          "group": "main",
          "priority": 1,
          "state": "RUNNING",
          "%cpu" : "20.1",
          "cpu_time" : "1:21",
          "daemon": false
      }],
      "jvm_info": {}
   }
}
```

####6）获取线程的CPU时间趋势数据
获取指定时间范围的线程CPU时间趋势数据
```json
{
   "cmd": "cpu_ts",
   "options" : {
      "session_id": "localhost:2233",
      "thread_ids": [], // 为空时获取全部线程
      "start_time": 1567669466207,
      "end_time": 1567669485649,
      "graph_width": 900 
    }
}
```
响应结果：
```json
{
   "result": "success",
   "cmd": "cpu_ts",
   "data": {
      "session_id": "localhost:2233",
      "threads": [{
          "id": 132,
          "name": "DiscoveryClient-1",
          "start_time": 1567669466207,
          "end_time": 1567669485649,
          "unit_time_ms": 1000,
          "cpu_time_ms": 2342,
          "ts_data": [10,2,0,0,2,4] 
      }]
   }
}
```
