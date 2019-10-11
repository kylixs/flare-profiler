

function fill_ts_data(thread_ts_data, thread_start_time, thread_end_time, start_time, end_time, unit_time_ms) {
    let fill_steps_before = (thread_start_time - start_time)/unit_time_ms;
    let fill_steps_after = (end_time - thread_end_time)/unit_time_ms;
    if (fill_steps_before < 1 && fill_steps_after < 1) {
        return thread_ts_data;
    }

    let new_data_vec = [];// Vec::with_capacity(data_vec.len()+(fill_steps_before+fill_steps_after) as usize);
    for (var i=0; i<fill_steps_before; i++) {
        new_data_vec.push(0);
    }

    new_data_vec = new_data_vec.concat(thread_ts_data);

    for (var i=0; i<fill_steps_after; i++) {
        new_data_vec.push(0);
    }
    return new_data_vec;
}

//"hello {0}, {1}".format("Tom", "Great")
String.prototype.format = function()
{
    var args = arguments;
    return this.replace(/\{(\d+)\}/g,
        function(m,i){
            return args[i];
        });
}


/**
 * @param {number} num
 * @param {number} min
 * @param {number} max
 * @return {number}
 */
Number.constrain = function(num, min, max) {
    if (num < min)
        num = min;
    else if (num > max)
        num = max;
    return num;
};


