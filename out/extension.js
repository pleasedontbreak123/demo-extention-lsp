"use strict";
var __createBinding = (this && this.__createBinding) || (Object.create ? (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    var desc = Object.getOwnPropertyDescriptor(m, k);
    if (!desc || ("get" in desc ? !m.__esModule : desc.writable || desc.configurable)) {
      desc = { enumerable: true, get: function() { return m[k]; } };
    }
    Object.defineProperty(o, k2, desc);
}) : (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    o[k2] = m[k];
}));
var __setModuleDefault = (this && this.__setModuleDefault) || (Object.create ? (function(o, v) {
    Object.defineProperty(o, "default", { enumerable: true, value: v });
}) : function(o, v) {
    o["default"] = v;
});
var __importStar = (this && this.__importStar) || (function () {
    var ownKeys = function(o) {
        ownKeys = Object.getOwnPropertyNames || function (o) {
            var ar = [];
            for (var k in o) if (Object.prototype.hasOwnProperty.call(o, k)) ar[ar.length] = k;
            return ar;
        };
        return ownKeys(o);
    };
    return function (mod) {
        if (mod && mod.__esModule) return mod;
        var result = {};
        if (mod != null) for (var k = ownKeys(mod), i = 0; i < k.length; i++) if (k[i] !== "default") __createBinding(result, mod, k[i]);
        __setModuleDefault(result, mod);
        return result;
    };
})();
Object.defineProperty(exports, "__esModule", { value: true });
exports.activate = activate;
exports.deactivate = deactivate;
const path = __importStar(require("path"));
const node_1 = require("vscode-languageclient/node");
let client;
function activate(context) {
    // 在控制台输出激活信息
    console.log('扩展 "demo-ext" 已激活!');
    // 指定 LSP 服务器可执行文件的路径（修改为Windows下的.exe文件）
    let serverModule = path.join(context.extensionPath, 'lsp-server-demo', 'target', 'release', 'lsp-server-demo.exe');
    // 显示服务器路径以便调试
    console.log('LSP服务器路径:', serverModule);
    // 设置 LSP 服务器的环境变量
    const serverEnvironment = {
        RUST_LOG: 'info'
    };
    let serverOptions = {
        run: {
            command: serverModule,
            transport: node_1.TransportKind.stdio,
            options: { env: serverEnvironment }
        },
        debug: {
            command: serverModule,
            transport: node_1.TransportKind.stdio,
            options: { env: serverEnvironment }
        }
    };
    // 配置客户端，支持的文件类型
    let clientOptions = {
        documentSelector: [{ scheme: 'file', language: 'plaintext' }] // 匹配.txt文件
    };
    // 创建语言客户端
    client = new node_1.LanguageClient('myLanguageServer', // 语言客户端名称
    'My LSP Server', // 显示名称
    serverOptions, clientOptions);
    // 启动 LSP 客户端并确保它返回一个可以 "dispose" 的对象
    const clientStartPromise = client.start();
    // 因为 client.start() 返回的是 Promise<void>，所以需要确保它被正确处理
    context.subscriptions.push({
        dispose() {
            // 停止客户端
            clientStop(clientStartPromise);
        }
    });
}
// 客户端停止函数
async function clientStop(clientStartPromise) {
    try {
        await clientStartPromise; // 等待客户端启动
        await client.stop(); // 停止客户端
    }
    catch (error) {
        console.error('Error stopping the client:', error);
    }
}
function deactivate() {
    if (!client) {
        return undefined;
    }
    return client.stop(); // 停止 LSP 客户端
}
//# sourceMappingURL=extension.js.map