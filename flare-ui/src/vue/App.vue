<template>
    <div id="app">
        <!--header-->
        <div class="mg10" style="height: 35px;">
            <ul class="pull-left ul-menu">
                <li @click="drawer = true" class="background-hierarchy cursor mt5"></li>
                <li @click="addProjectInit" class="background-insert cursor mt5"></li>
                <li @click="" class="background-run cursor mt5"></li>
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
                        <el-dropdown-item command="about">关于</el-dropdown-item>
                        <el-dropdown-item command="reload">重新加载</el-dropdown-item>
                        <el-dropdown-item command="toggleDevTools">切换开发者工具</el-dropdown-item>
                        <el-dropdown-item command="exit">退出</el-dropdown-item>
                    </el-dropdown-menu>
                </el-dropdown>
            </div>

            <el-drawer title="title" :visible.sync="drawer" direction="ltr" :before-close="handleClose">
                <span>测试数据</span>
            </el-drawer>
        </div>
        <div style="clear: both;"></div>
        <div class="mg10">
            <div>
                <div style="margin: 20px auto;">
                    <!--<h2>菜单操作数据：<el-tag type="info">{{text}}</el-tag></h2>-->
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

            <el-dialog :title="'应用信息'" :visible.sync="appInfoShow" width="350px">
                <div style="padding: 5px 20px;">
                    <div style="margin: 10px 10px; margin-top: 0px;">名称   {{appInfo.appName}}</div>
                    <div style="margin: 10px 10px">版本   {{appInfo.appVersion}}</div>
                </div>
            </el-dialog>

            <div class="mg10" style="margin-bottom: 50px;">
                <el-button @click="refEchartsData">刷新</el-button>
                <!--echarts bar图表-->
                <div id="echartsId" style="width: 900px;height:400px;" v-show="isShowEcharts"></div>
            </div>

            <!--<div class="mg10">
                <el-button @click="refD3Line">刷新</el-button>
            </div>
            <div id="d3Div">
                <svg width="1100" height="400"></svg>
            </div>-->
        </div>
    </div>
</template>

<script>
    const fs = require('fs')
    const os = require('os')
    const { remote , shell, ipcRenderer} = require('electron')
    const { Menu, MenuItem, dialog, BrowserWindow } = remote
    import index from '../vue/assets/index.js'

    export default {
        name: 'app',
        data() {
            return {
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
                appInfoShow:false,//是否显示app信息
                appInfo:{},//app信息
                //新建文件form
                form:{
                    fileName:'', //文件名
                    filePath:'',//文件夹路径
                },
                isShowEcharts:true,//是否显示echarts

                tags: ['tag','botton1','botton2','botton3','botton4'],
                drawer: false,
                // echart实例
                myChart:{},
                updateChart:'',
                refCount:0,
            }
        },
        mounted(){
            //this.getD3Line();
            this.echartsBar();
        },
        created() {
            // 监听主线程操作事件
            this.menuOption();
            // 注册右键菜单
            this.windowMenu();
        },
        methods: {
            getD3Line(){
                let data = [];
                let count = 1000;
                for (let i = 0; i < count; i++) {
                    let info = {date: (i/100) ,price:Math.random()*200};
                    data.push(info);
                }
                this.updateChart = index.getD3LineChart('d3Div', data);
            },
            refD3Line(){
                let data = [];
                this.refCount = this.refCount + 2000
                let count = this.refCount;
                console.log('数据个数：', count)
                for (let i = 0; i < count; i++) {
                    let info = {date:i, price:Math.random()*200};
                    data.push(info);
                }
                this.updateChart.updateD3ChartDate(data);
            },
            /*刷新echarts 渲染数据*/
            refEchartsData: function(){
                this.refCount = this.refCount + 100;
                //var dataCount = 5000;
                var data = index.echartsData(this.refCount);
                var option = {
                    xAxis: {
                        data: data.categoryData,
                        silent: false,
                        splitLine: {
                            show: false
                        },
                        splitArea: {
                            show: false
                        }
                    },
                    series: [{
                        type: 'bar',
                        data: data.valueData,
                        large: true,
                        largeThreshold:50
                    }]
                };
                this.myChart.setOption(option);
            },
            /*初始化echarts bar*/
            echartsBar: function(){
                var dataCount = this.refCount = 3600;
                var data = index.echartsData(dataCount);

                this.myChart = this.$echarts.init(document.getElementById('echartsId'));

                var option = index.echartsOptions(data);
                this.myChart.setOption(option);

                setInterval(() => {
                    this.refEchartsData();
                }, 30000);
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
                ipcRenderer.on('add-directory', (event, arg) => {
                    console.log('新建文件夹：' + arg);
                    this.isAddFile = false;
                    this.addFileDialogFormVisible = true;
                })
                ipcRenderer.on('select-file-path', (event, arg) => {
                    console.log('选择的文件：' + arg)
                })
                ipcRenderer.on('select-directory-path', (event, paths) => {
                    console.log('选择文件夹路径：' + paths);
                    if (paths) {
                        this.form.filePath = paths[0];
                    }
                })
                ipcRenderer.on('show-echarts', (event, arg) => {
                    console.log('显示echarts：' + arg)
                    this.isShowEcharts = true;
                })
                ipcRenderer.on('app-info', (event, args) => {
                    console.log(args)
                    this.appInfo = args;
                    this.appInfoShow = true;
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
                if (command == 'exit') {
                    console.log("退出")
                    ipcRenderer.send('app-exit', 'exit');
                } else if (command == 'reload') {
                    ipcRenderer.send('app-reload', 'reload');
                } else if (command == 'toggleDevTools') {
                    ipcRenderer.send('app-toggleDevTools', 'toggleDevTools');
                } else if (command == 'about') {
                    ipcRenderer.send('app-about', 'about');
                }
            },
            /*注册右键菜单*/
            windowMenu(){
                const menu = new Menu()
                menu.append(new MenuItem({ label: '重新加载', click() { ipcRenderer.send('app-reload', 'reload');}}))
                menu.append(new MenuItem({ label: '检查元素', click() {ipcRenderer.send('app-toggleDevTools', 'toggleDevTools');}}))

                window.addEventListener('contextmenu', (e) => {
                    e.preventDefault()
                    menu.popup({ window: remote.getCurrentWindow() })
                }, false)
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
        computed:{

        }
    }
</script>
<style>


</style>
