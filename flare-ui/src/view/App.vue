<template>
    <div>
        <!--header-->
        <div class="mg10" style="height: 35px;">
            <ul class="pull-left ul-menu">
                <li @click="drawer = true" class="background-hierarchy cursor mt5"></li>
                <li @click="addProjectInit" class="background-insert cursor mt5"></li>
                <li @click="refEchartsData" class="background-run cursor mt5"></li>
                <li class="background-download cursor mt5"></li>
                <li class="background-compare cursor mt5"></li>
                <li @click="selectFile" class="background-directory cursor mt5"></li>
                <li @click="helpInfo" class="background-help cursor mt5"></li>
            </ul>
            <div class="pull-left">
                <el-button v-for="tag in tags" type="" size="mini" plain>
                    {{tag}}<i @click="removeButton(tag)" class="el-icon-close el-icon--right"></i>
                </el-button>
            </div>
            <div class="mt5 pull-right">
                <el-dropdown trigger="click" @command="handleCommand">
                    <i class="background-menu pull-right cursor"></i>
                    <el-dropdown-menu slot="dropdown">
                        <el-dropdown-item command="changeInfo">关于</el-dropdown-item>
                        <el-dropdown-item command="exit">退出</el-dropdown-item>
                    </el-dropdown-menu>
                </el-dropdown>
            </div>

            <el-drawer title="title" :visible.sync="drawer"
                    direction="ltr"
                    :before-close="handleClose">
                <span>测试数据</span>
            </el-drawer>
        </div>
        <div style="clear: both;"></div>
        <div class="mg10">
            <div>
                <div style="margin: 20px auto;">
                    <h2>菜单操作数据：<el-tag type="info">{{text}}</el-tag></h2>
                    <h2>当前选中数据：{{chooseValue}}</h2>
                </div>
                <el-tree class="filter-tree" accordion :props="props" lazy :load="loadNode"
                         @current-change="currentChangeDate"></el-tree>
            </div>

            <el-dialog title="提示" :visible.sync="centerDialogVisible" width="50%">
                <span v-text="dialogConter"></span>
            </el-dialog>

            <el-dialog :title="isAddFile?'新建文件':'新建文件夹'" :visible.sync="addFileDialogFormVisible">
                <el-form :model="form">
                    <el-form-item label="文件名" v-if="isAddFile">
                        <el-input v-model="form.fileName"></el-input>
                    </el-form-item>
                    <el-form-item label="文件夹路径">
                        <el-input v-model="form.filePath" style="float: left">
                            <el-button slot="append" icon="el-icon-folder" @click="openDirectoryPath"></el-button>
                        </el-input>
                    </el-form-item>
                </el-form>
                <div slot="footer" class="dialog-footer">
                    <el-button @click="closeFileDialog">取 消</el-button>
                    <el-button type="primary" @click="saveFile">确 定</el-button>
                </div>
            </el-dialog>

            <!--<div class="mg10">-->
                <!--<el-button @click="refEchartsData">刷新</el-button>-->
            <!--</div>-->

            <!--echarts bar图表-->
            <!--<div id="echartsId" style="width: 900px;height:400px;" v-show="isShowEcharts"></div>-->

            <!--<h1>Event Drops Demo</h1>-->
            <!--<div id="eventdrops-demo" style="width: 90%;"></div>-->
            <div class="mg10">
                <el-button @click="refD3Line">刷新</el-button>
            </div>
            <div id="d3Div">
                <svg width="1100" height="400"></svg>
            </div>
        </div>
    </div>
</template>

