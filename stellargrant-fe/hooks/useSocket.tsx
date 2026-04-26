"use client";

import React, { createContext, useContext, useEffect, useState } from "react";
import { io, Socket } from "socket.io-client";
import { useWalletStore } from "@/lib/store/walletStore";

interface Notification {
  type: string;
  payload: any;
  timestamp: string;
}

interface SocketContextType {
  socket: Socket | null;
  connected: boolean;
  lastNotification: Notification | null;
}

const SocketContext = createContext<SocketContextType>({
  socket: null,
  connected: false,
  lastNotification: null,
});

export const useSocket = () => useContext(SocketContext);

export const SocketProvider: React.FC<{ children: React.ReactNode }> = ({ children }) => {
  const { address } = useWalletStore();
  const [socket, setSocket] = useState<Socket | null>(null);
  const [connected, setConnected] = useState(false);
  const [lastNotification, setLastNotification] = useState<Notification | null>(null);

  useEffect(() => {
    if (!address) {
      if (socket) {
        socket.disconnect();
        setSocket(null);
      }
      return;
    }

    const socketUrl = process.env.NEXT_PUBLIC_API_URL || "http://localhost:3001";
    const newSocket = io(socketUrl, {
      query: { address },
      reconnectionAttempts: 5,
      reconnectionDelay: 1000,
    });

    newSocket.on("connect", () => {
      setConnected(true);
      console.log("WebSocket connected");
    });

    newSocket.on("disconnect", () => {
      setConnected(false);
      console.log("WebSocket disconnected");
    });

    newSocket.on("notification", (data: Notification) => {
      setLastNotification(data);
      console.log("New notification received:", data);
    });

    setSocket(newSocket);

    return () => {
      newSocket.disconnect();
    };
  }, [address]);

  return (
    <SocketContext.Provider value={{ socket, connected, lastNotification }}>
      {children}
    </SocketContext.Provider>
  );
};
