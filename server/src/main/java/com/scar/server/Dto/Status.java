package com.scar.server.Dto;

public class Status {
    private String timestamp;
    private long totalBandwidth;

    public Status() {
    }

    public Status(String timestamp, long totalBandwidth) {
        this.timestamp = timestamp;
        this.totalBandwidth = totalBandwidth;
    }

    public String getTimestamp() {
        return timestamp;
    }

    public void setTimestamp(String timestamp) {
        this.timestamp = timestamp;
    }

    public long getTotalBandwidth() {
        return totalBandwidth;
    }

    public void setTotalBandwidth(long totalBandwidth) {
        this.totalBandwidth = totalBandwidth;
    }
}
