package com.scar.server.Dto;

public class ServeResponse {
    private String status;
    private String sessionId;
    private int socketPort;
    private String message;
    private long expiresIn;

    // Constructors
    public ServeResponse() {
    }

    public ServeResponse(String status, String sessionId, int socketPort,
                         String message, long expiresIn) {
        this.status = status;
        this.sessionId = sessionId;
        this.socketPort = socketPort;
        this.message = message;
        this.expiresIn = expiresIn;
    }

    // Getters
    public String getStatus() {
        return status;
    }

    public String getSessionId() {
        return sessionId;
    }

    public int getSocketPort() {
        return socketPort;
    }

    public String getMessage() {
        return message;
    }

    public long getExpiresIn() {
        return expiresIn;
    }

    // Setters
    public void setStatus(String status) {
        this.status = status;
    }

    public void setSessionId(String sessionId) {
        this.sessionId = sessionId;
    }

    public void setSocketPort(int socketPort) {
        this.socketPort = socketPort;
    }

    public void setMessage(String message) {
        this.message = message;
    }

    public void setExpiresIn(long expiresIn) {
        this.expiresIn = expiresIn;
    }
}
