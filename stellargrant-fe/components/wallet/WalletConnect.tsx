/**
 * WalletConnect Component
 * 
 * Primary connect button. Detects installed wallet extensions and shows
 * a modal to select Freighter, xBull, or Passkey.
 */

interface WalletConnectProps {
  variant?: "button" | "icon";
  onConnect?: (address: string) => void;
}

export function WalletConnect({ variant: _variant = "button", onConnect: _onConnect }: WalletConnectProps) {
  // TODO: Implement wallet connect component
  return (
    <button className="px-4 py-2 bg-stellar-blue text-white rounded">
      Connect Wallet
    </button>
  );
}
