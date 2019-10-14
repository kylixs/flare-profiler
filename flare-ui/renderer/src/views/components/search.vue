<template>
    <div class="cpu_time_content" style="width: 100%"><!--{{threads}}-->
        <el-form :inline="true" :model="formInline" class="demo-form-inline">
            <el-form-item label="关键字">
                <el-input v-model="formInline.keyword" placeholder="关键字"></el-input>
            </el-form-item>
            <el-form-item label="时间">
                <el-input v-model="formInline.times" placeholder="时间"></el-input>
            </el-form-item>
            <el-form-item>
                <el-button type="primary" @click="">查询</el-button>
            </el-form-item>
        </el-form>
        <el-table ref="cpuTable" :data="threads" highlight-current-row @row-click="selectCurRow" style="cursor: pointer">
            <el-table-column width="400">
                <template slot-scope="scope">
                    <span>{{scope.row.name}}</span>
                </template>
            </el-table-column>
            <el-table-column>
                <template slot-scope="scope">
                    <div class="thread_bar" v-bind:id="'thread_cpu_chart_' + scope.row.id+''"></div>
                </template>
            </el-table-column>
        </el-table>
    </div>
</template>

<script>
    export default {
        name: 'cpu',
        data() {
            return {
                threads: [],
                selected_thread_id: null,
                formInline: {
                    keyword: '',
                    times: ''
                }
            }
        },
        computed: {
            sampleInfo() {
                return this.$store.state.sampleInfo;
            },
            sessionSampleInfo() {
                return this.$store.state.sessionSampleInfo;
            },
            exampleInfo() {
                return this.$store.state.exampleInfo;
            },
            sessionThreads() {
                return this.$store.state.sessionThreads;
            },
            sessionId() {
                return this.$route.params.sessionInfo;
            },
            sessionCpuTimes() {
                return this.$store.state.sessionCpuTimes;
            },
            historySamples() {
                return this.$store.state.historySamples;
            },
            sessionTabsValue() {
                return this.$store.state.sessionTabsValue;
            },
            selectCpuRow() {
                return this.$store.state.selectCpuRow;
            },
            sessionCallTabs() {
                return this.$store.state.sessionCallTabs;
            },
        },
        mounted(){
            this.$nextTick(()=>{
                this.on_cpu_time_result();
            })
        },
        created(){
            this.getThreads();
        },
        methods: {
            selectCurRow(row, column, event){

                let tabsValueArray = this.sessionTabsValue.filter(item => {
                    if (item.sessionId != this.sessionId) {
                        return item
                    }
                });
                let tabsInfo = {sessionId: this.sessionId, tabsValue: row.id + '', routerValue: '/call/' + row.id}
                tabsValueArray.push(tabsInfo);

                let cpuRowList = [];
                let sessionCpuRowArray = this.selectCpuRow.filter(item => {
                    if (item.sessionId != this.sessionId) {
                        return item;
                    } else {
                        //cpuRowList.push(item);
                        item.selectRow.forEach(item1 => {
                            if (item1.threadName != row.id) {
                                cpuRowList.push(item1)
                            }
                        })
                    }
                });

                cpuRowList.push({threadName: row.id, threadInfo: row});

                let selectRowInfo = {sessionId: this.sessionId, selectRow: cpuRowList};
                sessionCpuRowArray.push(selectRowInfo);

                let callTabs = [];

                let callTab = {
                    title: row.name,
                    name: row.id + '',
                    router: '/call/' + row.id,
                    closable: true
                };
                let callTabsArray = this.sessionCallTabs.filter(item => {
                    if (item.sessionId != this.sessionId) {
                        return item;
                    } else {
                        item.callTabs.forEach((item2) => {
                            if (item2.name != row.id) {
                                callTabs.push(item2);
                            }
                        })
                    }
                });

                // 限制同一会话下最多只能打开5个标签页
                if (callTabs.length >= 5) {
                    this.$notify({type:'warning', title:'提示', message: '同一会话下最多只能打开5个标签页'})
                    return false;
                }

                callTabs.push(callTab)
                let sessionCalls = {sessionId: this.sessionId, callTabs: callTabs};
                callTabsArray.push(sessionCalls)

                this.$store.commit('session_tabs_value', tabsValueArray);
                this.$store.commit('select_cpu_row', sessionCpuRowArray);
                this.$store.commit('session_call_tabs', callTabsArray);
            },
            select_thread(thread_id) {
                this.selected_thread_id = thread_id;
            },
            getThreads(){
                if (this.sessionId && this.sessionThreads.length > 0) {
                    let threadsInfo = this.sessionThreads.filter(item => {
                        if (item.sessionId == this.sessionId) {
                            return item;
                        }
                    });
                    this.threads = [];
                    if (threadsInfo.length > 0) {
                        this.threads = threadsInfo[0].threads;
                    }
                }
                if (this.historySamples.length <= 0) {
                    this.$router.push({
                        path:'/samples'
                    });
                }
            },
            on_cpu_time_result(){
                let cpuTimeArray = this.sessionCpuTimes.filter(item => {
                    if (item.sessionId == this.sessionId) {
                        return item;
                    }
                })
                if (cpuTimeArray.length <= 0) {
                    return false;
                }
                console.log('当前cpu time：', cpuTimeArray);
                let data = cpuTimeArray[0].cpuTimeData;

                if (!data) {
                    return false;
                }

                let sessionSampleInfoArray = this.sessionSampleInfo.filter(item => {
                    if (item.sessionId == this.sessionId) {
                        return item;
                    }
                });

                let sessionSample = sessionSampleInfoArray[0].sessionSample

                let sess_start_time = sessionSample.record_start_time;
                let sess_end_time = sessionSample.last_record_time;
                let unit_time_ms = sessionSample.unit_time_ms;

                console.log('sess_start_time:',sess_start_time,'sess_end_time:',sess_end_time,'unit_time_ms:',unit_time_ms)

                for (let i = 0; i < data.length; i++) {
                    let thread = data[i];
                    if (thread.total_cpu_time > 0) {
                        let ts_data = this.fill_ts_data(thread.ts_data, thread.start_time, thread.end_time, sess_start_time, sess_end_time, unit_time_ms);
                        let myChart = this.create_echarts_bar("thread_cpu_chart_"+thread.id, ts_data);
                    }
                }
            },
            update_call_stack_tree(thread_id, start_time, end_time) {
                var request = {
                    "cmd": "flame_graph",
                    "options": {
                        "session_id": this.sessionId,
                        "thread_id": thread_id,
                        "start_time": start_time,
                        "end_time": end_time
                    }
                };
                this.$webSocket.webSocketSendMessage(JSON.stringify(request));
            },
            create_echarts_bar(elemId, echartsData) {
                if (!echartsData) {
                    echartsData = [];
                    for (let i = 0; i < 3000; i++) {
                        echartsData.push(Math.random().toFixed(2) * 1000);
                    }
                }

                let options = {
                    dataZoom: [{
                        type: 'inside',
                        start: 0,
                        end: 0,
                        moveOnMouseMove: false,
                        moveOnMouseWheel: false,
                        zoomOnMouseWheel: false,
                        disabled: true
                    }, {
                        type: 'slider',
                        //backgroundColor:'#cccccc',
                        dataBackground: {
                            lineStyle: {
                                color: '#409eff',
                                opacity: 1
                            },
                            areaStyle: {
                                color: '#3a8ee6',
                                opacity: 0.3
                            }
                        },
                        realtime: false,
                        filterMode: 'empty',
                        top: 'top',
                        left: 'left',
                        fillerColor: '',
                        handleStyle: {
                            opacity: 0,
                            shadowBlur: 0
                        }
                    }],
                    xAxis: {
                        data: echartsData,
                        show: false
                    },
                    yAxis: {show: false},
                    series: [{
                        type: 'bar',//bar
                        data: echartsData,
                        large: true,
                        largeThreshold: 50,
                        itemStyle: {
                            color: '#e74911', // bar颜色
                            opacity: 0 // 透明度，0：不绘制
                        }
                    }]
                }
                let myChart = this.$echarts.init(document.getElementById(elemId));
                myChart.setOption(options);

                return myChart;
            },
            fill_ts_data(thread_ts_data, thread_start_time, thread_end_time, start_time, end_time, unit_time_ms) {
                let fill_steps_before = (thread_start_time - start_time) / unit_time_ms;
                let fill_steps_after = (end_time - thread_end_time) / unit_time_ms;
                if (fill_steps_before < 1 && fill_steps_after < 1) {
                    return thread_ts_data;
                }

                let new_data_vec = [];// Vec::with_capacity(data_vec.len()+(fill_steps_before+fill_steps_after) as usize);
                for (var i = 0; i < fill_steps_before; i++) {
                    new_data_vec.push(0);
                }

                new_data_vec = new_data_vec.concat(thread_ts_data);

                for (var i = 0; i < fill_steps_after; i++) {
                    new_data_vec.push(0);
                }
                return new_data_vec;
            },
        },
        watch: {
            '$route': (to, from) => {
                /*console.log('sessionThreads===', this.sessionThreads)
                if (this.sessionThreads.length <= 0) {
                    this.$router.push('/samples')
                }*/
            },
            sessionId() {
                this.getThreads();
                this.$nextTick(()=>{
                    this.on_cpu_time_result();
                })
            }
        }
    }
</script>

<!-- Add "scoped" attribute to limit CSS to this component only -->
<style scoped>
    .selected {
        background-color: #c2e7b0;
    }
    .echarts_bar{
        height: 30px;
        width: 100%;
        float: left;
    }
    .thread_name {
        height: 30px;
        line-height: 30px;
        width: 30%;
        margin-right: 5px;
        float: left;
        overflow: hidden;
        text-overflow:ellipsis;
        cursor: default;
        word-break: break-all;
    }
    .thread_bar {
        height: 30px;
        width: 100%;
        float: left;
        color: #e74911;
        overflow: hidden;
    }
    #cpu_time_content {
        width: 100%;
        height: 100%;
        overflow-x: hidden;
        overflow-y: auto;
    }
</style>
