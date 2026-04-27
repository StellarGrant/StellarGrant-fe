import { Server as SocketServer, type Socket } from "socket.io";
import { Server as HttpServer } from "http";
import { env } from "../config/env";

export class NotificationService {
  private io: SocketServer | null = null;
  private userSockets: Map<string, string[]> = new Map();

  initialize(server: HttpServer): void {
    this.io = new SocketServer(server, {
      cors: {
        origin: env.corsOrigins,
        methods: ["GET", "POST"],
        credentials: true,
      },
    });

    this.io.on("connection", (socket: Socket) => {
      const address = socket.handshake.query.address as string;
      if (address) {
        const sockets = this.userSockets.get(address) || [];
        sockets.push(socket.id);
        this.userSockets.set(address, sockets);
        console.log(`User ${address} connected with socket ${socket.id}`);

        socket.on("disconnect", () => {
          const updatedSockets = (this.userSockets.get(address) || []).filter(
            (id) => id !== socket.id
          );
          if (updatedSockets.length === 0) {
            this.userSockets.delete(address);
          } else {
            this.userSockets.set(address, updatedSockets);
          }
          console.log(`User ${address} disconnected socket ${socket.id}`);
        });
      }
    });
  }

  notifyUser(address: string, type: string, payload: any): void {
    if (!this.io) return;

    const sockets = this.userSockets.get(address);
    if (sockets && sockets.length > 0) {
      sockets.forEach((socketId) => {
        this.io?.to(socketId).emit("notification", {
          type,
          payload,
          timestamp: new Date().toISOString(),
        });
      });
      console.log(`Notification sent to ${address}: ${type}`);
    } else {
      console.log(`No active sockets for user ${address}, notification cached/skipped`);
    }
  }

  broadcast(type: string, payload: any): void {
    if (!this.io) return;
    this.io.emit("notification", {
      type,
      payload,
      timestamp: new Date().toISOString(),
    });
    console.log(`Broadcasted notification: ${type}`);
  }
}

export const notificationService = new NotificationService();
