
//填充数据，使得不同线程的时间坐标系(X)范围相同比例相同（不同线程的开始时间、结束时间不同，自由显示则比例不一致）
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


var default_uistate = function () {
    return {
        last_loading_thread_cpu_time: 0,
        is_loaded_dashboard: false,
        unit_time_ms: 0,
        cpu_charts: {},
        //等待打开的方法调用，等待ts数据加载完毕后再打开
        jumping_method_call: null,
    };
}

let chartIdPrefix = "thread_cpu_chart_";

var profiler = {
    connected: false,
    agent_addr: "localhost:3333",
    profiler_addr: "localhost:3891",
    // profiler_addr: "192.168.2.220:3891",
    socket: null,
    sample_dir: null,
    dashboard_timer: null,
    list_method_timer: null,
    show_history_samples: false,
    show_message: false,
    show_sessions: false,

    show_call_tree: false,
    show_flame_graph: false,
    show_d3_flame_graph: false,
    show_chrome_flame_chart: true,

    selected_thread_id: null,
    selected_thread_name: null,
    stats_type: "duration",
    flame_graph_state: {
        start_time: 0,
        end_time: 0,
        thread_id: null,
        chart: null,
    },
    tabs: {
        profile: 'profile',
        dashboard: 'dashboard',
        threads: 'threads',
        call_graph: 'call_graph',
        call_tree: 'call_tree',
        method_analysis: 'method_analysis',
    },
    uistate: default_uistate(),
    data: {
        version: 'Flare Profiler v0.2.0',
        activeTab: 'profile',
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
    parse_host(url){
        var s = url.indexOf('://');
        var p = url.indexOf('/', s+3);
        var addr = url.substring(s+3, p);

        var p2 = addr.indexOf(':');
        if (p2 != -1){
            return addr.substring(0, p2);
        }
        return addr;
    },
    init(){
        profiler.profiler_addr = this.parse_host(window.location.href)+":3891";
        console.info("Connecting flare websocket server: "+profiler.profiler_addr);
        var socket = new WebSocket("ws://"+profiler.profiler_addr, "flare-profiler");
        socket.onopen = function(evt) {
            console.log("Connected to flare profiler successfully.");
            profiler.connected = true;
            profiler.list_sessions();
            profiler.start_auto_refresh();
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
        this.socket = socket;

        //注册滚动事件
        // document.getElementById('cpu_time_content').addEventListener('scroll', function(e) {
        //     if (profiler.data.activeTab == profiler.tabs.threads){
        //         profiler.on_scroll_thread_cpu_charts();
        //     }
        // });
    },
    start_auto_refresh() {
        if (this.dashboard_timer == null) {
            this.do_refresh(true);
            this.dashboard_timer = setInterval(function () {
                profiler.do_refresh();
            }.bind(this), 2000);
        }
    },
    stop_auto_refresh() {
        if (this.dashboard_timer != null) {
            clearInterval(this.dashboard_timer);
            this.dashboard_timer = null;
        }
    },
    do_refresh(force){
        if (profiler.data.session_id == ""){
            return;
        }
        let should_update_dashboard = (force || !profiler.uistate.is_loaded_dashboard || profiler.data.type == "attach");
        switch (profiler.data.activeTab) {
            case profiler.tabs.threads:
                //case profiler.tabs.call_graph: //暂时不刷新火焰图页面的线程CPU图，因为数据变化导致选择的范围改变
                if(should_update_dashboard) {
                    profiler.update_dashboard();
                }
                setTimeout(function () {
                    profiler.update_cpu_time();
                }, 200);
                break;
            case profiler.tabs.dashboard:
                if(should_update_dashboard) {
                    profiler.update_dashboard();
                }
                break;
        }
    },
    activeTab(tab) {
        this.data.activeTab = tab;
    },
    update_dashboard(){
        if (profiler.data.session_id == ""){
            return;
        }
        console.log("send request: get_dashboard");
        this.socket.send(JSON.stringify({
            "cmd": "dashboard",
            "options": {
                "session_id": profiler.data.session_id
            }
        }));
        this.uistate.is_loaded_dashboard=true;
    },
    update_cpu_time(){
        if (!profiler.uistate.is_loaded_dashboard){
            return;
        }

        var thread_ids = [];
        // for ( var i=0;i<profiler.data.threads.length;i++) {
        //     thread_ids.push(profiler.data.threads[i].id);
        // }
        // if ( profiler.data.activeTab == profiler.tabs.call_graph) {
        //     //如果激活火焰图标签页，则只刷新选择的线程CPU图
        //     thread_ids.push(profiler.flame_graph_state.thread_id);
        //} else
        if (profiler.data.activeTab == profiler.tabs.threads) {
            //计算当前在可见区域的线程
            var view = document.getElementById("cpu_time_region");
            let top = view.scrollTop;
            let bottom = top + view.offsetHeight;

            let items = document.getElementById("cpu_time_content").children;
            for(var i=0;i<items.length;i++){
                let item = items[i];
                //显示区域相交：item.top < view.bottom && item.bottom > view.top
                if(item.offsetTop < bottom && item.offsetTop+item.offsetHeight > top){
                    let chartId = item.children[1].id;
                    if (chartId.startsWith(chartIdPrefix)){
                        thread_ids.push(parseInt(chartId.substring(chartIdPrefix.length)));
                    }
                }
                if (item.offsetTop > bottom) {
                    break;
                }
            }
            //如果为打开文件，不需要重复加载统计图数据
            if (profiler.data.type == 'file'){
                let new_thread_ids = [];
                for(var i=0;i<thread_ids.length;i++){
                    if(!profiler.uistate.cpu_charts[chartIdPrefix+thread_ids[i]]){
                        new_thread_ids.push(thread_ids[i]);
                    }
                }
                thread_ids = new_thread_ids;
            }
            //如果没有线程需要更新，则终止
            if (thread_ids.length == 0){
                return;
            }
        }else {
            //其它tab标签不用更新线程CPU图表
            return;
        }

        profiler.load_cpu_time(thread_ids);
    },
    load_cpu_time(thread_ids){
        var graph_width = 900;
        var sample_interval = profiler.data.sample_info.sample_interval;
        var start_time = profiler.data.sample_info.record_start_time;
        var end_time = profiler.data.sample_info.last_record_time;
        var ratio = Math.ceil((end_time - start_time) / graph_width / sample_interval);
        if (ratio > 10 ){
            ratio = Math.floor(ratio/10)*10;
        }
        var unit_time_ms = ratio * sample_interval;
        if (!unit_time_ms) {
            console.log("update_cpu_time error, unit_time_ms is NaN");
            return;
        }
        var now = new Date().getTime();
        //暂时避免CPU图表更新混乱而导致的闪烁问题
        if (now - profiler.uistate.last_loading_thread_cpu_time < 500){
            return;
        }
        profiler.uistate.last_loading_thread_cpu_time = now;
        profiler.uistate.unit_time_ms = unit_time_ms;

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
        console.log("send request: "+JSON.stringify(request));
        this.socket.send(JSON.stringify(request));
        // console.log("update_cpu_time: ", request);
    },
    on_scroll_thread_cpu_charts(){
        console.log("on_scroll_thread_cpu_charts ..")
        if(profiler.update_cpu_chart_timer){
            clearTimeout(profiler.update_cpu_chart_timer);
        }
        profiler.update_cpu_chart_timer = setTimeout(function () {
            profiler.update_cpu_time();
            profiler.update_cpu_chart_timer = null;
        }, 100);
    },
    list_sessions() {
        this.show_sessions = true;
        this.show_history_samples = false;
        this.socket.send(JSON.stringify({
            "cmd": "list_sessions"
        }))
    },
    list_history: function () {
        this.show_sessions = false;
        this.show_history_samples = true;
        this.socket.send(JSON.stringify({
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
        this.socket.send(JSON.stringify(request));
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
        this.socket.send(JSON.stringify(request));
    },
    close_all_session() {
        this.connected = false;
        var request = {
            "cmd": "close_all_session",
            "options": {
            }
        };
        this.socket.send(JSON.stringify(request));
    },
    open_sample: function (sample_data_dir) {
        this.clear_session();
        var request = {
            "cmd": "open_sample",
            "options": {
                "sample_data_dir": sample_data_dir
            }
        };
        this.socket.send(JSON.stringify(request));
    },
    active_session:function (session_id, type) {
        this.clear_session();
        profiler.data.session_id = session_id;
        profiler.data.type = type;
        this.activeTab(this.tabs.dashboard);
        this.update_dashboard();
    },
    clear_session: function () {
        for (let [key, value] of Object.entries(this.uistate.cpu_charts)) {
            //console.log(`${key}: ${value}`);
            value.off('datazoom');
            value.dispose();
        }
        this.data.session_id = "";
        this.data.threads = [];
        this.data.sample_info = {};
        this.data.thread_cpu_time_map = {};
        this.uistate = default_uistate();

        let methodAnalysis = get_method_analysis();
        if(methodAnalysis) methodAnalysis.clear_session();
    },
    on_cpu_time_result(data){
        var sess_start_time = profiler.data.sample_info.record_start_time;
        var sess_end_time = profiler.data.sample_info.last_record_time;

        for (let i = 0; i < data.thread_cpu_times.length; i++) {
            let thread = data.thread_cpu_times[i];
            let unit_time_ms = thread.unit_time_ms;
            let ts_data = fill_ts_data(thread.ts_data, thread.start_time, thread.end_time, sess_start_time, sess_end_time, unit_time_ms);

            let chartElemId = chartIdPrefix+thread.id;
            let myChart = profiler.uistate.cpu_charts[chartElemId];
            if (myChart){
                myChart.off('datazoom');
                //myChart.dispose();
                update_echarts_bar(myChart, ts_data)
            }else {
                myChart = create_echarts_bar(chartElemId, ts_data);
                profiler.uistate.cpu_charts[chartElemId] = myChart;
            }
            myChart.ts_data = ts_data;
            myChart.thread = thread;

            myChart.on('datazoom', function (evt) {
                var axis = myChart.getModel().option.xAxis[0];
                // var starttime = axis.data[axis.rangeStart];
                // var endtime = axis.data[axis.rangeEnd];
                let start_time = sess_start_time + axis.rangeStart*unit_time_ms;
                let end_time = sess_start_time + axis.rangeEnd*unit_time_ms;
                console.log("datazoom: thread:",thread.id, ", index:", axis.rangeStart,"-", axis.rangeEnd,", time:", start_time,"-", end_time );
                //profiler.update_stack_stats(thread.id, start_time, end_time, myChart);
                profiler.update_call_graph_thread_cpu_data(thread.id, start_time, end_time, ts_data, unit_time_ms, sess_start_time, evt.start, evt.end);
            });
            profiler.data.thread_cpu_time_map[thread.id] = thread;

            //continue open method call
            let jumping_method_call = profiler.uistate.jumping_method_call;
            if(jumping_method_call && jumping_method_call.thread_id == thread.id){
                profiler.jump_to_method_call();
            }

        }
        profiler.uistate.last_loading_thread_cpu_time = new Date().getTime();

    },
    update_call_graph_thread_cpu_data(thread_id, start_time, end_time, ts_data, unit_time_ms, sess_start_time, start, end) {
        //active 'Call Graph' tab
        this.activeTab(this.tabs.call_graph);

        //update current thread cpu graph
        let myChart = create_echarts_bar("thread_cpu_chart_call_graph", ts_data, start, end);
        myChart.off('datazoom');
        myChart.on('datazoom', function (evt) {
            var axis = myChart.getModel().option.xAxis[0];
            let start_time = sess_start_time + axis.rangeStart*unit_time_ms;
            let end_time = sess_start_time + axis.rangeEnd*unit_time_ms;
            console.log("datazoom: thread:",thread_id, ", index:", axis.rangeStart,"-", axis.rangeEnd,", time:", start_time,"-", end_time );
            profiler.update_stack_stats(thread_id, start_time, end_time, myChart);
        });
        profiler.update_stack_stats(thread_id, start_time, end_time, myChart);
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
        this.socket.send(JSON.stringify(request));
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

        this.socket.send(JSON.stringify(request));
    },
    update_d3_flame_graph(thread_id, start_time, end_time) {
        var request = {
            "cmd": "sequenced_call_tree",
            "options": {
                "session_id": profiler.data.session_id,
                "thread_id": thread_id,
                "start_time": start_time,
                "end_time": end_time,
                "stats_type": profiler.stats_type
            }
        };
        this.socket.send(JSON.stringify(request));
    },
    select_thread(thread_id, thread_name) {
        this.selected_thread_id = thread_id;
        this.selected_thread_name = thread_name;
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
    jump_to_method_call(method_call){
        method_call = method_call || profiler.uistate.jumping_method_call;
        let chartElemId = chartIdPrefix + method_call.thread_id;
        let myChart = profiler.uistate.cpu_charts[chartElemId];
        //load chart data as required
        if (!myChart){
            //TODO load thread cpu chart
            profiler.uistate.jumping_method_call = method_call;
            profiler.load_cpu_time([method_call.thread_id]);
        } else {
            profiler.uistate.jumping_method_call = null;

            let thread = myChart.thread;
            let ts_data = myChart.ts_data;
            let unit_time_ms = thread.unit_time_ms;
            let thread_start_time = thread.start_time;

            var sess_start_time = profiler.data.sample_info.record_start_time;
            var sess_end_time = profiler.data.sample_info.last_record_time;
            let total_time = sess_end_time - sess_start_time;

            //设置显示范围
            let start_time = thread_start_time + method_call.start_time;
            let end_time = start_time + method_call.duration;
            start_time -= 500;
            end_time += 500;
            //let start_time = sess_start_time + rangeStart*unit_time_ms;
            //let end_time = sess_start_time + rangeEnd*unit_time_ms;
            let rangeStart = (start_time-sess_start_time)/unit_time_ms;
            let rangeEnd = (end_time-sess_start_time)/unit_time_ms;
            let rangeTotal = total_time/unit_time_ms;
            let start_percent = 100*rangeStart/rangeTotal;
            let end_percent = 100*rangeEnd/rangeTotal;

            console.log("jump_to_method_call: thread:",thread.id, ", index:", rangeStart,"-", rangeEnd,", time:", start_time,"-", end_time );
            profiler.update_call_graph_thread_cpu_data(thread.id, start_time, end_time, ts_data, unit_time_ms, sess_start_time, start_percent, end_percent);
        }
    },
    onmessage(json){
        var success = (json.result == "success");
        profiler.show_message = !success;
        //将response属性赋值到共享对象
        Object.assign(profiler.data, json.data);
        switch (json.cmd) {
            case "dashboard":
                //profiler.on_dashboard_result(json.data);
                break;
            case "open_sample":
                if (success) {
                    profiler.list_sessions();
                    profiler.update_dashboard();
                    profiler.activeTab(profiler.tabs.dashboard);
                }
                break;
            case "connect_agent":
                if (success) {
                    profiler.list_sessions();
                    profiler.update_dashboard();
                    profiler.activeTab(profiler.tabs.dashboard);
                }
                break;
            case "close_session":
            case "close_all_session":
                profiler.clear_session();
                profiler.list_sessions();
                break;
            case "history_samples":
                profiler.show_history_samples = true;
                break;
            case "cpu_time":
                setTimeout(function () {
                    profiler.on_cpu_time_result(json.data);
                }, 50);
                break;
            case "flame_graph":
                profiler.set_zoom_time_range(json.data.start_time, json.data.end_time);
                break;
            case "sequenced_call_tree":
                let stack = json.data.sequenced_call_tree_data;
                if(profiler.show_chrome_flame_chart){
                    get_chrome_flame_chart().set_flame_chart_data(stack);
                } else if(profiler.show_d3_flame_graph){
                    process_d3_flamegraph_stack(stack)
                    set_d3_flamegraph_data(stack);
                }
                break;
            case "list_methods_by_filter":
                get_method_analysis().on_list_methods_result(json.data);
                break;
            case "search_slow_method_calls":
                get_method_analysis().on_slow_method_calls(json.data);
                break;
            default:
                console.log("unknown message: ", json);
                break;
        }
    }
}

document.onreadystatechange=function () {
    //document.getElementById("flame_graph_svg").onmousewheel = profiler.onFlameMouseWheel;
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

function create_echarts_bar(elemId, echartsData, start, end) {
    if (!echartsData){
        echartsData = [];
        for (let i = 0; i < 3000; i++) {
            echartsData.push(Math.random().toFixed(2) * 1000);
        }
    }
    start = start || 0;
    end = end || 10;

    let options = {
        dataZoom: [{
            type: 'inside',
            start: start,
            end: end,
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
            // large: true,
            // largeThreshold:50,
            showSymbol: true,
            hoverAnimation: false,
            animation: false,
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

function update_echarts_bar(myChart, echartsData, start, end) {
    myChart.setOption({
        xAxis: {
            data: echartsData,
        },
        series:[{
            data:echartsData
        }]
    });
}


function get_chrome_flame_chart(){
    return document.getElementById("chrome_flame_chart").contentWindow;
}

function get_method_analysis(){
    let frame = document.getElementById("method_analysis_frame");
    if(frame){
        return frame.contentWindow.methodAnalysis;
    }
    return null;
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
        profiler: profiler,
    },
    // components() {
    // 	"d3-flamegraph"
    // },
    watch: {
        treeFilterText(val) {
            this.$refs.tree.filter(val);
        },
    },
    methods: {
        filterNode(value, data) {
            if (!value) return true;
            return data.label.indexOf(value) !== -1;
        },
        handleTabClick(tab, event) {
            console.log("active tab: ", tab.name);
            if ( tab.name == profiler.tabs.threads){
                // if(profiler.data.type == "attach") {
                // }
                setTimeout(function () {
                    profiler.update_cpu_time();
                }, 50);
            } else if(tab.name == profiler.tabs.dashboard){
                profiler.update_dashboard();
            }
        },
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
profiler.init();