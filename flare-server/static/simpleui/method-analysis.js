
function get_session_id(){
    return window.parent.profiler.data.session_id;
}

function get_profiler(){
    return window.parent.profiler;
}

var default_uistate = function () {
    return {
        thread_name_filter: "",
        search_methods: [],
        excluded_methods: [],
        min_method_duration: "200",
        max_method_duration: "",
        show_filter_methods: true,
        method_call_groups: [],
        current_call_group: null,
        search_method_message: "",
        search_method_error: false,
        call_stack_visible: false,
    };
}
var methodAnalysis = {
    data: {
        method_infos: [],
        total_method_size: 0,
    },
    uistate: default_uistate(),

    send(data){
        window.parent.profiler.socket.send(data);
    },
    clear_session(){
        methodAnalysis.data = {
            method_infos: [],
            total_method_size: 0,
        };
        methodAnalysis.uistate = default_uistate();
        methodAnalysis.load_excluded_methods();
    },
    list_methods_by_filter: function (method_name_filter) {
        methodAnalysis.uistate.show_filter_methods = true;
        if(methodAnalysis.list_method_timer != null){
            clearTimeout(methodAnalysis.list_method_timer);
            methodAnalysis.list_method_timer = null;
        }
        if (method_name_filter.trim() == ""){
            return;
        }
        //filter delay 500ms
        methodAnalysis.list_method_timer = setTimeout(function () {
            var request = {
                "cmd": "list_methods_by_filter",
                "options": {
                    "session_id": get_session_id(),
                    "method_name_filter": method_name_filter.trim()
                }
            };
            methodAnalysis.send(JSON.stringify(request));
        }, 200);
    },
    on_list_methods_result(data){
        //将response属性赋值到共享对象
        //Object.assign(methodAnalysis.data, data);

        methodAnalysis.data.method_infos = data.method_infos;
        methodAnalysis.data.total_method_size = data.total_method_size;
    },
    add_all_filter_methods(){
        let search_methods = methodAnalysis.uistate.search_methods.slice();
        for( var method_info of methodAnalysis.data.method_infos){
            if (!methodAnalysis.contains_search_method(method_info, search_methods)){
                search_methods.push(method_info);
            }
        }
        methodAnalysis.uistate.search_methods = search_methods;
    },
    add_search_method(method_info){
        if (!methodAnalysis.contains_search_method(method_info)){
            methodAnalysis.uistate.search_methods.push(method_info);
        }
    },
    contains_search_method(method_info, search_methods){
        search_methods = search_methods || methodAnalysis.uistate.search_methods;
        for( var m of search_methods) {
            //filter_excluded_methods
            // if (methodAnalysis.uistate.excluded_methods.indexOf(m.full_name)!=-1){
            //     return;
            // }
            if (m.method_id == method_info.method_id){
                return true;
            }
        }
        return false;
    },
    remove_search_method(method_info) {
        let search_methods = methodAnalysis.uistate.search_methods;
        search_methods.splice(search_methods.indexOf(method_info), 1);
    },
    clear_search_methods(){
        methodAnalysis.uistate.search_methods = [];
    },
    clear_search_message(){
        methodAnalysis.uistate.search_method_message = "";
        methodAnalysis.uistate.search_method_error = false;
    },
    add_excluded_method(method_name){
        method_name = method_name.trim();
        if(method_name.length == 0){
            return;
        }
        if (methodAnalysis.uistate.excluded_methods.indexOf(method_name) == -1 ){
            methodAnalysis.uistate.excluded_methods.push(method_name);
        }
        methodAnalysis.save_excluded_methods();
    },
    remove_excluded_method(method_name){
        methodAnalysis.uistate.excluded_methods.splice(methodAnalysis.uistate.excluded_methods.indexOf(method_name), 1);
        methodAnalysis.save_excluded_methods();
    },
    clear_excluded_methods(){
        methodAnalysis.uistate.excluded_methods = [];
        methodAnalysis.save_excluded_methods();
    },
    save_excluded_methods(){
        configs.setLocalStoreValue(configs.keys.excluded_methods, JSON.stringify(methodAnalysis.uistate.excluded_methods));
    },
    load_excluded_methods(){
        methodAnalysis.uistate.excluded_methods = configs.getLocalStoreValue(configs.keys.excluded_methods);
    },
    set_search_message(msg, error) {
        methodAnalysis.uistate.search_method_message = msg;
        methodAnalysis.uistate.search_method_error = error;
    },
    search_slow_methods(){
        let method_ids = [];
        for ( var m of methodAnalysis.uistate.search_methods){
            //filter_excluded_methods
            if (methodAnalysis.uistate.excluded_methods.indexOf(m.full_name)==-1){
                method_ids.push(m.method_id);
            }
        }
        if (method_ids.length == 0){
            methodAnalysis.set_search_message("Please specify searching methods!", true);
            methodAnalysis.uistate.show_filter_methods = true;
            return;
        }
        methodAnalysis.uistate.show_filter_methods = false;

        //parse duration
        var min_method_duration = methodAnalysis.uistate.min_method_duration.trim();
        if(min_method_duration == ""){
            min_method_duration = 1000;
        }else {
            min_method_duration = parseInt(min_method_duration);
        }
        methodAnalysis.uistate.min_method_duration = ""+min_method_duration;

        var max_method_duration = methodAnalysis.uistate.max_method_duration.trim();
        if(max_method_duration == ""){
            max_method_duration = -1;
        }else {
            max_method_duration = parseInt(max_method_duration);
            methodAnalysis.uistate.max_method_duration = ""+max_method_duration;
        }
        let thread_name_filter = methodAnalysis.uistate.thread_name_filter || "";
        methodAnalysis.set_search_message("Searching method calls ...");

        methodAnalysis.uistate.method_call_groups = [];
        var request = {
            "cmd": "search_slow_method_calls",
            "options": {
                "session_id": get_session_id(),
                "method_ids": method_ids,
                "min_duration": min_method_duration,
                "max_duration": max_method_duration,
                "max_size": 3000,
                "thread_name_filter": thread_name_filter
            }
        };
        methodAnalysis.send(JSON.stringify(request));
    },
    on_slow_method_calls(data){
        //将response属性赋值到共享对象
        Object.assign(methodAnalysis.data, data);
        //update search message
        methodAnalysis.uistate.search_method_error = data.search_error;
        if(data.search_error){
            methodAnalysis.uistate.search_method_message = data.search_message;
        }else if(data.search_finished){
            methodAnalysis.uistate.search_method_message = "Search finished."
        }else if (data.search_progress){
            methodAnalysis.uistate.search_method_message = "Search progress: "+data.search_progress+"% ";
            // methodAnalysis.uistate.search_method_message += (data.search_message?(", "+data.search_message):"");
        }else {
            methodAnalysis.uistate.search_method_message = "";
        }

        //merge array method_calls
        if (data.method_call_groups){
            //let groups =  methodAnalysis.uistate.method_call_groups.slice(0);
            let groups =  [];
            for (var group of data.method_call_groups) {
                methodAnalysis.process_call_group(group);
                //keep 10 method_calls
                //group.method_calls = group.method_calls.slice(0,10);
                groups.push(group);
            }

            //default sort by calls
            methodAnalysis.sort_slow_method_calls('calls', groups);
        }
    },

    sort_slow_method_calls(command, groups){
        //如果groups 不是数组则重新克隆一份（可能是vue组件）
        if(!groups || !groups.length){
            //groups = methodAnalysis.uistate.method_call_groups.slice(0);
            groups = [];
            for(var item of methodAnalysis.uistate.method_call_groups){
                groups.push(item);
            }
        }
        if (command == 'max_duration'){
            groups.sort(function (a, b) {
                if ( a.max_duration < b.max_duration){
                    return 1;
                } else if ( a.max_duration > b.max_duration) {
                    return -1;
                }
                return 0;
            });
        } else if (command == 'avg_duration') {
            groups.sort(function (a, b) {
                if ( a.avg_duration < b.avg_duration){
                    return 1;
                } else if ( a.avg_duration > b.avg_duration) {
                    return -1;
                }
                return 0;
            });
        } else if (command == 'calls') {
            groups.sort(function (a, b) {
                if ( a.method_calls.length < b.method_calls.length){
                    return 1;
                } else if ( a.method_calls.length > b.method_calls.length) {
                    return -1;
                }
                return 0;
            });
        } else if(command == 'method_name'){
            groups.sort(function (a, b) {
                if (a.first_method_name < b.first_method_name) {
                    return -1;
                } else if (a.first_method_name > b.first_method_name) {
                    return 1;
                }
                return 0;
            });
        } else if(command == 'similarity'){
            //TODO 改进分组相似度计算方法

            //category by similarity
            let similarity_category_stack_ids = [];
            let similarity_category_list = [];

            for(var group of groups){
                if (similarity_category_stack_ids.length == 0){
                    //create_group_category(group, similarity_category_list, similarity_category_stack_ids);
                    similarity_category_list.push([group]);
                    similarity_category_stack_ids.push(group.call_stack_ids);
                } else {
                    let max_category_idx=0;
                    let max_similarity = 0;
                    for(var i=0;i<similarity_category_stack_ids.length;i++){
                        let stack_ids = similarity_category_stack_ids[i];
                        let count = array_match_count(group.call_stack_ids, stack_ids );
                        let similarity = count*2 / (group.call_stack_ids.length + stack_ids.length);
                        if (similarity > max_similarity){
                            max_similarity = similarity;
                            max_category_idx = i;
                        }
                    }
                    if (max_similarity > 0.6 ){
                        similarity_category_list[max_category_idx].push(group);
                    }else {
                        //create new category
                        similarity_category_list.push([group]);
                        similarity_category_stack_ids.push(group.call_stack_ids);
                    }
                }
            }

            similarity_category_list.sort(function (a, b) {
               return a.length - b.length;
            });
            groups = [];
            for (var category of similarity_category_list){
                //sort groups by name
                category.sort(function (a,b) {
                    if (a.first_method_name < b.first_method_name) {
                        return -1;
                    } else if (a.first_method_name > b.first_method_name) {
                        return 1;
                    }
                    return 0;
                });
                Array.prototype.push.apply(groups, category);
            }

            //sort by similarity
            // let std_group_stack_ids = groups[0].call_stack_ids;
            // groups.sort(function (a, b) {
            //     //features 多的排在前面
            //     let feat_count = array_match_count(a.features, b.features);
            //     if (feat_count != a.features.length || feat_count != b.features.length){
            //         if( a.features.length < b.features.length ){
            //             return 1;
            //         }else if(a.features.length > b.features.length ){
            //             return -1;
            //         }
            //         if( a.first_method_name < b.first_method_name ){
            //             return -1;
            //         }else if( a.first_method_name > b.first_method_name){
            //             return 1;
            //         }
            //     }
            //
            //     //比较相同的方法数量
            //     let method_count = array_match_count(a.call_stack_ids, b.call_stack_ids);
            //     let similarity_a = method_count / a.call_stack_ids.length;
            //     let similarity_b = method_count / b.call_stack_ids.length;
            //     return ( similarity_a - similarity_b);
            // });
        }
        methodAnalysis.uistate.method_call_groups = groups;
    },

    process_call_group(group){
        if (group.inited){
            return;
        }
        group.inited = true;

        group.method_calls.sort(function (a, b) {
            if (a.duration < b.duration){
                return 1;
            }else if(a.duration > b.duration) {
                return -1;
            }
            return 0;
        });

        //调用栈第一个方法
        try {
            group.first_method_name = group.call_stack[0].full_name;
        }catch (e) {
        }

        //copy first method call duration
        let first_method_call = group.method_calls[0];
        group.duration = first_method_call.duration;
        let call_stack_ids = [];
        for(let i=0;i<group.call_stack.length;i++){
            group.call_stack[i].duration = first_method_call.durations[i];
            call_stack_ids.push(group.call_stack[i].method_id);
        }
        group.call_stack_ids = call_stack_ids;

        //计算最大/最小/平均 调用时间
        let max=null,min=null,avg,total=0;
        for (var method_call of group.method_calls) {
            if (!max){
                max = min = method_call.duration;
            } else {
                if (max < method_call.duration){
                    max = method_call.duration;
                } else if(min > method_call.duration){
                    min = method_call.duration;
                }
            }

            total += method_call.duration;
        }
        avg = Math.round(total / group.method_calls.length);
        group.max_duration = max;
        group.min_duration = min;
        group.avg_duration = avg;

        methodAnalysis.process_features(group);
    },

    //TODO 对方法栈特征识别
    process_features(group){
        let group_features = [];
        for (var call of group.call_stack){
            let features = this.get_features_of_method(call.full_name);
            call.features = features;
            if (features.length > 0){
                call.feature_style = features[0].style;
            }
            //merge features
            for (var i =0;i<features.length;i++){
                let feat = features[i];
                if(group_features.indexOf(feat)==-1){
                    group_features.push(feat);
                }
            }
        }

        /* 去除重复数据 */
        for (let i = 0; i < group_features.length; i++) {
            for (let j = i + 1; j < group_features.length; j++) {
                if (i !== j && group_features[i].name == group_features[j].name) {
                    group_features.splice(j, 1);
                    j--;
                }
            }
        }

        group.features = group_features;
    },

    get_features_of_method(method_name) {

        let methodFeatures = configs.getLocalStoreValue(configs.keys.method_features);

        let result = [];
        for (let i=0; i < methodFeatures.length;i++){
            let feature = methodFeatures[i];
            let includes = feature.includes;
            let found = false;
            for (let x=0; x<includes.length;x++){
                if (method_name.indexOf(includes[x])!=-1){
                    result.push(feature);
                    found = true;
                    break;
                }
            }
            //only first feature
            if (found){
                break;
            }
        }
        return result;
    },
    format_features(features){
        let str = "";
        for(let i=0;i<features.length;i++){
            str += features[i].name;
            if(i < features.length-1){
                str +=","
            }
        }
        return str;
    },

    jump_to_method_call(method_call){
        get_profiler().jump_to_method_call(method_call);
    },

    show_call_stack_of_group(call_group){
        methodAnalysis.uistate.current_call_group = call_group;
        methodAnalysis.uistate.call_stack_visible = true;
        setTimeout(function () {
            document.getElementById("call_stack_div").focus();
        }, 500);
    },
    hide_call_stack_of_group(){
        methodAnalysis.uistate.call_stack_visible = false;
    },
};