<script>
    const fs = require('fs')
    const os = require('os')
    const { remote , shell, ipcRenderer} = require('electron')
    const { Menu, MenuItem, dialog, BrowserWindow } = remote
    import * as d3 from 'd3/build/d3';

    export default {
        name: 'app',
        data() {
            return {
                text: 'Electron Forge with Vue.js!',
                message: 'How is the weather today?',
                message2: "上天揽月，下海蛟龙",
                checkedNames: [],
                files: [],
                //tree显示模板
                props: {
                    label: 'name',
                    children: 'zones',
                    isLeaf: 'leaf'
                },
                path: 'D:\\',//默认tree加载的文件路径
                chooseValue:"",//选中的数据
                centerDialogVisible:false,//是否显示文件内容弹框
                dialogConter:"",//弹框文件内容
                addFileDialogFormVisible:false,//是否打开新建文件弹框
                isAddFile:true,//是否新建文件，默认为是
                //新建文件form
                form:{
                    fileName:'', //文件名
                    filePath:'',//文件夹路径
                },
                isShowEcharts:true,//是否显示echarts

                tags: ['标签一','标签二','标签三','标签四','标签五'],
                drawer: false,
                // echart实例
                myChart:{},
                updateChart:'',
                refCount:0,
            }
        },
        mounted(){
            //this.getD3Bar();
            this.getD3Line();
        },
        created() {

            // this.$nextTick(()=>{
            //     //this.echartsBar();
            //     this.getD3();
            // });
            // 监听主线程操作事件
            this.menuOption();
            //let win = new BrowserWindow();

            //this.text = win.getTitle();

            const menu = new Menu()
            menu.append(new MenuItem({ label: 'MenuItem1', click() { console.log('item 1 clicked') } }))
            menu.append(new MenuItem({ type: 'separator' }))
            menu.append(new MenuItem({ label: 'MenuItem2', type: 'checkbox', checked: true }))

            window.addEventListener('contextmenu', (e) => {
                e.preventDefault()
                menu.popup({ window: remote.getCurrentWindow() })
            }, false)
        },
        methods: {
            getD3Line() {
                let data = [];

                let count = 1000;
                for (let i = 0; i < count; i++) {
                    let info = {date: (i/100) ,price:Math.random()*200};
                    data.push(info);
                }
                this.updateChart = this.getD3LineChart('d3Div', data);
            },
            refD3Line() {
                let data = [];
                this.refCount = this.refCount + 2000
                let count = this.refCount;
                console.log('数据个数：', count)
                for (let i = 0; i < count; i++) {
                    let info = {date:i, price:Math.random()*200};
                    data.push(info);
                }
                //this.updateD3ChartDate(data)
                this.updateChart.updateD3ChartDate(data);
                //setInterval()
            },
            getD3LineChart (elementId, data){
                var svg = d3.select("#"+elementId).select("svg"),
                    margin = {top: 20, right: 20, bottom: 110, left: 40},
                    width = +svg.attr("width") - margin.left - margin.right,
                    height = +svg.attr("height") - margin.top - margin.bottom;

                var xData = [];

                for (let i = 0; i < 30000; i++) {
                    xData.push(i);
                }

                var x = d3.scaleLinear().range([0, width]);
                var x2 = d3.scaleLinear().range([0, width]);
                var y = d3.scaleLinear().range([height, 0]);

                var xAxis = d3.axisBottom(x).ticks(30);


                x.domain(d3.extent(data, function(d) {
                    return d.date;
                }));
                y.domain([0, d3.max(data, function(d) { return d.price; })]);
                x2.domain(x.domain());

                var zoom = d3.zoom()
                    .scaleExtent([1, 50])
                    .translateExtent([[0, 0], [width, height]])
                    .extent([[0, 0], [width, height]])
                    .on("zoom", zoomed);

                var area = d3.area()
                    .curve(d3.curveMonotoneX)
                    .x(function(d) { return x(d.date); })
                    .y0(height)
                    .y1(function(d) { return y(d.price); });

                svg.append("defs").append("clipPath")
                    .attr("id", "clip")
                    .append("rect")
                    .attr("width", width)
                    .attr("height", height);


                svg.append("rect")
                    .attr("class", "zoom")
                    .attr("width", width)
                    .attr("height", height)
                    .attr("transform", "translate(" + margin.left + "," + margin.top + ")")

                var focus = svg.append("g")
                    .attr("class", "focus")
                    .attr("transform", "translate(" + margin.left + "," + margin.top + ")");


                focus.append("path")
                    .datum(data)
                    .attr("class", "area")
                    .attr("d", area);

                focus.append("g")
                    .attr("class", "axis axis--x")
                    .attr("transform", "translate(0," + height + ")")
                    .call(xAxis);

                var tooltip = d3.select("#d3Div")
                    .append("div")
                    .attr("class","tooltip")
                    .style("opacity",0.0);

                var tooltipText = tooltip.append("div")
                    .attr("class", "desText");

                focus.on("mouseover", function (d,i) {
                    let numAry = d3.mouse(this);
                    let xValue = parseFloat(x.invert(numAry[0])).toFixed(2);
                    let yValue = parseFloat(y.invert(numAry[1])).toFixed(2);
                    let tooltipHtml = '';
                    tooltipHtml += '<div style="margin-left: 15px;">测试哇</div>';
                    tooltipHtml += '<div style="margin-left: 15px;">X:'+xValue+'</div>';
                    tooltipHtml += '<div style="margin-left: 15px;">Y:'+yValue+'</div>';
                    tooltip.html(tooltipHtml)
                        .style("left",(d3.event.pageX) +"px")
                        .style("top",(d3.event.pageY +20)+"px")
                        .style("opacity",1.0)
                }).on("mousemove",function (d,i) {
                    let numAry = d3.mouse(this);
                    let xValue = parseFloat(x.invert(numAry[0])).toFixed(2);
                    let yValue = parseFloat(y.invert(numAry[1])).toFixed(2);
                    let tooltipHtml = '';
                    tooltipHtml += '<div style="margin-left: 15px;">测试哇</div>';
                    tooltipHtml += '<div style="margin-left: 15px;">X:'+xValue+'</div>';
                    tooltipHtml += '<div style="margin-left: 15px;">Y:'+yValue+'</div>';
                    tooltip.html(tooltipHtml)
                        .style("left",(d3.event.pageX) +"px")
                        .style("top",(d3.event.pageY +20)+"px")
                        .style("opacity",1.0)
                }).on("mouseout",function (d,i) {
                    tooltip.style("opacity",0.0);
                });

                svg.call(zoom);

                function brushed() {
                    if (d3.event.sourceEvent && d3.event.sourceEvent.type === "zoom") return; // ignore brush-by-zoom
                    var s = d3.event.selection || x2.range();
                    x.domain(s.map(x2.invert, x2));
                    focus.select(".area").attr("d", area);
                    focus.select(".axis--x").call(xAxis);
                    svg.select(".zoom").call(zoom.transform, d3.zoomIdentity
                        .scale(width / (s[1] - s[0]))
                        .translate(-s[0], 0));
                }

                function zoomed() {
                    var t = d3.event.transform;
                    x.domain(t.rescaleX(x2).domain());
                    focus.select(".area").attr("d", area);
                    focus.select(".axis--x").call(xAxis);
                }

                function updateLineChart() {

                    this.updateD3ChartDate = function (updateData) {
                        x.domain(d3.extent(updateData, function(d) {
                            console.log('d.date:',d.date);
                            return d.date;
                        }));
                        x2.domain(x.domain());
                        xAxis = d3.axisBottom(x).ticks(30);

                        area = d3.area()
                            .curve(d3.curveMonotoneX)
                            .x(function(d) { return x(d.date); })
                            .y0(height)
                            .y1(function(d) { return y(d.price); });

                        focus.select("path")
                            .datum(updateData)
                            .attr("class", "area")
                            .attr("d", area);
                        focus.select("g")
                            .attr("class", "axis axis--x")
                            .attr("transform", "translate(0," + height + ")")
                            .call(xAxis);

                        function updateZoomed() {
                            var t = d3.event.transform;
                            x.domain(t.rescaleX(x2).domain());
                            focus.select(".area").attr("d", area);
                            focus.select(".axis--x").call(xAxis);
                        }
                    }
                }
                return new updateLineChart();
            },
            getD3Bar(){
                let height = 600
                let width = 750
                //画布周边的空白
                var padding = {left:30, right:30, top:20, bottom:20};
                var margin = {top: 20, right: 20, bottom: 110, left: 40};

                let dataset = [10,20,30,40,50,60,70,80,90,100]

                dataset = [];

                let xDomain = [];

                let count = 1000;

                for (let i = 1; i <= count; i++) {
                    xDomain.push(i);
                    dataset.push(Math.random() * 200)
                }

                let xScale = d3.scaleLinear().range([0, width]).domain(d3.extent(dataset, (d)=>{return d}));
                // x轴
                let xAxis = d3.axisBottom(xScale)


                //scaleExtent   Infinity
                var zoom = d3.zoom()
                    .scaleExtent([1, 20])
                    .translateExtent([[0, 0], [width, height]])
                    .extent([[0, 0], [width,height]])
                    .on("zoom", zoomed);

                let svg = d3.select('#d3Div').select('svg')
                    .attr('width',width)
                    .attr('height',height)
                    .attr("class", "focus")
                    .attr('class', 'mg50');

                // 柱状图
                let view = svg.append('g')
                    .attr("width", width)
                    .attr("height", height)
                    .attr('transform', "translate(0, 250)");

                let rect = view.selectAll('rect').data(dataset).enter().append('rect')
                    .attr('width',(d,i)=>{
                        return 0.1;
                    })
                    .attr('height', (d)=>{
                        console.log(d);
                        return d
                    })
                    .attr('x', (d,i)=>{
                        let x1 = width / dataset.length * i;
                        return x1;
                    })
                    .attr('y', (d,i)=>{return -d})
                    .attr('fill', 'blue');

                let gx = svg.append('g')
                    .attr("class", "axis axis--x")
                    .attr('transform', "translate(0, 250)")
                    //.attr('class', 'x axis')
                    .call(xAxis);

                svg.call(zoom);

                function zoomed() {// translate
                    //rect.attr("transform", d3.event.transform);
                    rect.attr('transform', 'translate('+d3.event.transform.x+',0)scale('+d3.event.transform.k+')');

                    //view.attr("transform", d3.event.transform);
                    gx.call(xAxis.scale(d3.event.transform.rescaleX(xScale)));
                    //svg.select(".x.axis").call(xAxis);
                }
            },
            getD3(){
                debugger
                const chart = eventDrops({ d3 });

                const repositoriesData = repositories.map(repository => ({
                    name: repository.name,
                    data: repository.commits,
                }));

                repositoriesData.forEach(item=>{
                    repositoriesData.data = repositoriesData.fullData;
                })

                console.log(repositoriesData);

                d3.select('#eventdrops-demo')
                    .data([repositoriesData])
                    .call(chart);
            },
            /*刷新echarts 渲染数据*/
            refEchartsData(){
                var dataCount = 5000;
                var data = this.generateData(dataCount);
                var option = {
                    series: [
                        {
                            name:'测试1',
                            type:'bar',
                            data:data.valueData,
                            large:true,
                            largeThreshold:500,
                            itemStyle:{
                                opacity:0
                            }
                        },
                    ]
                };
                this.myChart.setOption(option);
            },
            /*初始化echarts bar*/
            echartsBar(){
                var dataCount = 50000;
                var data = this.generateData(dataCount);

                this.myChart = this.$echarts.init(document.getElementById('echartsId'));

                var option = {
                    grid: {
                        right: '20%'
                    },
                    xAxis: {
                        show:false
                    },
                    yAxis: {
                        show:false
                    },
                    dataZoom: [{
                        start: 0,
                        end: 30,
                        handleIcon: 'M10.7,11.9v-1.3H9.3v1.3c-4.9,0.3-8.8,4.4-8.8,9.4c0,5,3.9,9.1,8.8,9.4v1.3h1.3v-1.3c4.9-0.3,8.8-4.4,8.8-9.4C19.5,16.3,15.6,12.2,10.7,11.9z M13.3,24.4H6.7V23h6.6V24.4z M13.3,19.6H6.7v-1.4h6.6V19.6z',
                        handleSize: '60%',
                        handleStyle: {
                            color: '#fff',
                            shadowBlur: 2,
                            shadowColor: 'rgba(0, 0, 0, 1)',
                            shadowOffsetX: 2,
                            shadowOffsetY: 2
                        },
                        top:'top',
                        left:50,
                        height:100,
                        showDetail:false,
                    }],
                    series: [
                        {
                            name:'测试1',
                            type:'bar',
                            data:data.valueData,
                            large:true,
                            largeThreshold:500,
                            itemStyle:{
                                opacity:0
                            }
                        },
                    ]
                };
                this.myChart.setOption(option);
            },
            generateData(count) {
                var baseValue = Math.random() * 1000;
                var time = +new Date(2011, 0, 1);
                var smallBaseValue;

                function next(idx) {
                    smallBaseValue = idx % 30 === 0
                        ? Math.random() * 900
                        : (smallBaseValue + Math.random() * 1500 - 250);
                    baseValue += Math.random() * 20 - 10;
                    return Math.max(
                        0,
                        Math.round(baseValue + smallBaseValue) + 3000
                    );
                }

                var categoryData = [];
                var valueData = [];

                for (var i = 0; i < count; i++) {
                    categoryData.push(i + "ms");
                    valueData.push(next(i).toFixed(2));
                    time += 1000;
                }

                return {
                    categoryData: categoryData,
                    valueData: valueData
                };
            },
            loadNode(node, resolve) {
                if (node.level === 0) {
                    fs.readdir(this.path,{"withFileTypes":true}, (error, files) => {
                        let nameList = [];
                        files.forEach(item => {
                            if (item.isDirectory()) {
                                let file = {name: item.name};
                                nameList.push(file);
                            }
                        })
                        return resolve(nameList);
                    })
                }

                if (node.level > 0) {
                    let url = this.path;
                    let node1 = node;
                    for (let i = node.level - 1; i >= 0; i--) {
                        for (let j = 0; j < i; j++) {
                            node1 = node1.parent;
                        }
                        if (node1.label) {
                            url = url + "\\" + node1.label;
                        }
                        node1 = node;
                    }
                    console.log("url地址：" + url);

                    fs.readdir(url, {'withFileTypes':true}, (error, files1)=>{
                        this.files = files1;
                        let nameList = [];
                        if (files1) {
                            files1.forEach(item=>{
                                let file = {name:item.name,leaf:item.isFile()};
                                nameList.push(file);
                            })
                        }
                        resolve(nameList);
                    });
                };
            },
            /*点击tree树触发*/
            currentChangeDate(data, node){
                this.chooseValue = '';
                let node1 = node;
                for (let i = node.level - 1; i >= 0; i--) {
                    for (let j = 0; j < i; j++) {
                        node1 = node1.parent;
                    }
                    if (node1.label) {
                        this.chooseValue = this.chooseValue + "\\" + node1.label;
                    }
                    node1 = node;
                }

                let url = this.path + this.chooseValue;
                console.log("需打开文件路径：" + url)
                fs.readFile(url, 'utf8', (err, data) => {
                    if (err) {
                        console.log(url + " 是目录，无法打开");
                        return;
                    }
                    //data = data.replace(/(\r\n)|(\n)/g,'<br>');
                    console.log(data);
                    this.centerDialogVisible = true;
                    this.dialogConter = data;
                })
            },
            /*监听菜单操作事件*/
            menuOption(){
                ipcRenderer.on("open-window1", (event,message) => {
                    console.log("更改的值:"+message)
                    this.text = message;
                })
                ipcRenderer.on('open-file-path', (event, filePaths) => {
                    console.log('选择的文件路径：' + filePaths);
                    this.text = filePaths;
                })
                ipcRenderer.on('open-file-directory-path', (event, paths) => {
                    console.log("选择的文件夹路径：" + paths);
                    this.text = paths;
                })
                ipcRenderer.on('add-file', (event, fileNames) => {
                    console.log("新建文件：" + fileNames);
                    this.text = fileNames;
                    this.addProjectInit();
                })
                ipcRenderer.on('select-directory-path', (event, paths) => {
                    console.log('选择文件夹路径：' + paths);
                    if (paths) {
                        this.form.filePath = paths[0];
                    }
                })
                ipcRenderer.on('add-directory', (event, arg) => {
                    console.log('新建文件夹：' + arg);
                    this.isAddFile = false;
                    this.addFileDialogFormVisible = true;
                })
                ipcRenderer.on('show-echarts', (event, arg) => {
                    console.log('显示echarts：' + arg)
                    this.isShowEcharts = true;
                })
                ipcRenderer.on('select-file-path', (event, arg) => {
                    console.log('选择的文件：' + arg)
                })
            },
            /*新建文件*/
            addProjectInit(){
                this.isAddFile = true;
                this.addFileDialogFormVisible = true;
            },
            /*选择文件*/
            selectFile(){
                ipcRenderer.send('open-select-file','openSelectFile');
            },
            /*打开文件目录*/
            openDirectoryPath(){
                console.log("打开文件目录")
                ipcRenderer.send('open-directory','openDirectory');
            },
            /*保存文件*/
            saveFile(){

                if (this.isAddFile && !this.form.fileName) {
                    this.$notify({title: '提示',message: '请填写文件名',type: 'warning'});
                    return;
                }
                if (!this.form.filePath) {
                    this.$notify({title: '提示',message: '请填写或者选择文件夹路径',type: 'warning'});
                    return;
                }

                try {
                    fs.accessSync(this.form.filePath, fs.constants.F_OK);
                    if (!this.isAddFile) {
                        this.$notify({title:'提示',message:this.form.filePath + ' 已经存在',type:'warning'});
                        return;
                    }
                } catch (e) {
                    console.log("文件不存在");
                    fs.mkdirSync(this.form.filePath, {'recursive':true})
                    if (!this.isAddFile) {
                        this.$notify({title:'提示',message:'创建成功',type:'success'});
                    }
                }

                if (this.isAddFile) {
                    let filePath = this.form.filePath + "\\" + this.form.fileName;

                    if (this.form.fileName.indexOf('.') <= 0) {
                        filePath = filePath + ".txt";
                    }

                    fs.writeFile(filePath, '  ', (err => {
                        if (err) {
                            console.log("创建文件出错，文件路径：" + filePath);
                            throw err;
                        }
                        this.$notify({title:'提示',message:'创建成功',type:'success'});
                    }))
                }

                this.form = {fileName:'',filePath:''};
                this.addFileDialogFormVisible = false;
            },
            /*关闭新建文件、文件夹弹框*/
            closeFileDialog(){
                this.form = {fileName:'',filePath:''};
                this.addFileDialogFormVisible = false;
            },
            /*删除button*/
            removeButton(name) {
                this.tags = this.tags.filter((item) => {
                    return item != name;
                })
            },
            /*应用信息*/
            helpInfo(){
                ipcRenderer.send('query-app-info', 'queryAppInfo');
            },
            /*菜单*/
            handleCommand(command){

            },
            openShell(){
                let openshelltag = document.getElementById("open-shell-tag");
                openshelltag.addEventListener('click', (event) => {
                    shell.showItemInFolder(os.homedir())
                })
            },
            createDialog() {
                //dialog.showOpenDialog();
                //dialog.showOpenDialogSync();
                dialog.showErrorBox('错误信息', '错误信息');
                //alert("11111111111")
            },
        },
    }
</script>
<style>


</style>
