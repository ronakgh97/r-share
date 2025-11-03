package com.scar.server.Dto;

public class ListenResponse {
    private String status;
    private String sessionId;
    private String senderFp;
    private String filename;
    private long fileSize;
    private String signature;
    private int socketPort;
    private String message;

    // Constructors
    public ListenResponse() {
    }

    public ListenResponse(String status, String sessionId, String senderFp,
                          String filename, long fileSize, String signature,
                          int socketPort, String message) {
        this.status = status;
        this.sessionId = sessionId;
        this.senderFp = senderFp;
        this.filename = filename;
        this.fileSize = fileSize;
        this.signature = signature;
        this.socketPort = socketPort;
        this.message = message;
    }

    // Getters
    public String getStatus() {
        return status;
    }

    public String getSessionId() {
        return sessionId;
    }

    public String getSenderFp() {
        return senderFp;
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

    public int getSocketPort() {
        return socketPort;
    }

    public String getMessage() {
        return message;
    }

    // Setters
    public void setStatus(String status) {
        this.status = status;
    }

    public void setSessionId(String sessionId) {
        this.sessionId = sessionId;
    }

    public void setSenderFp(String senderFp) {
        this.senderFp = senderFp;
    }

    public void setFilename(String filename) {
        this.filename = filename;
    }

    public void setFileSize(long fileSize) {
        this.fileSize = fileSize;
    }

    public void setSignature(String signature) {
        this.signature = signature;
    }

    public void setSocketPort(int socketPort) {
        this.socketPort = socketPort;
    }

    public void setMessage(String message) {
        this.message = message;
    }
}
