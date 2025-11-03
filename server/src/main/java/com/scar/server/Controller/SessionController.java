package com.scar.server.Controller;

import com.scar.server.Dto.ListenRequest;
import com.scar.server.Dto.ListenResponse;
import com.scar.server.Dto.ServeRequest;
import com.scar.server.Dto.ServeResponse;
import com.scar.server.Service.SessionService;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.http.ResponseEntity;
import org.springframework.web.bind.annotation.*;
import org.springframework.web.context.request.async.DeferredResult;

@RestController
@RequestMapping("/api/relay")
public class SessionController {

        private static final Logger log = LoggerFactory.getLogger(SessionController.class);

        private final SessionService sessionService;

        @Autowired
        public SessionController(SessionService sessionService) {
                this.sessionService = sessionService;
        }

        /**
         * Sender calls: POST /api/relay/serve
         * BLOCKS until Receiver accepts or 30 second timeout
         */
        @PostMapping("/serve")
        public DeferredResult<ResponseEntity<ServeResponse>> initiate(
                        @RequestBody ServeRequest request) {

                log.info("Session request from sender {} to receiver {}",
                                request.getSenderFp(),
                                request.getReceiverFp()

                );

                DeferredResult<ResponseEntity<ServeResponse>> result = new DeferredResult<>(30_000L); // 30 second
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

                // Call blocking service
                sessionService.initiateAndWait(
                                request.getSenderFp(),
                                request.getReceiverFp(),
                                request.getFilename(),
                                request.getFileSize(),
                                request.getSignature()).thenAccept(session -> {
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

                log.info("Listen request from receiver {}",
                                request.getReceiverFp());

                DeferredResult<ResponseEntity<ListenResponse>> result = new DeferredResult<>(30_000L); // 30 second
                                                                                                       // timeout

                // Validate
                if (request.getReceiverFp() == null || request.getReceiverFp().isEmpty()) {
                        result.setResult(ResponseEntity.badRequest().body(
                                        new ListenResponse("error", null, null, null, 0, null, 0,
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
                                                                        session.getSocketPort(),
                                                                        "Incoming transfer from " + session
                                                                                        .getSenderFp()
                                                                                        .substring(0, 8))));
                                }).exceptionally(ex -> {
                                        log.error("Listen failed: {}", ex.getMessage());
                                        result.setResult(ResponseEntity.status(408).body(
                                                        new ListenResponse("timeout", null, null, null, 0, null, 0,
                                                                        "No sender found: " + ex.getMessage())));
                                        return null;
                                });

                return result;
        }

        /**
         * Optional: Mark session as complete (cleanup)
         */
        @DeleteMapping("/session/{sessionId}")
        public ResponseEntity<String> completeSession(@PathVariable String sessionId) {
                sessionService.completeSession(sessionId);
                log.info("Session marked complete: {}", sessionId.substring(0, 8));
                return ResponseEntity.ok("Session completed");
        }

        /**
         * Health check
         */
        @GetMapping("/health")
        public ResponseEntity<String> health() {
                return ResponseEntity.ok("OK");
        }
}
