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
const vscode = __importStar(require("vscode"));
const path = __importStar(require("path"));
const node_1 = require("vscode-languageclient/node");
const child_process_1 = require("child_process");
let client;
let outputChannel;
let traceChannel;
function activate(context) {
    // 创建普通日志通道
    outputChannel = vscode.window.createOutputChannel('demo-ext');
    outputChannel.appendLine('扩展 "demo-ext" 已激活!');
    // 创建LSP协议通讯专用通道
    traceChannel = vscode.window.createOutputChannel('demo-ext-trace');
    // 指定 LSP 服务器可执行文件的路径（修改为Windows下的.exe文件）
    let serverModule = path.join(context.extensionPath, 'lsp-server-demo', 'target', 'release', 'lsp-server-demo.exe');
    // 显示服务器路径以便调试
    outputChannel.appendLine('LSP服务器路径: ' + serverModule);
    // 设置 LSP 服务器的环境变量
    const serverEnvironment = {
        RUST_LOG: 'debug'
    };
    let serverOptions = () => {
        // 启动 LSP 服务器进程
        const child = (0, child_process_1.spawn)(serverModule, [], {
            env: serverEnvironment,
            stdio: ['pipe', 'pipe', 'pipe']
        });
        let jsonBuffer = ''; // 用于存储不完整的 JSON 消息
        let bracketCount = 0; // 用于跟踪方括号
        let braceCount = 0; // 用于跟踪花括号
        // 检查JSON是否完整的辅助函数
        function isJsonComplete(text) {
            let tempBracketCount = 0;
            let tempBraceCount = 0;
            for (const char of text) {
                if (char === '[') {
                    tempBracketCount++;
                }
                if (char === ']') {
                    tempBracketCount--;
                }
                if (char === '{') {
                    tempBraceCount++;
                }
                if (char === '}') {
                    tempBraceCount--;
                }
            }
            return tempBracketCount === 0 && tempBraceCount === 0;
        }
        // 处理日志行的辅助函数
        function processLogLine(line) {
            const trimmed = line.trim();
            if (trimmed.startsWith('[') && trimmed.includes('INFO  lsp_server_demo')) {
                // 检查是否包含JSON内容的标志
                const hasJson = trimmed.includes('发送响应:') ||
                    trimmed.includes('收到初始化请求:') ||
                    trimmed.includes('{') ||
                    trimmed.includes('[');
                if (hasJson) {
                    // 如果是新的JSON消息开始，清空缓冲区并开始新的收集
                    jsonBuffer = trimmed;
                }
                else if (jsonBuffer && !trimmed.startsWith('[')) {
                    // 如果缓冲区不为空且当前行不是新的日志，则追加到缓冲区
                    jsonBuffer += ' ' + trimmed;
                }
                else if (!jsonBuffer) {
                    // 如果是普通日志且缓冲区为空，直接输出
                    traceChannel.appendLine(trimmed);
                }
                // 检查缓冲区中的消息是否完整
                if (jsonBuffer && isJsonComplete(jsonBuffer)) {
                    traceChannel.appendLine(jsonBuffer);
                    jsonBuffer = ''; // 清空缓冲区
                }
            }
            else if (jsonBuffer && !trimmed.startsWith('[')) {
                // 处理多行JSON内容
                jsonBuffer += ' ' + trimmed;
                // 每次追加后检查是否完整
                if (isJsonComplete(jsonBuffer)) {
                    traceChannel.appendLine(jsonBuffer);
                    jsonBuffer = '';
                }
            }
        }
        //捕获 stdout，只输出 LSP 协议 JSON 包
        child.stdout.on('data', (data) => {
            data.toString().split('\n').forEach((line) => {
                const trimmed = line.trim();
                if (trimmed.startsWith('{')) {
                    outputChannel.appendLine('[lsp 协议内容]' + trimmed);
                }
            });
        });
        // 捕获 stderr
        child.stderr.on('data', (data) => {
            const logLines = data.toString().split('\n');
            logLines.forEach(processLogLine);
        });
        // 捕获进程关闭
        child.on('close', (code) => {
            outputChannel.appendLine('[LSP] 进程退出，代码: ' + code);
        });
        return Promise.resolve({
            reader: child.stdout,
            writer: child.stdin
        });
    };
    // 配置客户端，支持的文件类型
    let clientOptions = {
        // 为了调试方便保留两个文件支持
        documentSelector: [
            { scheme: 'file', language: 'plaintext' },
            { scheme: 'file', language: 'cir' }
        ],
        traceOutputChannel: traceChannel, // LSP协议通讯内容专用通道
        middleware: {
            sendRequest: async (type, params, token, next) => {
                //outputChannel.appendLine('[lsp 协议内容stdin]' + JSON.stringify({ type, params }));
                if (typeof next === 'function') {
                    return next(type, params, token);
                }
            },
            sendNotification: (type, params, next) => {
                //outputChannel.appendLine('[lsp 协议内容stdin]' + JSON.stringify({ type, params }));
                if (typeof next === 'function') {
                    return next(type, params);
                }
            }
        }
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
            clientStop(clientStartPromise);
        }
    });
    context.subscriptions.push(outputChannel); // 确保通道被释放
    context.subscriptions.push(traceChannel); // 确保trace通道被释放
    traceChannel.appendLine('traceChannel is ready');
    outputChannel.appendLine('LSP Client started');
}
// 客户端停止函数
async function clientStop(clientStartPromise) {
    try {
        await clientStartPromise; // 等待客户端启动
        await client.stop(); // 停止客户端
        outputChannel.appendLine('LSP 客户端已停止');
    }
    catch (error) {
        outputChannel.appendLine('Error stopping the client: ' + error);
    }
}
function deactivate() {
    if (!client) {
        return undefined;
    }
    outputChannel.appendLine('扩展已停用');
    return client.stop(); // 停止 LSP 客户端
}
//# sourceMappingURL=extension.js.map