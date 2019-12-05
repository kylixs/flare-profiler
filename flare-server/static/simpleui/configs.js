let method_features = [{
    name: 'JAR',
    style: 'severe',
    includes: ['java.util.jar','java.util.zip']
},{
    name: 'Json',
    style: 'severe',
    includes: ['com.fasterxml.jackson','net.sf.json','fastjson']
},{
    name: 'Log',
    style: 'severe',
    includes: ['logback']
},{
    name: 'Wait',
    style: 'severe',
    includes: ['CountDownLatch.await()','java.util.concurrent.locks','Unsafe.park()']
},{
    name: 'Except',
    style: 'severe',
    includes: ['java.lang.Exception','java.lang.Throwable','IllegalArgumentException','RuntimeException']
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
    includes: ['java.net','org.apache.tomcat.util.net']
},{
    name: 'File',
    style: 'rpc',
    includes: ['FileOutputStream','FileInputStream']
},{
    name: 'IO',
    style: 'rpc',
    includes: ['.read()','.doRead()','readFully()','.write()','.doWrite()','.writeAndFlush()','.flush','PrintStream']
},{
    name: 'Redis',
    style: 'db',
    includes: ['redis']
},{
    name: 'SQL',
    style: 'db',
    includes: ['jdbc', 'mybatis', 'ibatis', 'jtds', 'dbcp']
},{
    name: 'Cache',
    style: 'db',
    includes: ['LocalCache']
},{
    name: 'Filter',
    style: 'gray',
    tag: false,
    includes: ['doFilter()','internalDoFilter()']
},{
    name: 'Struts',
    style: 'gray',
    tag: false,
    includes: ['struts2.','xwork2.']
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
},{
    name: 'Major',
    style: 'main',
    includes: ['com.sun.proxy.$','_jsp','gordian', 'szjlc', 'jlc'],
    excludes: ['doFilter()']
}];

let excluded_methods = ['doFilter()','doFilterInternal()'];

/* 如下所示，当前配置key、 配置菜单中configCode、本地存储key 此三项需要保持一致，默认配置key = 当前配置key + '_source'*/
var configs = {
    method_features: method_features, // 当前配置key、value
    method_features_source: method_features, // 默认配置，用于重置使用
    excluded_methods: excluded_methods,
    excluded_methods_source: excluded_methods,
    configMenuList: [
            { configId: 1, configName: 'Method Features', configCode:'method_features' }, // 设置中左侧菜单栏，根据configCode匹配配置项以及设置本地存储
            { configId: 2, configName: 'Excluded Methods', configCode:'excluded_methods' }, // 设置中左侧菜单栏，根据configCode匹配配置项以及设置本地存储
        ],
    /* 本地存储key */
    keys: {
        method_features: "method_features",
        excluded_methods: "excluded_methods",
        flare_profiler: 'flare-profiler.',
    },
    /* 根据指定key获取配置数据 */
    getLocalStoreValue(key) {
        let localStoreValue = [];
        // 如果本地存在数据，则使用本地存储的配置数据

        let itemValue = localStorage.getItem(configs.keys.flare_profiler + key);
        if (itemValue) {
            localStoreValue = JSON.parse(itemValue);
        } else {
            localStoreValue = configs[key];
        }
        return localStoreValue;
    },
    /* 根据指定key设置配置数据 */
    setLocalStoreValue(key, value) {
        configs[key] = value;
        localStorage.setItem(configs.keys.flare_profiler + key, value);
    }
}