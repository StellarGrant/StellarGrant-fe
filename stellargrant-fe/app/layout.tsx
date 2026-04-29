import type { Metadata } from "next";
import { Orbitron, IBM_Plex_Mono } from "next/font/google";
import "./globals.css";
import { SocketProvider } from "@/hooks/useSocket";
import { NotificationToast } from "@/components/ui/NotificationToast";

const orbitron = Orbitron({
  variable: "--font-orbitron",
  subsets: ["latin"],
  weight: ["400", "700", "900"],
});

const ibmPlexMono = IBM_Plex_Mono({
  variable: "--font-ibm-plex-mono",
  subsets: ["latin"],
  weight: ["400", "500"],
});

export const metadata: Metadata = {
  title: "StellarGrant Protocol",
  description: "Decentralized milestone-based grant management on Stellar",
  other: {
    "theme-color": "#050A14",
  },
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en">
      <body
        className={`${orbitron.variable} ${ibmPlexMono.variable} antialiased`}
      >
        <SocketProvider>
          {children}
          <NotificationToast />
        </SocketProvider>
      </body>
    </html>
  );
}