var app = new Vue({
    el: '#method_analysis',
    data: {
        message: '',
        methodFilterText: '',
        excludedMethodText: '',
        dialogExcludedMethodsVisible: false,
        methodAnalysis: methodAnalysis,
        profiler: get_profiler(),
    },
    watch: {
        // methodFilterText(val) {
        //     methodAnalysis.list_methods_by_filter(val);
        // }
    },
    methods: {
        select_filter_method(method){
            this._data.methodFilterText = method;
            methodAnalysis.list_methods_by_filter(method);
        },
        add_excluded_method2(method_name){
            this.$confirm('是否将方法['+method_name+']添加到排除列表?', '提示', {
                confirmButtonText: '确定',
                cancelButtonText: '取消',
                type: 'warning'
            }).then(() => {
                methodAnalysis.add_excluded_method(method_name);
                this.$notify({
                    title: '成功',
                    message: '已将方法['+method_name+']添加到排除列表，请重新执行分析操作。',
                    type: 'success'
                })
            }).catch(() => {
                // this.$notify.error({
                //     title: '失败',
                //     message: '添加方法['+method_name+']到排除列表失败！'
                // })
            });
        },
        open_excluded_method_dialog() {
            this.dialogExcludedMethodsVisible = true;
            methodAnalysis.load_excluded_methods();
        },
    },
    filters: {
    },
    created(){
    },
    mounted(){
        methodAnalysis.load_excluded_methods();
    },
});


function array_match_count(a,b) {
    let count = 0;
    for (var i=0;i<a.length;i++){
        if (b.indexOf(a[i])!=-1){
            count += 1;
        }
    }
    return count;
}