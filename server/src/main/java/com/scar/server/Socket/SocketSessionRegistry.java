package com.scar.server.Socket;

import com.scar.server.Model.Session;
import io.netty.channel.Channel;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.springframework.stereotype.Component;

import java.util.Map;
import java.util.concurrent.ConcurrentHashMap;

import static com.scar.server.Controller.SessionController.*;

/**
 * Registry to track active socket connections and pair them by session ID
 */
@Component
public class SocketSessionRegistry {
    private static final Logger log = LoggerFactory.getLogger(SocketSessionRegistry.class);

    // Map: sessionId -> PendingConnection (waiting for pair)
    private final Map<String, PendingConnection> pendingConnections = new ConcurrentHashMap<>();

    // Map: sessionId -> ActiveTransfer (both parties connected)
    private final Map<String, ActiveTransfer> activeTransfers = new ConcurrentHashMap<>();
    private long totalBytesTransferred = 0;

    /**
     * Register a new socket connection with session ID
     * Returns the PendingConnection if a partner was waiting (so we can notify them),
     * or null if this is the first to arrive
     */
    public synchronized PendingConnection registerConnection(String sessionId, Channel channel, String role,
            Session session, Object handler) {
        log.info("Socket connection: {} | role={}{}{} | session={}{}{}",
                channel.remoteAddress(), green, role, reset, yellow, sessionId.substring(0, 8), reset);

        // Check if partner already waiting
        PendingConnection pending = pendingConnections.get(sessionId);

        if (pending != null) {
            // Partner found! Create active transfer
            log.info("Pair matched! Session: {}", sessionId.substring(0, 8));

            Channel senderChannel = role.equals("sender") ? channel : pending.channel;
            Channel receiverChannel = role.equals("receiver") ? channel : pending.channel;

            ActiveTransfer transfer = new ActiveTransfer(
                    sessionId, senderChannel, receiverChannel, session);

            activeTransfers.put(sessionId, transfer);
            pendingConnections.remove(sessionId);
            return pending; // Return partner's pending connection so we can notify them
        } else {
            // First to arrive, wait for partner
            log.info("Waiting for partner... Session: {}", sessionId.substring(0, 8));
            pendingConnections.put(sessionId, new PendingConnection(sessionId, channel, role, session, handler));
            return null; // Still waiting
        }
    }

    public ActiveTransfer getActiveTransfer(String sessionId) {
        return activeTransfers.get(sessionId);
    }

    public void removeTransfer(String sessionId) {
        ActiveTransfer transfer = activeTransfers.remove(sessionId);
        if (transfer != null) {
            totalBytesTransferred += transfer.bytesTransferred;
        }
        pendingConnections.remove(sessionId);
        log.info("Removed session: {}", sessionId.substring(0, 8));
    }

    public void removeByChannel(Channel channel) {
        // Find and remove any pending/active transfer using this channel
        pendingConnections.entrySet().removeIf(entry -> {
            if (entry.getValue().channel == channel) {
                log.info("Removed pending connection: {}", entry.getKey().substring(0, 8));
                return true;
            }
            return false;
        });

        activeTransfers.entrySet().removeIf(entry -> {
            ActiveTransfer transfer = entry.getValue();
            if (transfer.senderChannel == channel || transfer.receiverChannel == channel) {
                log.info("Removed active transfer: {}", entry.getKey().substring(0, 8));
                totalBytesTransferred += transfer.bytesTransferred;
                // Close the other channel too
                if (transfer.senderChannel != channel && transfer.senderChannel.isActive()) {
                    transfer.senderChannel.close();
                }
                if (transfer.receiverChannel != channel && transfer.receiverChannel.isActive()) {
                    transfer.receiverChannel.close();
                }
                return true;
            }
            return false;
        });
    }

    // Helper classes
    public static class PendingConnection {
        public final String sessionId;
        public final Channel channel;
        public final String role; // "sender" or "receiver"
        public final Session session;
        public final Object handler; // FileTransferHandler instance

        public PendingConnection(String sessionId, Channel channel, String role, Session session, Object handler) {
            this.sessionId = sessionId;
            this.channel = channel;
            this.role = role;
            this.session = session;
            this.handler = handler;
        }
    }

    public static class ActiveTransfer {
        public final String sessionId;
        public final Channel senderChannel;
        public final Channel receiverChannel;
        public final Session session;
        public long bytesTransferred = 0;
        public volatile boolean senderAcked = false;
        public volatile boolean receiverAcked = false;
        public volatile Object senderHandler;
        public volatile Object receiverHandler;

        public ActiveTransfer(String sessionId, Channel senderChannel, Channel receiverChannel, Session session) {
            this.sessionId = sessionId;
            this.senderChannel = senderChannel;
            this.receiverChannel = receiverChannel;
            this.session = session;
        }

        public synchronized boolean setBothHandlers(String role, Object handler) {
            if ("sender".equals(role)) {
                senderHandler = handler;
            } else {
                receiverHandler = handler;
            }
            return senderHandler != null && receiverHandler != null;
        }

        public synchronized boolean markAcked(String role) {
            if ("sender".equals(role)) {
                senderAcked = true;
            } else {
                receiverAcked = true;
            }
            return senderAcked && receiverAcked;
        }
    }

    /**
     * Mark that a client sent ACK, and check if both have ACK'd
     * If both ACK'd, enable relay mode on both handlers
     */
    public synchronized void checkBothAcked(String sessionId) {
        ActiveTransfer transfer = activeTransfers.get(sessionId);
        if (transfer == null) {
            log.warn("No active transfer for session: {}", sessionId.substring(0, 8));
            return;
        }

        if (transfer.senderAcked && transfer.receiverAcked) {
            log.info("Both clients ACK'd! Enabling relay mode | Session: {}", sessionId.substring(0, 8));

            // Enable paired mode on both handlers
            if (transfer.senderHandler instanceof FileTransferHandler) {
                ((FileTransferHandler) transfer.senderHandler).setPaired(true);
            }
            if (transfer.receiverHandler instanceof FileTransferHandler) {
                ((FileTransferHandler) transfer.receiverHandler).setPaired(true);
            }
        } else {
            log.info("Waiting for both ACKs | Session: {} | Sender: {}, Receiver: {}",
                    sessionId.substring(0, 8), transfer.senderAcked, transfer.receiverAcked);
        }
    }

    public long getTotalBytesTransferred() {
        long activeBytes = activeTransfers.values().stream()
                .mapToLong(transfer -> transfer.bytesTransferred)
                .sum();
        return totalBytesTransferred + activeBytes;
    }
}
