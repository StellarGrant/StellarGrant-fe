/**
 * Events Stream API Route
 * 
 * Server-Sent Events endpoint that streams decoded StellarGrants contract events
 * to the browser. Polls Stellar RPC for new events and pushes them to connected clients.
 * 
 * GET /api/events?grantId=42
 */

import { NextRequest } from "next/server";

export async function GET(request: NextRequest) {
  const { searchParams } = new URL(request.url);
  const _grantId = searchParams.get("grantId");

  // TODO: Implement SSE streaming for contract events
  // This will poll Stellar RPC for new events and push them to clients

  return new Response("Event streaming not yet implemented", {
    status: 501,
    headers: {
      "Content-Type": "text/event-stream",
      "Cache-Control": "no-cache",
      Connection: "keep-alive",
    },
  });
}
