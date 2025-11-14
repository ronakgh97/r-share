package com.scar.server.Socket;

import com.scar.server.Model.Session;
import com.scar.server.Service.SessionService;
import io.netty.buffer.ByteBuf;
import io.netty.channel.Channel;
import io.netty.channel.ChannelHandlerContext;
import io.netty.channel.ChannelInboundHandlerAdapter;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.util.ArrayList;
import java.util.List;
import java.util.Optional;

import static com.scar.server.Controller.SessionController.*;

/**
 * Handles file transfer socket connections
 * Protocol:
 * 1. Client connects
 * 2. Client sends "session_id:role\n" (raw text)
 * 3. Server validates session, waits for partner
 * 4. When both connected, forward all raw binary data bidirectionally
 */
public class FileTransferHandler extends ChannelInboundHandlerAdapter {
    private static final Logger log = LoggerFactory.getLogger(FileTransferHandler.class);

    private final SocketSessionRegistry registry;
    private final SessionService sessionService;
    private String sessionId;
    private String role; // "sender" or "receiver"
    private volatile boolean paired = false;
    private volatile boolean readyAckReceived = false;
    private final List<BufferedMessage> bufferedData = new ArrayList<>();

    public FileTransferHandler(SocketSessionRegistry registry, SessionService sessionService) {
        this.registry = registry;
        this.sessionService = sessionService;
    }

    // Helper class to store buffered data with context
    private static class BufferedMessage {
        final ChannelHandlerContext ctx;
        final ByteBuf buf;

        BufferedMessage(ChannelHandlerContext ctx, ByteBuf buf) {
            this.ctx = ctx;
            this.buf = buf;
        }
    }

    @Override
    public void channelRead(ChannelHandlerContext ctx, Object msg) {
        if (!(msg instanceof ByteBuf buf)) {
            return;
        }

        // First message: session_id + role handshake
        if (sessionId == null) {
            handleHandshake(ctx, buf);
            return;
        }

        // If not ACK'd yet, check if this is the ACK message
        if (!readyAckReceived) {
            handleAckMessage(ctx, buf);
            return;
        }

        // After ACK: forward data to partner
        if (paired) {
            forwardData(ctx, buf);
        } else {
            // Buffer data until both clients ACK
            log.debug("Buffering {} bytes until paired | Session: {}{}{}",
                    buf.readableBytes(), yellow, sessionId.substring(0, 8), reset);
            bufferedData.add(new BufferedMessage(ctx, buf.retain()));
            buf.release();
        }
    }

    private void handleAckMessage(ChannelHandlerContext ctx, ByteBuf buf) {
        // Check if this is an ACK message
        int newlineIndex = buf.indexOf(buf.readerIndex(), buf.writerIndex(), (byte) '\n');

        if (newlineIndex == -1) {
            log.warn("Waiting for ACK, but no newline found");
            buf.release();
            return;
        }

        int ackLength = newlineIndex - buf.readerIndex() + 1;
        byte[] ackData = new byte[ackLength];
        buf.readBytes(ackData);
        String ackMessage = new String(ackData).trim();

        if ("ACK".equals(ackMessage)) {
            this.readyAckReceived = true;
            log.info("ACK received from {}{}{} | Session: {}{}{}", green, role, reset, yellow,
                    sessionId.substring(0, 8), reset);

            // Mark ACK in the transfer object
            SocketSessionRegistry.ActiveTransfer transfer = registry.getActiveTransfer(sessionId);
            if (transfer != null) {
                transfer.markAcked(role);
                registry.checkBothAcked(sessionId);
            }

            // If there's remaining data in buffer, it's file data - save it for after paired=true
            if (buf.isReadable()) {
                log.info("Data arrived with ACK, buffering {} bytes", buf.readableBytes());
                // Re-trigger channelRead with remaining data once paired
                ctx.fireChannelRead(buf.retain());
            }
            buf.release();
        } else {
            log.error("Expected ACK, got: {} | Session: {}{}{}", ackMessage, yellow, sessionId.substring(0, 8), reset);
            buf.release();
            ctx.close();
        }
    }

