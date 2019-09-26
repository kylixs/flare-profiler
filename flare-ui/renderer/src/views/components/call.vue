<template>
    <div class="session">
        <div id="flame_graph" v-show="show_flame_graph">
            <el-table :data="selectCpuRowArray">
                <el-table-column width="400">
                    <template slot-scope="scope">
                        <span>{{scope.row.name}}</span>
                    </template>
                </el-table-column>
                <el-table-column>
                    <template slot-scope="scope">
                        <div class="thread_bar" v-bind:id="'thread_select_cpu_chart_' + scope.row.id+''"></div>
                    </template>
                </el-table-column>
            </el-table>
            <!--<h4 class="title">Flame Graph</h4>-->
            <div id="flame_graph_svg" v-html="flame_graph_data"></div>
        </div>
    </div>
</template>

<script>
    export default {
        name: 'call',
        data() {
            return {
                show_flame_graph: true,
                flame_graph_data: "",
                selectCpuRowArray:[],
            }
        },
        computed: {
            sampleInfo() {
                return this.$store.state.sampleInfo;
            },
            exampleInfo() {
                return this.$store.state.exampleInfo;
            },
            sessionId() {
                return this.$route.params.sessionInfo;
            },
            sessionCpuTimes() {
                return this.$store.state.sessionCpuTimes;
            },
            sessionFlameGraph() {
                return this.$store.state.sessionFlameGraph;
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
        created() {
            this.getFlameGraphData();
        },
        methods: {
            getFlameGraphData(){
                if (!this.sessionFlameGraph && this.sessionFlameGraph.length > 0) {
                    let flareGrapList = this.sessionFlameGraph.filter(item => {
                        if (item.session_id == this.sessionId) {
                            return item;
                        }
                    })
                    if (flareGrapList.length > 0) {
                        this.flame_graph_data = flareGrapList[0].flame_graph_data;
                    }
                }
                if (this.historySamples.length <= 0) {
                    this.$router.push({
                        path:'/samples'
                    });
                }

                let cpuRowList = this.selectCpuRow.filter(item => {
                    if (item.sessionId == this.sessionId) {
                        return item;
                    }
                })
                cpuRowList.forEach(item => {
                    this.selectCpuRowArray.push(item.selectRow)
                })
            },
            on_cpu_time_result(){

                let cpuTimeArray = this.sessionCpuTimes.filter(item => {
                    if (item.sessionId == this.sessionId) {
                        return item;
                    }
                })
                if (cpuTimeArray.length <= 0 || this.selectCpuRowArray.length <= 0) {
                    return false;
                }
                let data = cpuTimeArray[0].cpuTimeData;
                if (!data) {
                    return false;
                }
                var sess_start_time = this.sampleInfo.record_start_time;
                var sess_end_time = this.sampleInfo.last_record_time;
                var unit_time_ms = this.sampleInfo.unit_time_ms;

                let cpuInfo = data.filter(item => {
                    if (item.id == this.selectCpuRowArray[0].id) {
                        return item;
                    }
                })

                let thread = cpuInfo[0];
                if (thread.total_cpu_time > 0) {
                    let ts_data = this.fill_ts_data(thread.ts_data, thread.start_time, thread.end_time, sess_start_time, sess_end_time, unit_time_ms);

                    let myChart = this.create_echarts_bar("thread_select_cpu_chart_"+thread.id, ts_data);
                    myChart.on('datazoom', (evt) => {
                        var axis = myChart.getModel().option.xAxis[0];
                        // var starttime = axis.data[axis.rangeStart];
                        // var endtime = axis.data[axis.rangeEnd];
                        let start_time = sess_start_time + axis.rangeStart*unit_time_ms;
                        let end_time = sess_start_time + axis.rangeEnd*unit_time_ms;
                        console.log("datazoom: thread:",thread.id, ", index:", axis.rangeStart,"-", axis.rangeEnd,", time:", start_time,"-", end_time );
                        this.update_call_stack_tree(thread.id, start_time, end_time);
                    })
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
                        end: 10,
                        moveOnMouseMove: false,
                        moveOnMouseWheel: false,
                        zoomOnMouseWheel: false
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
                        left: 'left'
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
                this.getFlameGraphData();
            },
            sessionFlameGraph() {
                //this.getFlameGraphData();
                if (this.sessionFlameGraph.length > 0) {
                    let flareGrapList = this.sessionFlameGraph.filter(item => {
                        if (item.session_id == this.sessionId) {
                            return item;
                        }
                    })
                    this.flame_graph_data = flareGrapList[0].flame_graph_data;
                }
            },
        }
    }
</script>

<!-- Add "scoped" attribute to limit CSS to this component only -->
<style scoped>
    .thread_bar {
        height: 30px;
        width: 100%;
        float: left;
        color: #e74911;
        overflow: hidden;
    }
</style>
