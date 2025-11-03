package com.scar.server.Dto;

public class ListenRequest {
    private String receiverFp;

    // Constructors
    public ListenRequest() {
    }

    public ListenRequest(String receiverFp) {
        this.receiverFp = receiverFp;
    }

    // Getters
    public String getReceiverFp() {
        return receiverFp;
    }

    // Setters
    public void setReceiverFp(String receiverFp) {
        this.receiverFp = receiverFp;
    }
}
