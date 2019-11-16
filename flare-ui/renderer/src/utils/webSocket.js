import Vue from 'vue'
let webSocket, webSocketUrl = 'ws://localhost:3344'

function parse_host(url){
    var s = url.indexOf('://');
    var p = url.indexOf('/', s+3);
    var addr = url.substring(s+3, p);

    var p2 = addr.indexOf(':');
    if (p2 != -1){
        return addr.substring(0, p2);
    }
    return addr;
}

export default {
    webSocketInit:function(){
        webSocketUrl = 'ws://'+parse_host(window.location.href)+":3344";
        if ('WebSocket' in window) {
            webSocket = new WebSocket(webSocketUrl, "flare-profiler");
            webSocket.onopen = this.webSocketOnOpen;
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
    webSocketOnError:function(e){
        console.error("websocket连接失败:", e);
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