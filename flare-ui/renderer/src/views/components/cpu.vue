<template>
    <div class="cpu_time_content" style="width: 100%"><!--highlight-current-row="true" show-header="false"-->
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
        <!--<div id="cpu_time_content">
            <div v-for="thread,index in threads" @click="select_thread(thread.id)" class="echarts_bar" :class="{selected: selected_thread_id == thread.id}">
                <div class="thread_name" :title="thread.name" >{{thread.name}}</div>
                <div class="thread_bar" v-bind:id="'thread_cpu_chart_' + thread.id+''"></div>
            </div>
        </div>-->
    </div>
</template>

<script>
    export default {
        name: 'cpu',
        data() {
            return {
                threads: [],
                selected_thread_id: null,
            }
        },
        computed: {
            sampleInfo() {
                return this.$store.state.sampleInfo;
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
                console.log('row', row);
                console.log('column', column)
                console.log('event', event)

                let tabsValueArray = this.sessionTabsValue.filter(item => {
                    if (item.sessionId != this.sessionId) {
                        return item
                    }
                });
                let tabsInfo = {sessionId: this.sessionId, tabsValue: 'call'}
                tabsValueArray.push(tabsInfo);
                this.$store.commit('session_tabs_value', tabsValueArray);

                let cpuRowArray = this.selectCpuRow.filter(item => {
                    if (item.sessionId != this.sessionId) {
                        return item;
                    }
                });
                let selectRowInfo = {sessionId: this.sessionId, selectRow: row};
                cpuRowArray.push(selectRowInfo);
                this.$store.commit('select_cpu_row', cpuRowArray);

                this.$router.push({path: '/' + this.sessionId + '/call'})
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
                let data = cpuTimeArray[0].cpuTimeData;

                if (!data) {
                    return false;
                }
                var sess_start_time = this.sampleInfo.record_start_time;
                var sess_end_time = this.sampleInfo.last_record_time;
                var unit_time_ms = this.sampleInfo.unit_time_ms;

                for (let i = 0; i < data.length; i++) {
                    let thread = data[i];
                    if (thread.total_cpu_time > 0) {
                        let ts_data = this.fill_ts_data(thread.ts_data, thread.start_time, thread.end_time, sess_start_time, sess_end_time, unit_time_ms);

                        let myChart = this.create_echarts_bar("thread_cpu_chart_"+thread.id, ts_data);
                        /*myChart.on('datazoom', (evt) => {
                            var axis = myChart.getModel().option.xAxis[0];
                            // var starttime = axis.data[axis.rangeStart];
                            // var endtime = axis.data[axis.rangeEnd];
                            let start_time = sess_start_time + axis.rangeStart*unit_time_ms;
                            let end_time = sess_start_time + axis.rangeEnd*unit_time_ms;
                            console.log("datazoom: thread:",thread.id, ", index:", axis.rangeStart,"-", axis.rangeEnd,", time:", start_time,"-", end_time );
                            this.update_call_stack_tree(thread.id, start_time, end_time);
                        })*/
                    }
                    //profiler.data.thread_cpu_time_map[thread.id] = thread;
                }

                //debugger
                let cpuRowList = this.selectCpuRow.filter(item => {
                    if (item.sessionId == this.sessionId) {
                        return item;
                    }
                })
                if (cpuRowList.length > 0) {
                    this.$refs.cpuTable.setCurrentRow(cpuRowList[0].selectRow);
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
                this.on_cpu_time_result();
                this.getThreads();
                if (!this.sessionThreads) {
                    this.$router.push('/samples')
                }
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
