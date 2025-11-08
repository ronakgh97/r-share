package com.scar.server.Controller;

import com.scar.server.Dto.*;
import com.scar.server.Service.SessionService;
import com.scar.server.Socket.SocketSessionRegistry;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Value;
import org.springframework.http.ResponseEntity;
import org.springframework.web.bind.annotation.*;
import org.springframework.web.context.request.async.DeferredResult;

import java.time.Instant;

@CrossOrigin(origins = "*")
@RestController
@RequestMapping("/api/relay")
public class SessionController {

    @Value("${rshare.session.blocking-timeout-ms:30000}")
    private long BLOCKING_TIMEOUT;

    public static final String blue = "\u001B[34m"; // Blue
    public static final String red = "\u001B[31m"; // Red
    public static final String reset = "\u001B[0m"; // Reset
    public static final String yellow = "\u001B[33m"; // Yellow
    public static final String green = "\u001B[32m"; // Green
    public static final String cyan = "\u001B[36m"; // Cyan

    private static final Logger log = LoggerFactory.getLogger(SessionController.class);

    private final SessionService sessionService;
    private final SocketSessionRegistry socketSessionRegistry;

    @Autowired
    public SessionController(SessionService sessionService, SocketSessionRegistry socketSessionRegistry) {
        this.sessionService = sessionService;
        this.socketSessionRegistry = socketSessionRegistry;
    }

    /**
     * Sender calls: POST /api/relay/serve
     * BLOCKS until Receiver accepts or 30 second timeout
     */
    @PostMapping("/serve")
    public DeferredResult<ResponseEntity<ServeResponse>> initiate(
            @RequestBody ServeRequest request) {

        if (request.getSenderFp() != null && request.getReceiverFp() != null) {
            log.info("Session request from sender {}{}{} to receiver {}{}{}",
                    blue, request.getSenderFp().substring(0, 8), reset,
                    red, request.getReceiverFp().substring(0, 8), reset

            );
        }

        DeferredResult<ResponseEntity<ServeResponse>> result = new DeferredResult<>(BLOCKING_TIMEOUT); // 30 second
        // timeout

        // Validate
        if (request.getSenderFp() == null || request.getSenderFp().isEmpty()) {
            result.setResult(ResponseEntity.badRequest().body(
                    new ServeResponse("error", null, 0, "Missing sender fingerprint", 0)));
            return result;
        }

        if (request.getReceiverFp() == null || request.getReceiverFp().isEmpty()) {
            result.setResult(ResponseEntity.badRequest().body(
                    new ServeResponse("error", null, 0, "Missing receiver fingerprint", 0)));
            return result;
        }

        if (request.getSignature() == null || request.getSignature().isEmpty()) {
            result.setResult(ResponseEntity.badRequest().body(
                    new ServeResponse("error", null, 0, "Missing signature", 0)));
            return result;
        }

        if (request.getFileHash() == null || request.getFileHash().isEmpty()) {
            result.setResult(ResponseEntity.badRequest().body(
                    new ServeResponse("error", null, 0, "Missing file hash", 0)));
            return result;
        }

        if (request.getFileSize() < 0) {
            result.setResult(ResponseEntity.badRequest().body(
                    new ServeResponse("error", null, 0, "Invalid file size", 0)));
            return result;
        }

        if (result.hasResult()) {
            return result;
        }

        // Call blocking service
        sessionService.initiateAndWait(
                request.getSenderFp(),
                request.getReceiverFp(),
                request.getFilename(),
                request.getFileSize(),
                request.getSignature(),
                request.getFileHash()).thenAccept(session -> {
                    long expiresIn = session.getExpiresAt() - System.currentTimeMillis();
                    result.setResult(ResponseEntity.ok(
                            new ServeResponse(
                                    "matched",
                                    session.getSessionId(),
                                    session.getSocketPort(),
                                    "Receiver accepted, Proceeding to socket transfer.",
                                    expiresIn)));
                }).exceptionally(ex -> {
                    // Timeout or error
                    log.error("Serve failed: {}", ex.getMessage());
                    result.setResult(ResponseEntity.status(408).body(
                            new ServeResponse("timeout", null, 0,
                                    "Receiver didn't respond: " + ex.getMessage(),
                                    0)));
                    return null;
                });

        return result;
    }

