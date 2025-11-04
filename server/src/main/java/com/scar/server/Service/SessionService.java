package com.scar.server.Service;

import com.scar.server.Model.Session;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.springframework.stereotype.Service;

import java.util.Map;
import java.util.Optional;
import java.util.UUID;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.ConcurrentHashMap;
import java.util.concurrent.TimeoutException;

import static com.scar.server.Controller.SessionController.*;


@Service
public class SessionService {
    private static final long BLOCKING_TIMEOUT = 30_000; // 30 seconds
    private static final Logger log = LoggerFactory.getLogger(SessionService.class);

    private final Map<String, Session> sessions = new ConcurrentHashMap<>();
    private final Map<String, CompletableFuture<Session>> waitingSenders = new ConcurrentHashMap<>();
    private final Map<String, CompletableFuture<Session>> waitingReceivers = new ConcurrentHashMap<>();

    public CompletableFuture<Session> initiateAndWait(
            String senderFp, String receiverFp, String filename,
            long fileSize, String signature, String fileHash) {

        // Validate inputs
        if (senderFp == null || senderFp.isEmpty() ||
                receiverFp == null || receiverFp.isEmpty() ||
                filename == null || filename.isEmpty() ||
                signature == null || signature.isEmpty() ||
                fileHash == null || fileHash.isEmpty()) {
            CompletableFuture<Session> failed = new CompletableFuture<>();
            failed.completeExceptionally(new IllegalArgumentException("Missing required fields"));
            return failed;
        }

        String sessionId = UUID.randomUUID().toString();
        Session session = new Session(sessionId, senderFp, receiverFp, filename, fileSize, signature, fileHash);

        sessions.put(sessionId, session);

        log.info(" Created session: {}{}{} | {}{}{} -> {}{}{}",
                yellow, sessionId.substring(0, 8), reset,
                blue, senderFp.substring(0, 8), reset,
                red, receiverFp.substring(0, 8), reset);

        // Check if listener is already waiting
        CompletableFuture<Session> Waiting = waitingReceivers.get(receiverFp);
        if (Waiting != null) {
            log.info("Listener: {}{}{} already waiting! Matching: {}{}{} immediately",
                    red, receiverFp.substring(0, 8), reset,
                    blue, senderFp.substring(0, 8), reset);
            // Listen is waiting! Match immediately
            session.setStatus("matched");
            Waiting.complete(session); // Wake up Listener
            waitingReceivers.remove(receiverFp);
            return CompletableFuture.completedFuture(session); // Return to Sender
        }

        // Listener not waiting yet, Sender waits
        log.info("Listener: {}{}{} not ready yet, Sender: {}{}{} blocking...", red, receiverFp.substring(0, 8), reset,
                blue, senderFp.substring(0, 8), reset);
        CompletableFuture<Session> future = new CompletableFuture<>();
        waitingSenders.put(sessionId, future);

        // Timeout handling
        CompletableFuture.delayedExecutor(BLOCKING_TIMEOUT, java.util.concurrent.TimeUnit.MILLISECONDS)
                .execute(() -> {
                    if (!future.isDone()) {

                        log.warn("Session timeout: {}{}{} | {}{}{} -> {}{}{}",
                                yellow, sessionId.substring(0, 8), reset,
                                blue, senderFp.substring(0, 8), reset,
                                red, receiverFp.substring(0, 8), reset);
                        session.setStatus("timeout");
                        future.completeExceptionally(new TimeoutException("Receiver didn't respond"));
                        waitingSenders.remove(sessionId);
                        sessions.remove(sessionId);
                    }
                });

        return future;
    }

    public CompletableFuture<Session> listenAndWait(String receiverFp) {
        if (receiverFp == null || receiverFp.isEmpty()) {
            CompletableFuture<Session> failed = new CompletableFuture<>();
            failed.completeExceptionally(new IllegalArgumentException("Missing receiver fingerprint"));
            return failed;
        }

        log.info("Listener: {}{}{} waiting", red, receiverFp.substring(0, 8), reset);
        // Check if Sender already initiated
        Optional<Session> pendingSession = sessions.values().stream()
                .filter(s -> s.getReceiverFp().equals(receiverFp))
                .filter(s -> s.getStatus().equals("waiting_receiver"))
                .findFirst();

        if (pendingSession.isPresent()) {
            log.info("Sender already waiting! Matching immediately");
            Session session = pendingSession.get();
            session.setStatus("matched");

            CompletableFuture<Session> aliceWaiting = waitingSenders.get(session.getSessionId());
            if (aliceWaiting != null) {
                aliceWaiting.complete(session); // Wake up Sender
                waitingSenders.remove(session.getSessionId());
            }

            return CompletableFuture.completedFuture(session); // Return to Listener
        }

        // Sender not initiated yet, Listener waits
        log.info("Sender not ready yet, Listener: {}{}{} blocking...", red, receiverFp.substring(0, 8), reset);
        CompletableFuture<Session> future = new CompletableFuture<>();
        waitingReceivers.put(receiverFp, future);

        // Timeout handling
        CompletableFuture.delayedExecutor(BLOCKING_TIMEOUT, java.util.concurrent.TimeUnit.MILLISECONDS)
                .execute(() -> {
                    if (!future.isDone()) {
                        log.warn("Listen timeout: {}", receiverFp.substring(0, 8));
                        future.completeExceptionally(new TimeoutException("No sender found"));
                        waitingReceivers.remove(receiverFp);
                    }
                });

        return future;
    }

    public Optional<Session> getSession(String sessionId) {
        Session session = sessions.get(sessionId);
        if (session != null && session.isExpired()) {
            sessions.remove(sessionId);
            return Optional.empty();
        }
        return Optional.ofNullable(session);
    }

    public void completeSession(String sessionId) {
        Session session = sessions.get(sessionId);
        if (session != null) {
            session.setStatus("completed");
            log.info("Completed Session: {}{}{}", yellow, sessionId.substring(0, 8), reset);
        }
    }
}
