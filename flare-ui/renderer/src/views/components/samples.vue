<template>
    <div class="samples">
        <div>
            <p>Start Sample:
                <el-input v-model="agent_addr" style="width: 300px">
                    <!--<template slot="append" @click="connect_agent">Connect</template>-->
                </el-input>
                <el-button class="ml10" @click="connect_agent">Connect</el-button>
            </p>
        </div>
        <div style="margin-top: 10px">
            <el-button @click="list_sessions">Sessions</el-button>
            <el-button @click="list_history">History</el-button>
            <el-button @click="stop_auto_refresh">Stop Refresh</el-button>
            <el-button @click="close_session">Close</el-button>

            <div style="margin-top: 10px">
                <p>History samples:</p>
                <ul class="list-content">
                    <li class="list-item" v-for="sample in history_samples_list">
                        <el-link :underline="false" @click='open_sample(sample.path)'>[{{sample.type}}]{{sample.path}}</el-link>
                    </li>
                </ul>
            </div>
            <!--<div class="list-div" style="margin-top: 10px">
                <p>Sample sessions:</p>
                <ul class="list-content">
                    <li class="list-item" v-for="session in data.sample_sessions"
                        @click="active_session(session.session_id, session.type)">
                        [{{session.type}}]{{session.session_id}}
                    </li>
                </ul>
            </div>-->
        </div>

        <!--<div class="message" v-show="show_message">
            &lt;!&ndash;				<p>操作指令：{{profiler.data.cmd}}</p>&ndash;&gt;
            <p>错误信息：{{data.message}}</p>
        </div>
        <div style="margin-top: 10px">
            <p>Sample Session: {{data.session_id}} ({{data.type}})</p>
            <p>Sample Start Time: {{data.sample_info.sample_start_time}}</p>
            <p>Record Start Time: {{data.sample_info.record_start_time}}</p>
            <p>Last Record Time: {{data.sample_info.last_record_time}}</p>
            <p>Record Duration: {{(data.sample_info.last_record_time - data.sample_info.record_start_time)/1000}}s</p>
        </div>-->
    </div>
</template>

