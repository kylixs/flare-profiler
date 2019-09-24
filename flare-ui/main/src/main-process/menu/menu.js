const {app, ipcMain, shell, dialog, BrowserWindow, Menu} = require('electron')

const template = [
    {
        label: '自定义操作',
        submenu: [
            {
                label: '弹出提示框',
                click: (item, focusedWindow) => {
                    const option = {
                        type: "info",
                        title: "提示信息",
                        message: "这是一个提示信息",

                    }
                    dialog.showMessageBox(focusedWindow, option, () => {
                    });
                }
            },
            {
                label: '显示图表',
                click: (item, focusedWindow) => {
                    focusedWindow.webContents.send("show-echarts","show-echarts");
                }
            },
            {
                label: '文件',
                submenu: [

                    {
                        label:'新建文件',
                        click:(item, focusedWindow) => {
                            focusedWindow.webContents.send('add-file', 'add-file');
                        }
                    },
                    {
                        label:'新建文件夹',
                        click:(item, focusedWindow) => {
                            focusedWindow.webContents.send('add-directory', 'add-directory');
                        }
                    },
                    {
                        label:'选择文件',
                        click:(item, focusedWindow) => {
                            dialog.showOpenDialog({'properties':['openFile','multiSelections']},(filePaths => {
                                focusedWindow.webContents.send('open-file-path', filePaths);
                            }))
                        }
                    },
                    {
                        label:'选择文件夹',
                        click:(item, focusedWindow) => {
                            dialog.showOpenDialog({'properties':['openDirectory','multiSelections']},(filePaths => {
                                focusedWindow.webContents.send('open-file-directory-path', filePaths);
                            }))
                        }
                    },
                ]
            }
        ]
    },
    {
        label: '操作',
        submenu:[
            {
                label: '重新加载',
                accelerator:(() => {
                    if (process.platform === 'darwin') {
                        return 'Ctrl+Command+F'
                    } else {
                        return 'F5'
                    }
                })(),
                click: (item, focusedWindow) => {
                    focusedWindow.reload();
                }
            },
            {
                label: '切换开发者工具',
                accelerator: 'F12',
                click:(item, focusedWindow) => {
                    if (focusedWindow) {
                        focusedWindow.toggleDevTools();
                    }
                }
            },
            {
                label: '退出',
                accelerator: 'Control + Q',
                click: (item, focusedWindow) => {
                    focusedWindow.close();
                }
            }
        ]
    }
];

app.on('ready', () => {
    const menu = Menu.buildFromTemplate(template)
    Menu.setApplicationMenu(menu)
})