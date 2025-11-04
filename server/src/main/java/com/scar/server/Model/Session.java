package com.scar.server.Model;

public class Session {
    private String sessionId;
    private String senderFp;
    private String receiverFp;
    private String filename;
    private long fileSize;
    private String signature;
    private String fileHash;
    private String status;  // "waiting_receiver", "waiting_sender", "matched", "timeout"
    private long createdAt;
    private long expiresAt;
    private static final int SOCKET_PORT = 10000;

    public Session() {
    }

    // Constructor
    public Session(String sessionId, String senderFp, String receiverFp,
                   String filename, long fileSize, String signature, String fileHash) {
        this.sessionId = sessionId;
        this.senderFp = senderFp;
        this.receiverFp = receiverFp;
        this.filename = filename;
        this.fileSize = fileSize;
        this.signature = signature;
        this.fileHash = fileHash;
        this.createdAt = System.currentTimeMillis();
        this.expiresAt = this.createdAt + 120_000;  // 2 mins
        this.status = "waiting_receiver";
    }

    // Getters
    public String getSessionId() {
        return sessionId;
    }

    public String getSenderFp() {
        return senderFp;
    }

    public String getReceiverFp() {
        return receiverFp;
    }

    public String getFilename() {
        return filename;
    }

    public long getFileSize() {
        return fileSize;
    }

    public String getSignature() {
        return signature;
    }

    public String getFileHash() {
        return fileHash;
    }

    public String getStatus() {
        return status;
    }

    public void setStatus(String status) {
        this.status = status;
    }

    public long getExpiresAt() {
        return expiresAt;
    }

    public int getSocketPort() {
        return SOCKET_PORT;
    }

    public boolean isExpired() {
        return System.currentTimeMillis() > expiresAt;
    }
}