    private void handleHandshake(ChannelHandlerContext ctx, ByteBuf buf) {
        // Read only until newline (handshake message)
        int newlineIndex = buf.indexOf(buf.readerIndex(), buf.writerIndex(), (byte) '\n');

        if (newlineIndex == -1) {
            // No newline found yet, wait for more data
            log.warn("Handshake incomplete, waiting for newline...");
            return;
        }

        // Read handshake (everything up to and including newline)
        int handshakeLength = newlineIndex - buf.readerIndex() + 1;
        byte[] handshakeData = new byte[handshakeLength];
        buf.readBytes(handshakeData);
        String message = new String(handshakeData).trim(); // Trim newline and whitespace

        // Expected format: "session_id:role" where role is "sender" or "receiver"
        String[] parts = message.split(":", 2);
        if (parts.length < 2) {
            log.error("Invalid handshake format: {}", message);
            ctx.close();
            return;
        }

        this.sessionId = parts[0].trim();
        this.role = parts[1].trim();

        log.info("Handshake: session={}{}{}, role={}{}{}", yellow, sessionId.substring(0, 8), reset, green, role,
                green);

        // Validate session exists
        Optional<Session> sessionOpt = sessionService.getSession(sessionId);
        if (sessionOpt.isEmpty()) {
            log.error("Invalid session ID: {}{}{}", yellow, sessionId.substring(0, 8), reset);
            ctx.close();
            return;
        }

        Session session = sessionOpt.get();
        log.info("Session validated: {}{}{} | {}{}{} -> {}{}{}",
                yellow, sessionId.substring(0, 8), reset,
                blue, session.getSenderFp().substring(0, 8), reset,
                red, session.getReceiverFp().substring(0, 8), reset);

        // Register connection - returns partner's pending connection if pairing
        // complete
        SocketSessionRegistry.PendingConnection partner = registry.registerConnection(
                sessionId, ctx.channel(), role, session, this);

        if (partner != null) {
            log.info("Transfer ready! Session: {}{}{} | Both parties connected, sending READY signals",
                    yellow, sessionId.substring(0, 8), reset);

            // Get the active transfer and store both handlers
            SocketSessionRegistry.ActiveTransfer transfer = registry.getActiveTransfer(sessionId);
            if (transfer != null) {
                transfer.setBothHandlers(role, this);
                transfer.setBothHandlers(partner.role, partner.handler);
            }

            // Send READY signals - clients must ACK before relay starts
            try {
                ctx.channel().writeAndFlush(ctx.alloc().buffer(6).writeBytes("READY\n".getBytes())).sync();
                partner.channel.writeAndFlush(partner.channel.alloc().buffer(6).writeBytes("READY\n".getBytes()))
                        .sync();
            } catch (InterruptedException e) {
                log.error("Interrupted while sending READY signals", e);
                ctx.close();
                buf.release();
                return;
            }

            log.info("READY signals sent, waiting for ACKs | Session: {}{}{}", yellow, sessionId.substring(0, 8),
                    reset);

            // DON'T set paired=true here! Wait for both ACKs first
            // The checkBothAcked() method will set paired=true when both clients ACK

            buf.release();
        } else {
            log.info("Waiting for partner...");
            buf.release();
        }
    }

    private void forwardData(ChannelHandlerContext ctx, ByteBuf buf) {
        SocketSessionRegistry.ActiveTransfer transfer = registry.getActiveTransfer(sessionId);
        if (transfer == null) {
            log.error("No active transfer for session: {}{}{}", yellow, sessionId, reset);
            buf.release();
            return;
        }

        // Determine target channel (sender -> receiver, receiver -> sender)
        Channel target = (ctx.channel() == transfer.senderChannel)
                ? transfer.receiverChannel
                : transfer.senderChannel;

        if (!target.isActive()) {
            log.error("Partner channel not active");
            buf.release();
            ctx.close();
            return;
        }

        // Forward data immediately (low-latency streaming)
        int bytes = buf.readableBytes();
        transfer.bytesTransferred += bytes;

        ByteBuf copy = buf.retain();
        target.writeAndFlush(copy); // Flush immediately for responsive transfers

        // if (transfer.bytesTransferred % 1048576 == 0) { // Log every 1MB
        // log.info("Transferred: {} MB | Session: {}",
        // transfer.bytesTransferred / 1048576,
        // sessionId.substring(0, 8));
        // }

        // log.info("Total bytes: {}{}{} Transferred for session: {}{}{}",
        // cyan, transfer.bytesTransferred, reset,
        // yellow, sessionId.substring(0, 8), reset);

        buf.release();
    }

    @Override
    public void channelInactive(ChannelHandlerContext ctx) {
        if (sessionId != null) {
            log.info("Channel disconnected: {} | Session: {}{}{}",
                    ctx.channel().remoteAddress(),
                    yellow, sessionId.substring(0, 8), reset);

            SocketSessionRegistry.ActiveTransfer transfer = registry.getActiveTransfer(sessionId);
            if (transfer != null) {
                log.info("Transfer complete: ({}) gigabytes | Session: {}{}{}",
                        transfer.bytesTransferred / (1024 * 1024 * 1024),
                        yellow, sessionId.substring(0, 8), reset);
            }

            registry.removeByChannel(ctx.channel());
        }

        // Release any buffered data
        for (BufferedMessage msg : bufferedData) {
            msg.buf.release();
        }
        bufferedData.clear();
    }

    @Override
    public void exceptionCaught(ChannelHandlerContext ctx, Throwable cause) {
        log.warn("Socket warning: {}", cause.getMessage());

        // Release buffered data on error
        for (BufferedMessage msg : bufferedData) {
            msg.buf.release();
        }
        bufferedData.clear();

        ctx.close();
    }

    /**
     * Set the paired state and flush buffered data
     */
    public void setPaired(boolean paired) {
        this.paired = paired;

        // Flush any buffered data now that we're paired
        if (paired && !bufferedData.isEmpty()) {
            log.info("Flushing {} buffered chunks | Session: {}{}{}",
                    bufferedData.size(), yellow, sessionId.substring(0, 8), reset);
            for (BufferedMessage msg : bufferedData) {
                if (msg.buf.isReadable()) {
                    forwardData(msg.ctx, msg.buf);
                } else {
                    msg.buf.release();
                }
            }
            bufferedData.clear();
        }
    }
}
