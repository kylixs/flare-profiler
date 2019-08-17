const {ipcMain, dialog} = require('electron')

function ipcMainOn(){
    ipcMain.on('open-directory', (event, args) => {
        console.log('监听到渲染进程发送的消息：' + args);
        dialog.showOpenDialog({'properties':['openDirectory']},(filePaths => {
            event.sender.send('select-directory-path', filePaths);
        }))
    })
}

ipcMainOn();