    /**
     * Receiver calls: POST /api/relay/listen
     * BLOCKS until Sender initiates or 30 second timeout
     */
    @PostMapping("/listen")
    public DeferredResult<ResponseEntity<ListenResponse>> listen(
            @RequestBody ListenRequest request) {

        if (request.getReceiverFp() != null) {
            log.info("Listen request from receiver {}{}{}",
                    red, request.getReceiverFp().substring(0, 8), reset);
        }

        DeferredResult<ResponseEntity<ListenResponse>> result = new DeferredResult<>(BLOCKING_TIMEOUT); // 30 second
        // timeout

        // Validate
        if (request.getReceiverFp() == null || request.getReceiverFp().isEmpty()) {
            result.setResult(ResponseEntity.badRequest().body(
                    new ListenResponse("error", null, null, null, 0, null, null, 0,
                            "Missing receiver fingerprint")));
            return result;
        }

        // Call blocking service
        sessionService.listenAndWait(request.getReceiverFp())
                .thenAccept(session -> {
                    // Alice initiated! Return to Bob
                    long expiresIn = session.getExpiresAt() - System.currentTimeMillis();
                    result.setResult(ResponseEntity.ok(
                            new ListenResponse(
                                    "matched",
                                    session.getSessionId(),
                                    session.getSenderFp(),
                                    session.getFilename(),
                                    session.getFileSize(),
                                    session.getSignature(),
                                    session.getFileHash(),
                                    session.getSocketPort(),
                                    "Incoming transfer from " + session
                                            .getSenderFp()
                                            .substring(0, 8))));
                }).exceptionally(ex -> {
                    log.error("Listen failed: {}", ex.getMessage());
                    result.setResult(ResponseEntity.status(408).body(
                            new ListenResponse("timeout", null, null, null, 0, null, null, 0,
                                    "No sender found: " + ex.getMessage())));
                    return null;
                });

        return result;
    }

    /**
     * Mark session as complete (cleanup)
     */
    @DeleteMapping("/session/{sessionId}")
    public ResponseEntity<String> completeSession(@PathVariable String sessionId) {
        sessionService.completeSession(sessionId);
        log.info("Session marked complete: {}", sessionId.substring(0, 8));
        return ResponseEntity.ok("Session completed");
    }

    /**
     * Get server stats for dashboard
     */
    @GetMapping("/status")
    public ResponseEntity<Status> getStatus() {
        // Bandwidth statistics
        long totalBytes = socketSessionRegistry.getTotalBytesTransferred();
        double totalGB = totalBytes / (1024.0 * 1024.0 * 1024.0);
        double totalMB = totalBytes / (1024.0 * 1024.0);

        // Memory statistics
        Runtime runtime = Runtime.getRuntime();
        double memoryUsedMB = (runtime.totalMemory() - runtime.freeMemory()) / (1024.0 * 1024.0);
        double memoryMaxMB = runtime.maxMemory() / (1024.0 * 1024.0);

        // Thread statistics
        int threadCount = Thread.activeCount();

        // CPU usage
        // TODO: use OperatingSystemMXBean
        double cpuUsage = 0.0;
        try {
            com.sun.management.OperatingSystemMXBean osBean = (com.sun.management.OperatingSystemMXBean) java.lang.management.ManagementFactory
                    .getOperatingSystemMXBean();
            cpuUsage = osBean.getCpuLoad() * 100.0;
        } catch (Exception e) {
            log.debug("Could not get CPU usage: {}", e.getMessage());
        }

        // Build status object
        Status status = new Status.Builder()
                .timestamp(Instant.now().toString())
                .serverVersion("0.1.0-BETA")
                .uptimeSeconds(socketSessionRegistry.getUptimeSeconds())
                .totalBandwidthGB(totalGB)
                .totalBandwidthMB(totalMB)
                .activeSessions(socketSessionRegistry.getActiveSessionCount())
                .pendingSessions(socketSessionRegistry.getPendingSessionCount())
                .totalSessionsCompleted(socketSessionRegistry.getTotalSessionsCompleted())
                .totalSessionsFailed(socketSessionRegistry.getTotalSessionsFailed())
                .averageTransferSpeedMBps(socketSessionRegistry.getAverageTransferSpeedMBps())
                .currentTransferCount(socketSessionRegistry.getActiveSessionCount())
                .peakBandwidthMBps(socketSessionRegistry.getPeakBandwidthMBps())
                .memoryUsedMB(memoryUsedMB)
                .memoryMaxMB(memoryMaxMB)
                .cpuUsagePercent(cpuUsage)
                .threadCount(threadCount)
                .build();

        log.info("Status request: {}{}{} GB transferred, {} active sessions",
                green, String.format("%.2f", totalGB), reset,
                socketSessionRegistry.getActiveSessionCount());

        return ResponseEntity.ok(status);
    }
}