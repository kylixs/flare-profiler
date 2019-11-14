<template>
    <div class="session height100">
        <div id="flame_graph height100" v-show="show_flame_graph" class="height100">
            <!--<el-select v-model="selectValue" style="width: 100%">
                <el-option
                        v-for="item in threads"
                        :key="item.name"
                        :label="item.name"
                        :value="item.name" style="width: 100%">
                    <div style="width: 100%">
                        <div style="float: left; width: 400px">
                            {{ item.name }}
                        </div>
                        <div class="thread_bar" v-bind:id="'thread_select_cpu_chart_' + item.id+''"></div>
                    </div>
                </el-option>
            </el-select>-->

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
            <!--<div id="flame_graph_svg" style="margin-top: 20px;" v-html="flame_graph_data"></div>-->
            <div id="chrome_flame_chart_container" class="flame-container height100"  v-show="show_chrome_flame_chart">
                <!--<iframe id="chrome_flame_chart" src="/static/devtools/flamechart.html"></iframe>-->
                <iframe v-bind:id="'chrome_flame_chart_'+callName+''"
                        frameborder="no"
                        scrolling="no"
                        src="plugins/devtools/flamechart.html" height="100%" width="100%"></iframe>
            </div>
        </div>
    </div>
</template>

<script>
    export default {
        name: 'call',
        data() {
            return {
                show_chrome_flame_chart:true,
                show_flame_graph: true,
                flame_graph_data: "",
                selectCpuRowArray:[],
                dataZoomStart: 0,
                dataZoomEnd: 10,
                curChartInfo:{},
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
            sessionId() {
                return this.$route.params.sessionInfo;
            },
            callName() {
                return this.$route.params.call;
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
            historySamples() {
                return this.$store.state.historySamples;
            },
            sessionTabsValue() {
                return this.$store.state.sessionTabsValue;
            },
            selectCpuRow() {
                return this.$store.state.selectCpuRow;
            },
            echartsDataZoomPosition() {
                return this.$store.state.echartsDataZoomPosition;
            },
        },
        mounted(){
            this.$nextTick(()=>{
                this.on_cpu_time_result();
                setTimeout(()=>{
                    this.initPosition();
                }, 1000)
                // this.mousewheelGraph();
                // this.getFlameGraphData();
            })
        },
        /*activated(){
            this.getFlameGraphData();
            this.$nextTick(()=>{
                this.on_cpu_time_result();
            })
        },*/
        created() {
            this.getFlameGraphData();
        },
        methods: {
            get_chrome_flame_chart() {
                let contentWindow = document.getElementById("chrome_flame_chart_" + this.callName).contentWindow;
                console.log('contentWindow', contentWindow);
                return contentWindow;
            },
            getFlameGraphData(){
                let flareGrapList = this.sessionFlameGraph.filter(item => {
                    if (item.sessionId == this.sessionId) {
                        item.flameGraphList.filter(item1 => {
                            if (item1.threadId == this.callName && item1.flameGraphData) {
                                this.$nextTick(()=>{
                                    let contentWindow = this.get_chrome_flame_chart();
                                    try {
                                        contentWindow.set_flame_chart_data(item1.flameGraphData);
                                    } catch (e) {
                                        console.log('set_flame_chart_data 函数不存在', e);
                                    }
                                })
                            }
                        })
                    }
                })

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

                this.selectCpuRowArray = [];
                cpuRowList.forEach(item => {
                    item.selectRow.forEach(item1 => {
                        if (item1.threadName == this.callName) {
                            this.selectCpuRowArray.push(item1.threadInfo)
                        }
                    })
                })
            },
            mousewheelGraph(){
                console.log('绑定缩放init')//
                // var flame_graph_svg = document.getElementById("flame_graph_svg").addEventListener('mousewheel', this.mouseWheelFu, false);
                document.getElementById('flame_graph_svg').onmousewheel = this.mouseWheelFu;
                /*var flame_graph_svg = document.getElementsByTagName('svg');
                console.log('flame_graph_svg', flame_graph_svg);

                for (let i = 0; i < flame_graph_svg.length; i++) {
                    flame_graph_svg[i].onmousewheel = this.mouseWheelFu;
                }*/

                console.log('绑定缩放end')
            },
            mouseWheelFu(event) {
                this.initPosition();
                console.log('触发缩放了哟：', event)
                /*const zoomFactor = 1.1;
                const mouseWheelZoomSpeed = 1 / 120;
                const reference = event.offsetX / event.currentTarget.clientWidth;
                let pow = Math.pow(zoomFactor, -event.wheelDeltaY * mouseWheelZoomSpeed);
                console.log('reference:', reference, 'pow:', pow)*/
                if (typeof event.wheelDeltaY === 'number' && event.wheelDeltaY) {
                    const zoomFactor = 1.1;
                    const mouseWheelZoomSpeed = 1 / 120;
                    const reference = event.offsetX / event.currentTarget.clientWidth;
                    this._zoom(Math.pow(zoomFactor, -event.wheelDeltaY * mouseWheelZoomSpeed), reference);
                }
                event.preventDefault();
            },
            constrain(num, min, max) {
                if (num < min)
                    num = min;
                else if (num > max)
                    num = max;
                return num;
            },
            _zoom(factor, reference) {

                let sessionSampleInfoArray = this.sessionSampleInfo.filter(item => {
                    if (item.sessionId == this.sessionId) {
                        return item;
                    }
                });

                let sessionSample = sessionSampleInfoArray[0].sessionSample

                var sess_start_time = sessionSample.record_start_time;
                var sess_end_time = sessionSample.last_record_time;
                var unit_time_ms = sessionSample.unit_time_ms;

                var total_time = sess_end_time - sess_start_time;

                let left = (this.curChartInfo.start_time - sess_start_time)/total_time;
                let right = (this.curChartInfo.end_time - sess_start_time)/total_time;

                const windowSize = right - left;
                let newWindowSize = factor * windowSize;
                if (newWindowSize > 1) {
                    newWindowSize = 1;
                    factor = newWindowSize / windowSize;
                }

                var ratio = reference;
                reference = left + windowSize * reference;
                left = reference + (left - reference) * factor;
                left = this.constrain(left, 0, 1 - newWindowSize);
                right = reference + (right - reference) * factor;
                right = this.constrain(right, newWindowSize, 1);

                var startTime = sess_start_time + Math.ceil(total_time * left);
                var endTime = sess_start_time + Math.ceil(total_time * right);

                console.log('left:', left, 'right:', right)
                let myChart = this.curChartInfo.myChart;

                let options = {
                    dataZoom: [{
                        type: 'inside',
                        start: left * 100,
                        end: right * 100,
                        moveOnMouseMove: false,
                        moveOnMouseWheel: false,
                        zoomOnMouseWheel: false
                    }]
                }
                myChart.setOption(options);

                let curDataZoomPosition = {
                    threadName: this.callName,
                    dataZoomStart: left * 100,
                    dataZoomEnd: right * 100,
                    threadId: this.curChartInfo.threadId,
                    start_time:startTime,
                    end_time: endTime,
                    myChart: myChart
                };

                this.saveEchartsDataZoomPosition(curDataZoomPosition);

                this.update_call_stack_tree(this.curChartInfo.threadId, startTime, endTime);
                //this.update_d3_flame_graph(this.curChartInfo.threadId, startTime, endTime);
            },
            initPosition() {
                let dataZoomPositionArray = this.echartsDataZoomPosition.filter(item => {
                    if (item.sessionId == this.sessionId) {
                        return item;
                    }
                });

                if (dataZoomPositionArray.length > 0) {
                    let dataZoomPosition = {};
                    dataZoomPositionArray.filter(item => {
                        item.dataZoomPosition.forEach(item1 => {
                            if (item1.threadName == this.callName) {
                                dataZoomPosition = {...item1};
                            }
                        })
                    });

                    this.curChartInfo = {...dataZoomPosition}
                    this.dataZoomStart = dataZoomPosition.dataZoomStart | 0;
                    this.dataZoomEnd = dataZoomPosition.dataZoomEnd | 10;
                    /*threadId: thread.id,
                        start_time:start_time,
                        end_time: end_time,*/
                    this.update_d3_flame_graph(this.curChartInfo.threadId, this.curChartInfo.start_time, this.curChartInfo.end_time);
                }
            },
            saveEchartsDataZoomPosition(curDataZoomPosition){
                let postitionArray = [];
                let dataZoomPositionArray = this.echartsDataZoomPosition.filter(item => {
                    if (item.sessionId == this.sessionId) {
                        return item;
                    } else {
                        postitionArray.push(item);
                    }
                });

                let dataZoomPosition = [];
                dataZoomPositionArray.filter(item => {
                    item.dataZoomPosition.forEach(item1 => {
                        if (item1.threadName != this.callName) {
                            dataZoomPosition.push(item1);
                            //return item;
                        }
                    })
                });
                dataZoomPosition.push(curDataZoomPosition);
                postitionArray.push({sessionId: this.sessionId, dataZoomPosition:dataZoomPosition})
                //console.log('postitionArray', postitionArray, 'thread', thread);
                this.$store.commit('echarts_dataZoom_position', postitionArray);
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

                let sessionSampleInfoArray = this.sessionSampleInfo.filter(item => {
                    if (item.sessionId == this.sessionId) {
                        return item;
                    }
                });

                let sessionSample = sessionSampleInfoArray[0].sessionSample

                var sess_start_time = sessionSample.record_start_time;
                var sess_end_time = sessionSample.last_record_time;
                var unit_time_ms = sessionSample.unit_time_ms;

                let cpuInfo = data.filter(item => {
                    if (item.id == this.selectCpuRowArray[0].id) {
                        return item;
                    }
                })

                let thread = cpuInfo[0];
                if (thread.total_cpu_time > 0) {
                    let ts_data = this.fill_ts_data(thread.ts_data, thread.start_time, thread.end_time, sess_start_time, sess_end_time, unit_time_ms);

                    let myChart = this.create_echarts_bar("thread_select_cpu_chart_"+thread.id, ts_data, thread);
                    myChart.on('datazoom', (evt) => {
                        console.log('evt start', evt.start, 'evt end', evt.end)

                        var axis = myChart.getModel().option.xAxis[0];
                        // var starttime = axis.data[axis.rangeStart];
                        // var endtime = axis.data[axis.rangeEnd];
                        let start_time = sess_start_time + axis.rangeStart*unit_time_ms;
                        let end_time = sess_start_time + axis.rangeEnd*unit_time_ms;

                        let curDataZoomPosition = {
                            threadName: this.callName,
                            dataZoomStart: evt.start,
                            dataZoomEnd: evt.end,
                            threadId: thread.id,
                            start_time:start_time,
                            end_time: end_time,
                            myChart: myChart
                        };

                        this.saveEchartsDataZoomPosition(curDataZoomPosition);

                        console.log("datazoom: thread:",thread.id, ", index:", axis.rangeStart,"-", axis.rangeEnd,", time:", start_time,"-", end_time );
                        // this.update_call_stack_tree(thread.id, start_time, end_time);
                        this.update_d3_flame_graph(thread.id, start_time, end_time);
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
            update_d3_flame_graph(thread_id, start_time, end_time) {
                var request = {
                    "cmd": "d3_flame_graph",
                    "options": {
                        "session_id": this.sessionId,
                        "thread_id": thread_id,
                        "start_time": start_time,
                        "end_time": end_time,
                        "stats_type": 'duration'
                    }
                };
                this.$webSocket.webSocketSendMessage(JSON.stringify(request));
            },
            create_echarts_bar(elemId, echartsData, thread) {
                if (!echartsData) {
                    echartsData = [];
                    for (let i = 0; i < 3000; i++) {
                        echartsData.push(Math.random().toFixed(2) * 1000);
                    }
                }

                if (!thread.hasOwnProperty('id') || thread.id == undefined || !('id' in thread)) {
                    console.log("id为空，", thread)
                    return false;
                }

                this.initPosition();

                let options = {
                    dataZoom: [{
                        type: 'inside',
                        start: this.dataZoomStart,
                        end: this.dataZoomEnd,
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
                        showSymbol: true,
                        hoverAnimation: false,
                        animation: false,
                        itemStyle: {
                            color: '#e74911', // bar颜色
                            opacity: 0 // 透明度，0：不绘制
                        }
                    }]
                }
                let myChart = this.$echarts.init(document.getElementById(elemId));
                myChart.setOption(options);
                console.log('myChart', myChart)
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
            sessionFlameGraph() {
                if (this.sessionFlameGraph.length > 0) {
                    this.sessionFlameGraph.filter(item => {
                        if (item.sessionId == this.sessionId) {
                            item.flameGraphList.filter(item1 => {
                                if (item1.threadId == this.callName && item1.flameGraphData) {
                                    console.log('flame_graph_data',item1.flameGraphData)
                                    setTimeout(()=>{
                                        this.$nextTick(()=>{
                                            let contentWindow = this.get_chrome_flame_chart();
                                            try {
                                                contentWindow.set_flame_chart_data(item1.flameGraphData)
                                            } catch (e) {
                                                console.log('set_flame_chart_data 函数不存在', e)
                                            }
                                        })
                                    }, 1000)
                                }
                            })
                        }
                    })
                }
            },
            callName() {
                this.getFlameGraphData();
            },
            sessionId(){
                this.nextTick(()=>{
                    this.on_cpu_time_result();
                    setTimeout(()=>{
                        this.initPosition();
                    },1000)
                })
            }
        }
    }
</script>

<!-- Add "scoped" attribute to limit CSS to this component only -->
<style scoped>
    .thread_bar {
        height: 30px;
        width: 900px;
        float: left;
        color: #e74911;
        overflow: hidden;
    }
</style>
