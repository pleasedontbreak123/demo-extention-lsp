import * as vscode from 'vscode';
import * as path from 'path';
import { LanguageClient, LanguageClientOptions, ServerOptions, TransportKind } from 'vscode-languageclient/node';

let client: LanguageClient;

export function activate(context: vscode.ExtensionContext) {
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
    
    let serverOptions: ServerOptions = {
        run: { 
            command: serverModule, 
            transport: TransportKind.stdio,
            options: { env: serverEnvironment }
        },
        debug: { 
            command: serverModule, 
            transport: TransportKind.stdio,
            options: { env: serverEnvironment }
        }
    };

    // 配置客户端，支持的文件类型
    let clientOptions: LanguageClientOptions = {
        documentSelector: [{ scheme: 'file', language: 'plaintext' }]  // 匹配.txt文件
    };

    // 创建语言客户端
    client = new LanguageClient(
        'myLanguageServer',  // 语言客户端名称
        'My LSP Server',  // 显示名称
        serverOptions,
        clientOptions
    );

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
async function clientStop(clientStartPromise: Promise<void>) {
    try {
        await clientStartPromise; // 等待客户端启动
        await client.stop(); // 停止客户端
    } catch (error) {
        console.error('Error stopping the client:', error);
    }
}

export function deactivate(): Thenable<void> | undefined {
    if (!client) {
        return undefined;
    }
    return client.stop();  // 停止 LSP 客户端
}