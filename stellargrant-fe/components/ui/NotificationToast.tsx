"use client";

import React, { useEffect, useState } from "react";
import { motion, AnimatePresence } from "framer-motion";
import { useSocket } from "@/hooks/useSocket";
import { Bell, CheckCircle, Info, Rocket, X } from "lucide-react";

export const NotificationToast: React.FC = () => {
  const { lastNotification } = useSocket();
  const [visible, setVisible] = useState(false);
  const [currentNotification, setCurrentNotification] = useState<any>(null);

  useEffect(() => {
    if (lastNotification) {
      setCurrentNotification(lastNotification);
      setVisible(true);
      const timer = setTimeout(() => setVisible(false), 5000);
      return () => clearTimeout(timer);
    }
  }, [lastNotification]);

  if (!currentNotification) return null;

  const getIcon = () => {
    switch (currentNotification.type) {
      case "grant_created":
        return <Info className="text-blue-400" size={20} />;
      case "grant_updated":
        return <CheckCircle className="text-green-400" size={20} />;
      case "milestone_submitted":
        return <Rocket className="text-orange-400" size={20} />;
      default:
        return <Bell className="text-purple-400" size={20} />;
    }
  };

  const getTitle = () => {
    switch (currentNotification.type) {
      case "grant_created":
        return "New Grant Created";
      case "grant_updated":
        return "Grant Updated";
      case "milestone_submitted":
        return "Milestone Submitted";
      default:
        return "Notification";
    }
  };

  const getMessage = () => {
    const { payload } = currentNotification;
    switch (currentNotification.type) {
      case "grant_created":
        return `Grant "${payload.title}" has been successfully registered on-chain.`;
      case "grant_updated":
        return `Grant "${payload.title}" status changed from ${payload.oldStatus} to ${payload.newStatus}.`;
      case "milestone_submitted":
        return `A new milestone proof has been submitted for Grant #${payload.grantId} (Milestone ${payload.milestoneIdx + 1}).`;
      default:
        return "You have a new update.";
    }
  };

  return (
    <AnimatePresence>
      {visible && (
        <motion.div
          initial={{ opacity: 0, y: 50, scale: 0.9 }}
          animate={{ opacity: 1, y: 0, scale: 1 }}
          exit={{ opacity: 0, scale: 0.9, transition: { duration: 0.2 } }}
          className="fixed bottom-6 right-6 z-50 max-w-sm w-full"
        >
          <div className="bg-slate-900/90 backdrop-blur-xl border border-slate-800 rounded-2xl p-4 shadow-2xl flex items-start gap-4">
            <div className="bg-slate-800/50 p-2 rounded-xl">
              {getIcon()}
            </div>
            <div className="flex-1">
              <h4 className="text-sm font-bold text-white mb-1">{getTitle()}</h4>
              <p className="text-xs text-slate-400 leading-relaxed">
                {getMessage()}
              </p>
            </div>
            <button 
              onClick={() => setVisible(false)}
              className="text-slate-500 hover:text-white transition-colors"
            >
              <X size={16} />
            </button>
          </div>
        </motion.div>
      )}
    </AnimatePresence>
  );
};
