/**
 * WalletGuard Component
 * 
 * Wrapper component that renders children only when a wallet is connected.
 * Shows a connect prompt otherwise.
 */

interface WalletGuardProps {
  children: React.ReactNode;
  requiredRole?: "any" | "reviewer" | "contributor" | "owner";
}

export function WalletGuard({ children, requiredRole: _requiredRole = "any" }: WalletGuardProps) {
  // TODO: Implement wallet guard with role checking
  const isConnected = false; // This will be connected to useWallet hook

  if (!isConnected) {
    return (
      <div className="text-center p-8">
        <p className="text-muted-foreground mb-4">Please connect your wallet to continue</p>
        {/* WalletConnect component will be rendered here */}
      </div>
    );
  }

  return <>{children}</>;
}
