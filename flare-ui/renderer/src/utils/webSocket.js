import Vue from 'vue'
let webSocket, webSocketUrl = 'ws://localhost:3344'

export default {
    webSocketInit:function(){
        if ('WebSocket' in window) {
            webSocket = new WebSocket(webSocketUrl, "flare-profiler");
            webSocket.onopen = this.webSocketOnOpen();
            webSocket.onclose = this.webSocketOnClose;
            webSocket.onerror = this.webSocketOnError;
            //webSocket.onmessage = this.webSocketOnMessage;

            Vue.prototype.$ws = webSocket;
        } else {
            console.log("浏览器不支持websocket");
        }
    },
    webSocketOnOpen:function(){
        console.log("websocket建立连接");
    },
    webSocketOnClose:function(){
        console.log("websocket销毁连接");
    },
    webSocketOnError:function(){
        console.log("websocket连接失败");
    },
    webSocketOnMessage:function(msg){
        console.log("websocket接收到信息：" + msg.data);
        return msg;
    },
    webSocketSendMessage:function(msg){
        console.log("websocket发送消息：" + msg);
        webSocket.send(msg);
    },
}