var profiler = {
    connected: false,
    agent_addr: "localhost:3333",
    profiler_addr: "localhost:3344",
    sample_dir: null,
    dashboard_timer: null,
    show_history_samples: false,
    show_message: false,
    show_sessions: false,

    show_call_tree: false,
    show_flame_graph: false,
    show_d3_flame_graph: false,
    show_chrome_flame_chart: true,

    selected_thread_id: null,
    stats_type: "duration",
    flame_graph_state: {
        start_time: 0,
        end_time: 0,
        thread_id: null,
        chart: null,
    },
    data: {
        requests: {},
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
    start_auto_refresh() {
        if (this.dashboard_timer == null) {
            profiler.do_refresh();
            if(profiler.data.type == "attach") {
                this.dashboard_timer = setInterval(function () {
                    profiler.do_refresh();
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
    do_refresh(){
        if (profiler.data.session_id == ""){
            return;
        }
        this.update_dashboard();
        setTimeout(function () {
            profiler.update_cpu_time();
        }, 500);
    },
    update_dashboard(){
        console.log("send request: get_dashboard");
        socket.send(JSON.stringify({
            "cmd": "dashboard",
            "options": {
                "session_id": profiler.data.session_id
            }
        }));
    },
    update_cpu_time(){
        var thread_ids = [];
        for ( var i=0;i<profiler.data.threads.length;i++) {
            thread_ids.push(profiler.data.threads[i].id);
        }
        var graph_width = 900;
        var sample_interval = profiler.data.sample_info.sample_interval;
        var start_time = profiler.data.sample_info.record_start_time;
        var end_time = profiler.data.sample_info.last_record_time;
        var ratio = Math.ceil((end_time - start_time) / graph_width / sample_interval);
        if (ratio > 10 ){
            ratio = Math.floor(ratio/10)*10;
        }
        var unit_time_ms = ratio * sample_interval;
        profiler.data.sample_info.unit_time_ms = unit_time_ms;

        var request = {
            cmd: "cpu_time",
            options: {
                "session_id": profiler.data.session_id,
                "thread_ids": thread_ids,
                "start_time": start_time,
                "end_time": end_time,
                "unit_time_ms": unit_time_ms
            }
        };
        socket.send(JSON.stringify(request));
        // console.log("update_cpu_time: ", request);
    },
    list_sessions() {
        this.show_sessions = true;
        this.show_history_samples = false;
        socket.send(JSON.stringify({
            "cmd": "list_sessions"
        }))
    },
    list_history: function () {
        this.show_sessions = false;
        this.show_history_samples = true;
        socket.send(JSON.stringify({
            "cmd": "history_samples"
        }))
    },
    connect_agent: function () {
        this.clear_session();
        var request = {
            "cmd": "connect_agent",
            "options": {
                "agent_addr": this.agent_addr
            }
        };
        socket.send(JSON.stringify(request));
        this.connected = true;
    },
    close_session() {
        this.connected = false;
        var request = {
            "cmd": "close_session",
            "options": {
                "session_id": profiler.data.session_id
            }
        };
        socket.send(JSON.stringify(request));
    },
    open_sample: function (sample_data_dir) {
        this.clear_session();
        var request = {
            "cmd": "open_sample",
            "options": {
                "sample_data_dir": sample_data_dir
            }
        };
        socket.send(JSON.stringify(request));
    },
    active_session:function (session_id, type) {
        this.clear_session();
        profiler.data.session_id = session_id;
        profiler.data.type = type;
        profiler.start_auto_refresh();
    },
    clear_session: function () {
        this.stop_auto_refresh();
        this.data.session_id = "";
        this.data.threads = [];
        this.data.sample_info = {};
        this.data.thread_cpu_time_map = {};
    },
    on_cpu_time_result(data){
        var sess_start_time = profiler.data.sample_info.record_start_time;
        var sess_end_time = profiler.data.sample_info.last_record_time;
        var unit_time_ms = profiler.data.sample_info.unit_time_ms;

        for (let i = 0; i < data.thread_cpu_times.length; i++) {
            let thread = data.thread_cpu_times[i];
            let ts_data = fill_ts_data(thread.ts_data, thread.start_time, thread.end_time, sess_start_time, sess_end_time, unit_time_ms);

            let myChart = create_echarts_bar("thread_cpu_chart_"+thread.id, ts_data);
            myChart.on('datazoom', function (evt) {
                var axis = myChart.getModel().option.xAxis[0];
                // var starttime = axis.data[axis.rangeStart];
                // var endtime = axis.data[axis.rangeEnd];
                let start_time = sess_start_time + axis.rangeStart*unit_time_ms;
                let end_time = sess_start_time + axis.rangeEnd*unit_time_ms;
                console.log("datazoom: thread:",thread.id, ", index:", axis.rangeStart,"-", axis.rangeEnd,", time:", start_time,"-", end_time );
                profiler.update_stack_stats(thread.id, start_time, end_time, myChart);
            });
            profiler.data.thread_cpu_time_map[thread.id] = thread;
        }
    },
    update_stack_stats(thread_id, start_time, end_time, myChart){
        if (!thread_id) {
            thread_id = this.flame_graph_state.thread_id;
            start_time = this.flame_graph_state.start_time;
            end_time = this.flame_graph_state.end_time;
        }
        if (!myChart) {
            myChart = profiler.flame_graph_state.chart;
        }
        if (!thread_id || !start_time || !end_time){
            return;
        }
        profiler.flame_graph_state = profiler.flame_graph_state || {};
        profiler.flame_graph_state.thread_id = thread_id;
        profiler.flame_graph_state.start_time = start_time;
        profiler.flame_graph_state.end_time = end_time;
        profiler.flame_graph_state.chart = myChart;

        if(profiler.show_call_tree){
            profiler.update_call_stack_tree(thread_id, start_time, end_time);
        } else if(profiler.show_flame_graph){
            profiler.update_flame_graph(thread_id, start_time, end_time);
        } else if(profiler.show_d3_flame_graph || profiler.show_chrome_flame_chart){
            profiler.update_d3_flame_graph(thread_id, start_time, end_time);
        }
    },
    update_call_stack_tree(thread_id, start_time, end_time) {
        var request = {
            "cmd": "call_tree",
            "options": {
                "session_id": profiler.data.session_id,
                "thread_ids": [thread_id],
                "start_time": start_time,
                "end_time": end_time
            }
        };
        socket.send(JSON.stringify(request));
    },
    update_flame_graph(thread_id, start_time, end_time) {
        var request_time = new Date().getTime();
        var request = {
            "cmd": "flame_graph",
            "options": {
                "session_id": profiler.data.session_id,
                "request_time": request_time,
                "thread_id": thread_id,
                "start_time": start_time,
                "end_time": end_time,
                "stats_type": profiler.stats_type,
                "image_width": 900
            }
        };
        profiler.data.requests['flame_graph'] = request;

        socket.send(JSON.stringify(request));
    },
    update_d3_flame_graph(thread_id, start_time, end_time) {
        var request = {
            "cmd": "d3_flame_graph",
            "options": {
                "session_id": profiler.data.session_id,
                "thread_id": thread_id,
                "start_time": start_time,
                "end_time": end_time,
                "stats_type": profiler.stats_type
            }
        };
        socket.send(JSON.stringify(request));
    },
    select_thread(thread_id) {
        this.selected_thread_id = thread_id;
    },
    onFlameMouseWheel(event) {
        // if (event.target.tagName != "DIV" ){
        // 	return;
        // }
        if (!profiler.flame_graph_state.thread_id)
            return;
        if (typeof event.wheelDeltaY === 'number' && event.wheelDeltaY) {
            const zoomFactor = 1.1;
            const mouseWheelZoomSpeed = 1 / 120;
            const reference = event.offsetX / event.currentTarget.clientWidth;
            profiler.zoom_flame_graph(Math.pow(zoomFactor, -event.wheelDeltaY * mouseWheelZoomSpeed), reference);
        }
        event.preventDefault();

    },
    zoom_flame_graph(factor, reference) {
        var state = profiler.flame_graph_state;
        var sess_start_time = profiler.data.sample_info.record_start_time;
        var sess_end_time = profiler.data.sample_info.last_record_time;
        var total_time = sess_end_time - sess_start_time;

        let left = (state.start_time - sess_start_time)/total_time;
        let right = (state.end_time - sess_start_time)/total_time;
        const windowSize = right - left;
        let newWindowSize = factor * windowSize;
        if (newWindowSize > 1) {
            newWindowSize = 1;
            factor = newWindowSize / windowSize;
        }
        //fix reference
        var ratio = reference;
        reference = left + windowSize * reference;
        left = reference + (left - reference) * factor;
        left = Number.constrain(left, 0, 1 - newWindowSize);
        right = reference + (right - reference) * factor;
        right = Number.constrain(right, newWindowSize, 1);

        //update time range
        var new_start_time = sess_start_time + Math.ceil(total_time * left);
        var new_end_time = sess_start_time + Math.ceil(total_time * right);
        console.log("zoom_flame_graph, factor:{0}, reference:{1}, range: [{2},{3}] -> [{4},{5}]".format(
            factor, reference, state.start_time, state.end_time, new_start_time, new_end_time));
        state.start_time = new_start_time;
        state.end_time = new_end_time;

        //refresh stack graph
        var myChart = state.chart;
        myChart.setOption({
            dataZoom: [{
                type: 'inside',
                start: left*100,
                end: right*100,
                moveOnMouseMove: false,
                moveOnMouseWheel: false,
                zoomOnMouseWheel: false
            }]
        });
        this.update_stack_stats(state.thread_id, new_start_time, new_end_time);
    },
    set_zoom_time_range(start_time, end_time){
        var state = profiler.flame_graph_state;
        state.start_time = start_time;
        state.end_time = end_time;

        var sess_start_time = profiler.data.sample_info.record_start_time;
        var sess_end_time = profiler.data.sample_info.last_record_time;
        var total_time = sess_end_time - sess_start_time;
        let left = (state.start_time - sess_start_time)/total_time;
        let right = (state.end_time - sess_start_time)/total_time;

        var myChart = state.chart;
        myChart.setOption({
            dataZoom: [{
                type: 'inside',
                start: left*100,
                end: right*100,
                moveOnMouseMove: false,
                moveOnMouseWheel: false,
                zoomOnMouseWheel: false
            }]
        });
    },
    ajdust_flame_view(delta) {
        var state = profiler.flame_graph_state;
        var windowSize = state.end_time - state.start_time;
        var delta_time = Math.round(delta*windowSize);
        var new_start_time = state.start_time + delta_time;
        var new_end_time = state.end_time + delta_time;

        var thread = profiler.data.thread_cpu_time_map[state.thread_id];
        //keep left and width
        var thread_start_time = thread.start_time;
        if(new_start_time < thread_start_time){
            new_start_time = thread_start_time;
            new_end_time = new_start_time + (state.end_time-state.start_time);
        }
        //keep right
        var thread_end_time = thread.end_time;
        if(new_end_time > thread_end_time){
            new_end_time = thread_end_time;
            new_start_time = new_end_time - (state.end_time-state.start_time);
        }
        if(state.start_time != new_start_time || state.end_time != new_end_time){
            state.start_time = new_start_time;
            state.end_time = new_end_time;
            profiler.update_stack_stats();
        }
    },
    onmessage(json){
        var success = (json.result == "success");
        profiler.show_message = !success;
        //将response属性赋值到共享对象
        Object.assign(profiler.data, json.data);
        if (!success) {
            profiler.stop_auto_refresh();
        }
        switch (json.cmd) {
            case "dashboard":
                //profiler.on_dashboard_result(json.data);
                break;
            case "open_sample":
                profiler.start_auto_refresh();
                break;
            case "connect_agent":
                profiler.start_auto_refresh();
                break;
            case "history_samples":
                profiler.show_history_samples = true;
                break;
            case "cpu_time":
                profiler.on_cpu_time_result(json.data);
                break;
            case "flame_graph":
                profiler.set_zoom_time_range(json.data.start_time, json.data.end_time);
                //profiler.data.flame_graph_svg="data:image/svg+xml;utf8,"+json.data.flame_graph_data.replace(/<\?xml.*?\>.*\<!DOCTYPE.*\<svg/, "<svg");
                break;
            case "d3_flame_graph":
                let stack = json.data.d3_flame_graph_stacks;
                if(profiler.show_d3_flame_graph){
                    process_d3_flamegraph_stack(stack)
                    set_d3_flamegraph_data(stack);
                } else if(profiler.show_chrome_flame_chart){
                    get_chrome_flame_chart().set_flame_chart_data(stack);
                }
                break;
            default:
                console.log("unknown message: ", json);
                break;
        }
    }
}

document.onreadystatechange=function () {
    document.getElementById("flame_graph_svg").onmousewheel = profiler.onFlameMouseWheel;
}

function process_d3_flamegraph_stack(stack) {
    stack.name = stack.label;
    stack.value = stack.duration;
    if (stack.children) {
        for(var i=0;i<stack.children.length;i++){
            process_d3_flamegraph_stack(stack.children[i]);
        }
    }
}

var socket = new WebSocket("ws://"+profiler.profiler_addr, "flare-profiler");
socket.onopen = function(evt) {
    console.log("Connected to flare profiler successfully.");
    profiler.connected = true;

    //profiler.start_auto_refresh();
}

socket.onclose = function(evt) {
    console.log("Disconnected from flare profiler.");
    profiler.connected = false;
}

socket.onerror = function(evt) {
    console.log("Connection error.");
    profiler.connected = false;
}

socket.onmessage = function (event) {
    //parse result
    var json = JSON.parse(event.data);
    profiler.onmessage(json);
};

function send(element) {
    var input = document.getElementById(element);
    socket.send(input.value);
    input.value = "";
}

function create_echarts_bar(elemId, echartsData) {
    if (!echartsData){
        echartsData = [];
        for (let i = 0; i < 3000; i++) {
            echartsData.push(Math.random().toFixed(2) * 1000);
        }
    }

    let options = {
        dataZoom: [{
            type: 'inside',
            start:0,
            end:10,
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
            realtime:false,
            filterMode:'empty',
            top:'top',
            left:'left'
        }],
        xAxis: {
            data: echartsData,
            show:false
        },
        yAxis: {show:false},
        series: [{
            type: 'bar',//bar
            data: echartsData,
            large: true,
            largeThreshold:50,
            itemStyle:{
                color: '#e74911', // bar颜色
                opacity: 0 // 透明度，0：不绘制
            }
        }]
    }
    var myChart = echarts.init(document.getElementById(elemId));
    myChart.setOption(options);
    return myChart;
}

var app = new Vue({
    el: '#app',
    data: {
        message: '',
        treeFilterText: '',
        treeProps: {
            children: 'children',
            label: 'label'
        },
        profiler: profiler
    },
    // components() {
    // 	"d3-flamegraph"
    // },
    watch: {
        treeFilterText(val) {
            this.$refs.tree.filter(val);
        }
    },
    methods: {
        filterNode(value, data) {
            if (!value) return true;
            return data.label.indexOf(value) !== -1;
        }
    },
    filters: {
        cpuTimeFilter(value) {
            return (value/1000000000).toFixed(2);
        },
        nodeLabelRender(node) {
            return "[dura={0},cpu={1},calls={2}] {3}".format(node.duration||0, (node.cpu||0)/1000, node.calls||0, node.label);
        }
    },
    created(){

    },
    mounted(){
        //create_echarts_bar('echartsId');
    },
});

function get_chrome_flame_chart(){
    return document.getElementById("chrome_flame_chart").contentWindow;
}
