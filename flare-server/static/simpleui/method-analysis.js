

var methodAnalysis = {
    method_features: [{
        name: 'Redis',
        style: 'db',
        includes: ['redis']
    },{
        name: 'SQL',
        style: 'db',
        includes: ['jdbc', 'mybatis', 'ibatis', 'jtds', 'dbcp']
    },{
        name: 'Hessian',
        style: 'rpc',
        includes: ['hessian']
    },{
        name: 'Thrift',
        style: 'rpc',
        includes: ['thrift']
    },{
        name: 'HttpClient',
        style: 'rpc',
        includes: ['HttpURLConnection','HttpClient','okhttp','feign','ribbon']
    },{
        name: 'Net',
        style: 'rpc',
        includes: ['java.net']
    },{
        name: 'Json',
        style: 'severe',
        includes: ['com.fasterxml.jackson']
    },{
        name: 'Zip',
        style: 'severe',
        includes: ['java.util.jar','java.util.zip']
    },{
        name: 'Log',
        style: 'severe',
        includes: ['logback']
    },{
        name: 'Major',
        style: 'main',
        includes: ['com.sun.proxy.$','gordian', 'szjlc', 'jlc']
    },{
        name: 'RxJava',
        style: 'gray',
        tag: false,
        includes: ['rx.observables','rx.internal','rx.Observable',]
    },{
        name: 'Reflect',
        style: 'gray',
        tag: false,
        includes: ['java.lang.reflect', 'sun.reflect', 'cglib']
    },{
        name: 'Spring',
        style: 'gray',
        tag: false,
        includes: ['org.springframework.aop', 'org.springframework.transaction', 'org.springframework.web', 'org.springframework.remoting']
    }],

    process_call_group(group){
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
        for(let i=0;i<group.call_stack.length;i++){
            group.call_stack[i].duration = first_method_call.durations[i];
        }

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
        group.features = group_features;
    },

    get_features_of_method(method_name) {
        let result = [];
        for (let i=0; i < this.method_features.length;i++){
            let feature = this.method_features[i];
            let includes = feature.includes;
            for (let x=0; x<includes.length;x++){
                if ( method_name.indexOf(includes[x])!=-1){
                    result.push(feature);
                    break;
                }
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
    }
};
