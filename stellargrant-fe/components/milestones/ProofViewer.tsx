/**
 * ProofViewer Component
 * 
 * Renders the submitted milestone proof. If the proof_hash is an IPFS CID,
 * fetches and displays from the configured gateway.
 * Supports markdown, plain text, and image rendering.
 */

interface ProofViewerProps {
  proofHash: string;
  proofType?: "text" | "markdown" | "image" | "ipfs";
}

export function ProofViewer({ proofHash, proofType: _proofType = "ipfs" }: ProofViewerProps) {
  // TODO: Implement proof viewer component
  return (
    <div className="border rounded p-4">
      <p className="text-sm text-muted-foreground">Proof Hash: {proofHash}</p>
      {/* Proof viewer will be implemented here */}
    </div>
  );
}
