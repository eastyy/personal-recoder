// IPCClient.swift - Personal Log AI 文本捕获 IPC 客户端
// 通过 Unix Domain Socket 将提交的文本发送给 Personal Log AI 主应用
// 植入 Squirrel 输入法，在 commit(string:) 调用时同步发送

import Foundation

// MARK: - IPC 消息结构

enum LogTextType: String, Codable {
    case direct       // 直接输入（英文、数字、符号）
    case committed    // 已提交（最终汉字）
}

struct LogIPCMessage: Codable {
    let type: LogTextType
    let text: String
    let timestamp: TimeInterval
    let appBundleId: String?
    let appName: String?

    enum CodingKeys: String, CodingKey {
        case type, text, timestamp
        case appBundleId
        case appName
    }
}

// MARK: - IPC 客户端

/// 轻量级 IPC 客户端，通过 Unix Domain Socket 发送文本到 Personal Log AI
/// 设计为即发即忘（fire-and-forget），不阻塞输入法主线程
/// 断线时静默丢弃消息，不影响输入体验
final class LogIPCClient {

    static let shared = LogIPCClient()

    private let socketPath = "/tmp/personal-log-ai-ime.sock"
    private let recordSeparator: UInt8 = 0x1E
    private var socketFd: Int32 = -1
    private let sendQueue = DispatchQueue(label: "com.personallog.ipc", qos: .utility)
    private var lastConnectAttempt: TimeInterval = 0
    private let reconnectInterval: TimeInterval = 5.0

    private init() {}

    // MARK: - 公开接口

    /// 发送已提交文本（中文/英文/符号）
    func sendCommittedText(_ text: String, appBundleId: String?, appName: String?) {
        guard !text.isEmpty else { return }
        send(LogIPCMessage(
            type: .committed,
            text: text,
            timestamp: Date().timeIntervalSince1970,
            appBundleId: appBundleId,
            appName: appName
        ))
    }

    /// 发送直接输入文本（英文、数字等不经 IME 转换的字符）
    func sendDirectText(_ text: String, appBundleId: String?, appName: String?) {
        guard !text.isEmpty else { return }
        send(LogIPCMessage(
            type: .direct,
            text: text,
            timestamp: Date().timeIntervalSince1970,
            appBundleId: appBundleId,
            appName: appName
        ))
    }

    // MARK: - 内部实现

    private func send(_ message: LogIPCMessage) {
        sendQueue.async { [weak self] in
            guard let self = self else { return }

            // 序列化
            guard let jsonData = try? JSONEncoder().encode(message) else { return }
            var payload = jsonData
            payload.append(self.recordSeparator)

            // 尝试发送，如果未连接则尝试连接
            if self.socketFd < 0 {
                self.tryConnect()
            }

            guard self.socketFd >= 0 else { return }

            let result = payload.withUnsafeBytes { buffer -> Int in
                guard let base = buffer.baseAddress else { return -1 }
                return Darwin.write(self.socketFd, base, buffer.count)
            }

            if result < 0 {
                // 发送失败，关闭并等待下次重连
                Darwin.close(self.socketFd)
                self.socketFd = -1
            }
        }
    }

    private func tryConnect() {
        let now = Date().timeIntervalSince1970
        guard now - lastConnectAttempt > reconnectInterval else { return }
        lastConnectAttempt = now

        socketFd = Darwin.socket(AF_UNIX, SOCK_STREAM, 0)
        guard socketFd >= 0 else { return }

        var addr = sockaddr_un()
        addr.sun_family = sa_family_t(AF_UNIX)

        let pathBytes = socketPath.utf8
        let maxLen = MemoryLayout.size(ofValue: addr.sun_path) - 1
        guard pathBytes.count <= maxLen else {
            Darwin.close(socketFd)
            socketFd = -1
            return
        }

        _ = socketPath.withCString { cStr in
            strncpy(&addr.sun_path.0, cStr, maxLen)
        }

        let addrLen = socklen_t(MemoryLayout<UInt8>.size + MemoryLayout<sa_family_t>.size + pathBytes.count + 1)

        let result = withUnsafePointer(to: &addr) { ptr in
            ptr.withMemoryRebound(to: sockaddr.self, capacity: 1) { sockPtr in
                Darwin.connect(socketFd, sockPtr, addrLen)
            }
        }

        if result < 0 {
            Darwin.close(socketFd)
            socketFd = -1
        }
    }
}
