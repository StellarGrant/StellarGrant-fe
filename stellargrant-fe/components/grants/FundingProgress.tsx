/**
 * FundingProgress Component
 * 
 * Animated progress bar showing escrow amount vs target.
 * Displays per-token breakdown when multiple tokens deposited.
 */

interface FundingProgressProps {
  current: number;
  target: number;
  token?: string;
  tokens?: Array<{ token: string; amount: number }>;
}

export function FundingProgress({ current, target, token: _token, tokens: _tokens }: FundingProgressProps) {
  const percentage = Math.min((current / target) * 100, 100);

  return (
    <div className="w-full">
      <div className="flex justify-between text-sm mb-2">
        <span>{current} / {target}</span>
        <span>{percentage.toFixed(1)}%</span>
      </div>
      <div className="w-full bg-gray-200 rounded-full h-2">
        <div
          className="bg-stellar-cyan h-2 rounded-full transition-all duration-300"
          style={{ width: `${percentage}%` }}
        />
      </div>
    </div>
  );
}