<script>
    export default {
        name: 'samples',
        data() {
            return {
                agent_addr: "localhost:3333",
                profiler_addr: "localhost:3344",//
                show_history_samples: false,
                show_sessions: false,
                history_samples_list: [],
                sample_sessions_list: [],
                show_message: false,
                dashboard_timer: null,

                data: {
                    sample_info: {},
                    threads: [],
                    history_samples: [],
                    sample_sessions: [],
                    thread_cpu_time_map: {},
                    session_id: "",
                    type: "",
                    call_tree_data: [{
                        id: 1,
                        label: '方法调用',
                        children: [{
                            id: 4,
                            label: '二级 1-1'
                        }]
                    }],

                    flame_graph_svg: "",
                    flame_graph_data: ""
                },

                webSocket:"",
            }
        },
        mounted() {
          //this.webSocketInit();
            this.$webSocket.webSocketInit();
            this.$ws.onmessage = this.onmessage;
            //this.$ws.onopen = this.webSocketOnOpen();
            //this.$ws.onclose = this.webSocketOnClose();
            //this.$ws.onerror = this.webSocketOnError();
        },
        computed: {
            sessionOptions() {
                return this.$store.state.sessionOptions;
            },
            sessionThreads() {
                return this.$store.state.sessionThreads;
            },
            sessionCpuTimes() {
                return this.$store.state.sessionCpuTimes;
            },
            sessionFlameGraph() {
                return this.$store.state.sessionFlameGraph;
            },
            sampleInfo() {
                return this.$store.state.sampleInfo;
            },
            sessionSampleInfo() {
                return this.$store.state.sessionSampleInfo;
            },
            exampleInfo() {
                return this.$store.state.exampleInfo;
            },
            historySamples() {
                return this.$store.state.historySamples;
            },
        },
        created() {
            this.getHistorySamples();
        },
        methods: {
            getHistorySamples() {
                if (this.historySamples) {
                    this.history_samples_list = this.historySamples;
                }
            },
            /*websocket*/
            webSocketOnOpen(){
                console.log("websocket建立连接");
            },
            webSocketOnClose(){
                console.log("websocket销毁连接");
            },
            webSocketOnError(){
                console.log("websocket连接失败");
            },
            onmessage(event){
                //console.log('websocket接收到消息：', event);
                var json = JSON.parse(event.data);
                var success = (json.result == "success");
                Object.assign(this.data, json.data)
                this.show_message = !success;
                if (!success) {
                    this.stop_auto_refresh();
                }

                let sessionId = json.data.session_id
                this.$store.commit('example_info', json.data);
                switch (json.cmd) {
                    case "dashboard":
                        let threadSessionId = json.data.sample_info.sample_data_dir;
                        if (json.data.threads) {
                            let threadsArray = this.sessionThreads.filter(item => {
                                if (item.sessionId != threadSessionId) {
                                    return item;
                                }
                            });

                            let threadsInfo = {
                                sessionId: threadSessionId,
                                threads: json.data.threads,
                                threadSampleInfo: {...json.data.sample_info}
                            };
                            threadsArray.push(threadsInfo);
                            this.$store.commit('session_threads', threadsArray);
                        }

                        let sessionSampleInfoArray = this.sessionSampleInfo.filter(item => {
                            if (item.sessionId != threadSessionId) {
                                return item;
                            }
                        });
                        let sessionSampleInfo = {sessionId: threadSessionId, sessionSample: json.data.sample_info};
                        sessionSampleInfoArray.push(sessionSampleInfo)

                        this.$store.commit('session_sample_info', sessionSampleInfoArray);
                        this.$store.commit('sample_info', json.data.sample_info);
                        break;
                    case "open_sample":
                        this.start_auto_refresh();
                        break;
                    case "connect_agent":
                        this.start_auto_refresh();
                        break;
                    case "history_samples":
                        console.log('json.data.history_samples', json.data.history_samples)
                        if (json.data.history_samples) {
                            this.$store.commit('history_samples', json.data.history_samples);
                        }
                        this.show_history_samples = true;
                        break;
                    case "list_sessions":
                        // session tag
                        if (json.data.sample_sessions) {
                            this.$store.commit('session_options', json.data.sample_sessions)
                        }
                        this.show_sessions = true;
                        break;
                    case "cpu_time":
                        let sessionCpuTimeArray = [];
                        let cpuTimeArray = this.sessionCpuTimes.filter(item => {
                            if (item.sessionId != sessionId) {
                                return item;
                            } else {
                                item.cpuTimeData.forEach(item1 => {
                                    if (!sessionCpuTimeArray.includes(item1.id)) {
                                        sessionCpuTimeArray.push(item1);
                                    }
                                })
                            }
                        });
                        json.data.thread_cpu_times.forEach(item => {
                            if (!sessionCpuTimeArray.includes(item.id)) {
                                sessionCpuTimeArray.push(item);
                            }
                        })

                        let cpuTimeInfo = {sessionId: sessionId, cpuTimeData: sessionCpuTimeArray};
                        cpuTimeArray.push(cpuTimeInfo);

                        this.$store.commit('session_cpu_times', cpuTimeArray);
                        break;
                    case "call_tree":
                        break;
                    case "flame_graph":
                        if (json.data.flame_graph_data) {
                            let graphInfoList = [];
                            let flameGraphList = this.sessionFlameGraph.filter(item => {
                                if (item.sessionId != sessionId) {
                                    return item;
                                } else {
                                    item.flameGraphList.filter(item1 => {
                                        if (item1.threadId != json.data.thread_id) {
                                            graphInfoList.push(item1);
                                        }
                                    })
                                }
                            });

                            graphInfoList.push({threadId: json.data.thread_id, flameGraphData: json.data});
                            let graphInfo = {
                                sessionId: sessionId,
                                flameGraphList: graphInfoList
                            };
                            flameGraphList.push(graphInfo);
                            this.$store.commit('session_flame_graph', flameGraphList)
                        }
                        //profiler.data.flame_graph_svg="data:image/svg+xml;utf8,"+json.data.flame_graph_data.replace(/<\?xml.*?\>.*\<!DOCTYPE.*\<svg/, "<svg");
                        break;
                    default:
                        console.log("unknown message: ", json);
                        break;
                }

            },
            /*webSocketSendMessage(msg){
                this.webSocket.send(msg);
                console.log("websocket发送消息：" + msg);
            },*/
            connect_agent: function () {
                this.clear_session();
                var request = {
                    "cmd": "connect_agent",
                    "options": {
                        "agent_addr": this.agent_addr
                    }
                };
                this.$webSocket.webSocketSendMessage(JSON.stringify(request));
                this.connected = true;
            },
            close_session() {
                this.connected = false;
                var request = {
                    "cmd": "close_session",
                    "options": {
                        "session_id": this.sampleInfo.sample_data_dir
                    }
                };
                this.$webSocket.webSocketSendMessage(JSON.stringify(request));
                let sessionOption = this.sessionOptions.filter((item => {
                    if (item.session_id != this.sampleInfo.sample_data_dir) {
                        return item;
                    }
                }))
                this.$store.commit('session_options', sessionOption);
            },
            clear_session: function () {
                this.stop_auto_refresh();
                this.data.session_id = "";
                this.data.threads = [];
                this.data.sample_info = {};
                this.data.thread_cpu_time_map = {};
            },
            list_sessions() {
                this.show_sessions = true;
                this.show_history_samples = false;
                this.$webSocket.webSocketSendMessage(JSON.stringify({
                    "cmd": "list_sessions"
                }))
            },
            list_history: function () {
                this.show_sessions = false;
                this.show_history_samples = true;
                this.$webSocket.webSocketSendMessage(JSON.stringify({
                    "cmd": "history_samples"
                }))
            },
            start_auto_refresh() {
                if (this.dashboard_timer == null) {
                    this.do_refresh();
                    if(this.data.type == "attach") {
                        this.dashboard_timer = setInterval(function () {
                            this.do_refresh();
                        }, 2000);
                    }
                }
            },
            stop_auto_refresh() {
                if (this.dashboard_timer != null) {
                    clearInterval(this.dashboard_timer);
                    this.dashboard_timer = null;
                }
            },
            update_dashboard(){
                console.log("send request: get_dashboard");
                this.$webSocket.webSocketSendMessage(JSON.stringify({
                    "cmd": "dashboard",
                    "options": {
                        "session_id": this.data.session_id
                    }
                }));
            },
            update_cpu_time(){
                var thread_ids = [];
                for ( var i=0;i<this.data.threads.length;i++) {
                    thread_ids.push(this.data.threads[i].id);
                }
                var graph_width = 900;
                var sample_interval = this.data.sample_info.sample_interval;
                var start_time = this.data.sample_info.record_start_time;
                var end_time = this.data.sample_info.last_record_time;
                var ratio = Math.ceil((end_time - start_time) / graph_width / sample_interval);
                if (ratio > 10 ){
                    ratio = Math.floor(ratio/10)*10;
                }
                var unit_time_ms = ratio * sample_interval;
                this.data.sample_info.unit_time_ms = unit_time_ms;

                var request = {
                    cmd: "cpu_time",
                    options: {
                        "session_id": this.data.session_id,
                        "thread_ids": thread_ids,
                        "start_time": start_time,
                        "end_time": end_time,
                        "unit_time_ms": unit_time_ms
                    }
                };
                this.$webSocket.webSocketSendMessage(JSON.stringify(request));
            },
            do_refresh(){
                if (this.data.session_id == ""){
                    return;
                }
                this.update_dashboard();
                setTimeout(() => {
                    this.update_cpu_time();
                }, 500);
            },
            open_sample: function (sample_data_dir) {
                this.clear_session();
                var request = {
                    "cmd": "open_sample",
                    "options": {
                        "sample_data_dir": sample_data_dir
                    }
                };
                this.$webSocket.webSocketSendMessage(JSON.stringify(request));
                this.list_sessions()
            },
            active_session:function (session_id, type) {
                this.clear_session();
                this.data.session_id = session_id;
                this.data.type = type;
                this.start_auto_refresh();
            },
        },
        watch: {
            historySamples() {
                this.history_samples_list = this.historySamples;
            }
        }
    }
</script>

<style scoped>

</style>