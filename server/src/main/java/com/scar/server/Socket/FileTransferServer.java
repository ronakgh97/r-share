package com.scar.server.Socket;

import com.scar.server.Service.SessionService;
import io.netty.bootstrap.ServerBootstrap;
import io.netty.channel.*;
import io.netty.channel.nio.NioEventLoopGroup;
import io.netty.channel.socket.SocketChannel;
import io.netty.channel.socket.nio.NioServerSocketChannel;
import jakarta.annotation.PostConstruct;
import jakarta.annotation.PreDestroy;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.springframework.beans.factory.annotation.Value;
import org.springframework.stereotype.Component;

@Component
public class FileTransferServer {
    private static final Logger log = LoggerFactory.getLogger(FileTransferServer.class);
    @Value("${server.socket.port:10000}")
    private int PORT;
    @Value("${rshare.netty.boss-threads:1}")
    private int bossThreads;
    @Value("${rshare.netty.worker-threads:4}")
    private int workerThreads;
    @Value("${rshare.netty.backlog:128}")
    private int backlog;

    private EventLoopGroup bossGroup;
    private EventLoopGroup workerGroup;
    private final SocketSessionRegistry registry;
    private final SessionService sessionService;

    public FileTransferServer(SocketSessionRegistry registry, SessionService sessionService) {
        this.registry = registry;
        this.sessionService = sessionService;
    }

    @PostConstruct
    public void start() throws Exception {
        bossGroup = new NioEventLoopGroup(bossThreads);
        workerGroup = new NioEventLoopGroup(workerThreads);

        new Thread(() -> {
            try {
                ServerBootstrap bootstrap = new ServerBootstrap();
                bootstrap.group(bossGroup, workerGroup)
                        .channel(NioServerSocketChannel.class)
                        .childHandler(new ChannelInitializer<SocketChannel>() {
                            @Override
                            protected void initChannel(SocketChannel ch) {
                                ch.pipeline()
                                        // No frame decoder - raw bytes only!
                                        // Handshake: "session_id:role\n" (text)
                                        // binary file data (no framing)
                                        .addLast(new FileTransferHandler(registry, sessionService));
                            }
                        })
                        .option(ChannelOption.SO_BACKLOG, backlog)
                        .childOption(ChannelOption.SO_KEEPALIVE, true)
                        .childOption(ChannelOption.TCP_NODELAY, true)
                        .childOption(ChannelOption.AUTO_READ, true)
                        .childOption(ChannelOption.SO_SNDBUF, 2 * 1024 * 1024) // 2MB send buffer
                        .childOption(ChannelOption.SO_RCVBUF, 2 * 1024 * 1024); // 2MB receive buffer

                ChannelFuture future = bootstrap.bind(PORT).sync();
                log.info("Socket Server running on port {}", PORT);
                future.channel().closeFuture().sync();
            } catch (Exception e) {
                log.error("Socket server failed to start", e);
            }
        }, "socket-server-thread").start();
    }

    @PreDestroy
    public void stop() {
        log.info("Shutting down socket server...");
        if (workerGroup != null) {
            workerGroup.shutdownGracefully();
        }
        if (bossGroup != null) {
            bossGroup.shutdownGracefully();
        }
    }
}
