const {ipcMain, dialog} = require('electron')

function ipcMainOn(){
    ipcMain.on('open-directory', (event, args) => {
        dialog.showOpenDialog({'properties':['openDirectory']},(filePaths => {
            event.sender.send('select-directory-path', filePaths);
        }))
    })
}

ipcMainOn();