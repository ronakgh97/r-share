package com.scar.server.Dto;

/**
 * Rich status information for dashboard
 */
public class Status {
    // Server Information
    private String timestamp;
    private String serverVersion;
    private long uptimeSeconds;

    // Bandwidth Statistics
    private double totalBandwidthGB;
    private double totalBandwidthMB;

    // Session Statistics
    private int activeSessions;
    private int pendingSessions;
    private long totalSessionsCompleted;
    private long totalSessionsFailed;

    // Performance Metrics
    private double averageTransferSpeedMBps;
    private long currentTransferCount;
    private double peakBandwidthMBps;

    // Resource Usage
    private double memoryUsedMB;
    private double memoryMaxMB;
    private double cpuUsagePercent;
    private int threadCount;

    public Status() {
    }

    // Builder pattern constructor for easier creation
    public static class Builder {
        private final Status status = new Status();

        public Builder timestamp(String timestamp) {
            status.timestamp = timestamp;
            return this;
        }

        public Builder serverVersion(String version) {
            status.serverVersion = version;
            return this;
        }

        public Builder uptimeSeconds(long uptime) {
            status.uptimeSeconds = uptime;
            return this;
        }

        public Builder totalBandwidthGB(double gb) {
            status.totalBandwidthGB = gb;
            return this;
        }

        public Builder totalBandwidthMB(double mb) {
            status.totalBandwidthMB = mb;
            return this;
        }

        public Builder activeSessions(int count) {
            status.activeSessions = count;
            return this;
        }

        public Builder pendingSessions(int count) {
            status.pendingSessions = count;
            return this;
        }

        public Builder totalSessionsCompleted(long count) {
            status.totalSessionsCompleted = count;
            return this;
        }

        public Builder totalSessionsFailed(long count) {
            status.totalSessionsFailed = count;
            return this;
        }

        public Builder averageTransferSpeedMBps(double speed) {
            status.averageTransferSpeedMBps = speed;
            return this;
        }

        public Builder currentTransferCount(long count) {
            status.currentTransferCount = count;
            return this;
        }

        public Builder peakBandwidthMBps(double peak) {
            status.peakBandwidthMBps = peak;
            return this;
        }

        public Builder memoryUsedMB(double used) {
            status.memoryUsedMB = used;
            return this;
        }

        public Builder memoryMaxMB(double max) {
            status.memoryMaxMB = max;
            return this;
        }

        public Builder cpuUsagePercent(double cpu) {
            status.cpuUsagePercent = cpu;
            return this;
        }

        public Builder threadCount(int count) {
            status.threadCount = count;
            return this;
        }

        public Status build() {
            return status;
        }
    }

    // Getters and Setters
    public String getTimestamp() {
        return timestamp;
    }

    public void setTimestamp(String timestamp) {
        this.timestamp = timestamp;
    }

    public String getServerVersion() {
        return serverVersion;
    }

    public void setServerVersion(String serverVersion) {
        this.serverVersion = serverVersion;
    }

    public long getUptimeSeconds() {
        return uptimeSeconds;
    }

    public void setUptimeSeconds(long uptimeSeconds) {
        this.uptimeSeconds = uptimeSeconds;
    }

    public double getTotalBandwidthGB() {
        return totalBandwidthGB;
    }

    public void setTotalBandwidthGB(double totalBandwidthGB) {
        this.totalBandwidthGB = totalBandwidthGB;
    }

    public double getTotalBandwidthMB() {
        return totalBandwidthMB;
    }

    public void setTotalBandwidthMB(double totalBandwidthMB) {
        this.totalBandwidthMB = totalBandwidthMB;
    }

    public int getActiveSessions() {
        return activeSessions;
    }

    public void setActiveSessions(int activeSessions) {
        this.activeSessions = activeSessions;
    }

    public int getPendingSessions() {
        return pendingSessions;
    }

    public void setPendingSessions(int pendingSessions) {
        this.pendingSessions = pendingSessions;
    }

    public long getTotalSessionsCompleted() {
        return totalSessionsCompleted;
    }

    public void setTotalSessionsCompleted(long totalSessionsCompleted) {
        this.totalSessionsCompleted = totalSessionsCompleted;
    }

    public long getTotalSessionsFailed() {
        return totalSessionsFailed;
    }

    public void setTotalSessionsFailed(long totalSessionsFailed) {
        this.totalSessionsFailed = totalSessionsFailed;
    }

    public double getAverageTransferSpeedMBps() {
        return averageTransferSpeedMBps;
    }

    public void setAverageTransferSpeedMBps(double averageTransferSpeedMBps) {
        this.averageTransferSpeedMBps = averageTransferSpeedMBps;
    }

    public long getCurrentTransferCount() {
        return currentTransferCount;
    }

    public void setCurrentTransferCount(long currentTransferCount) {
        this.currentTransferCount = currentTransferCount;
    }

    public double getPeakBandwidthMBps() {
        return peakBandwidthMBps;
    }

    public void setPeakBandwidthMBps(double peakBandwidthMBps) {
        this.peakBandwidthMBps = peakBandwidthMBps;
    }

    public double getMemoryUsedMB() {
        return memoryUsedMB;
    }

    public void setMemoryUsedMB(double memoryUsedMB) {
        this.memoryUsedMB = memoryUsedMB;
    }

    public double getMemoryMaxMB() {
        return memoryMaxMB;
    }

    public void setMemoryMaxMB(double memoryMaxMB) {
        this.memoryMaxMB = memoryMaxMB;
    }

    public double getCpuUsagePercent() {
        return cpuUsagePercent;
    }

    public void setCpuUsagePercent(double cpuUsagePercent) {
        this.cpuUsagePercent = cpuUsagePercent;
    }

    public int getThreadCount() {
        return threadCount;
    }

    public void setThreadCount(int threadCount) {
        this.threadCount = threadCount;
    }
}
