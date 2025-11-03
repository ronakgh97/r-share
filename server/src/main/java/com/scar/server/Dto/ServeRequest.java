package com.scar.server.Dto;

public class ServeRequest {
    private String senderFp;
    private String receiverFp;
    private String filename;
    private long fileSize;
    private String signature;

    // Constructors
    public ServeRequest() {
    }

    public ServeRequest(String senderFp, String receiverFp, String filename,
                        long fileSize, String signature) {
        this.senderFp = senderFp;
        this.receiverFp = receiverFp;
        this.filename = filename;
        this.fileSize = fileSize;
        this.signature = signature;
    }

    // Getters
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

    // Setters
    public void setSenderFp(String senderFp) {
        this.senderFp = senderFp;
    }

    public void setReceiverFp(String receiverFp) {
        this.receiverFp = receiverFp;
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
}
